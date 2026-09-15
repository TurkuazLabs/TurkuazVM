// # 📄 Dosya Yolu: /turkuazvm/apps/engine/src/services/artifact_cache_application_service.rs
// # 📌 Amac: Artifact Cache yonetim use-case'lerini ve downloader icin shared cache client'i orkestre eder
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Tek Arc/Mutex Cache Service ile background download ve Desktop yonetim islemlerini race olmadan serilestirir
// # Bagimli Oldugu Katman: Service | Repo | Tool

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::{Arc, Mutex};

use turkuazvm_artifact_cache::domain::artifact_cache::{
    ArtifactCacheRecord, ArtifactCacheRequest, ArtifactCacheRevalidation,
    ArtifactCacheRevalidationSummary, ArtifactCacheStats, ArtifactCacheVerification,
};
use turkuazvm_artifact_cache::ports::artifact_cache_client_port::ArtifactCacheClientPort;
use turkuazvm_artifact_cache::services::artifact_cache_service::ArtifactCacheService;
use turkuazvm_guest::tools::artifact_cache_http_fetch_tool::{
    ArtifactCacheHttpFetchSettings, ArtifactCacheHttpFetchTool,
};
use turkuazvm_guest::tools::artifact_cache_http_policy_tool::ArtifactHttpPolicy;
use turkuazvm_guest::tools::artifact_cache_http_validation_tool::{
    ArtifactCacheHttpValidationSettings, CurlArtifactSourceValidationTool,
};
use turkuazvm_repositories::repositories::yaml_artifact_cache_repository::YamlArtifactCacheRepository;

use crate::config::engine_config::ArtifactCacheEngineConfig;

type EngineArtifactCacheService = ArtifactCacheService<
    YamlArtifactCacheRepository,
    CurlArtifactSourceValidationTool,
>;

type SharedEngineArtifactCacheService = Arc<Mutex<EngineArtifactCacheService>>;
pub type SharedArtifactCacheClient = Arc<Mutex<Box<dyn ArtifactCacheClientPort>>>;

struct EngineArtifactCacheClient {
    service: SharedEngineArtifactCacheService,
}

impl ArtifactCacheClientPort for EngineArtifactCacheClient {
    fn acquire_source_lock(&mut self, source_key: &str) -> Result<(), String> {
        let mut service = self.service.lock().map_err(|_| String::from("Artifact Cache service lock poisoned"))?;
        ArtifactCacheClientPort::acquire_source_lock(&mut *service, source_key)
    }

    fn release_source_lock(&mut self, source_key: &str) -> Result<(), String> {
        let mut service = self.service.lock().map_err(|_| String::from("Artifact Cache service lock poisoned"))?;
        ArtifactCacheClientPort::release_source_lock(&mut *service, source_key)
    }

    fn restore(
        &mut self,
        request: &ArtifactCacheRequest,
        destination: &Path,
    ) -> Result<Option<ArtifactCacheRecord>, String> {
        self.service
            .lock()
            .map_err(|_| String::from("Artifact Cache service lock poisoned"))?
            .restore(request, destination)
    }

    fn store(
        &mut self,
        request: &ArtifactCacheRequest,
        source_path: &Path,
    ) -> Result<ArtifactCacheRecord, String> {
        self.service
            .lock()
            .map_err(|_| String::from("Artifact Cache service lock poisoned"))?
            .store(request, source_path)
    }
}

pub struct ArtifactCacheApplicationService {
    enabled: bool,
    verify_on_hit: bool,
    allow_stale_on_transient_error: bool,
    service: SharedEngineArtifactCacheService,
    fetcher: ArtifactCacheHttpFetchTool,
    incoming_root: PathBuf,
}

impl ArtifactCacheApplicationService {
    pub fn new(
        config: ArtifactCacheEngineConfig,
        curl_binary: PathBuf,
    ) -> Self {
        let cache_root = config.root.clone();
        let policy = ArtifactHttpPolicy {
            require_https: config.mutable.require_https,
            allow_private_networks: config.mutable.allow_private_networks,
            max_redirects: config.mutable.max_redirects,
        };
        let repository = YamlArtifactCacheRepository::new(config.root);
        let validator = CurlArtifactSourceValidationTool::new(ArtifactCacheHttpValidationSettings {
            curl_binary: curl_binary.clone(),
            connect_timeout_seconds: config.downloader.connect_timeout_seconds,
            request_timeout: config.mutable.revalidation_timeout,
            policy,
        });
        let fetcher = ArtifactCacheHttpFetchTool::new(ArtifactCacheHttpFetchSettings {
            curl_binary,
            connect_timeout_seconds: config.downloader.connect_timeout_seconds,
            request_timeout: config.mutable.revalidation_timeout,
            retry_count: config.downloader.retry_count,
            retry_delay_seconds: config.downloader.retry_delay_seconds,
            policy,
        });
        let service = ArtifactCacheService::with_validator(
            repository,
            validator,
            config.quota_bytes,
            config.verify_on_hit,
            config.mutable.allow_stale_on_transient_error,
        );
        Self {
            enabled: config.enabled,
            verify_on_hit: config.verify_on_hit,
            allow_stale_on_transient_error: config.mutable.allow_stale_on_transient_error,
            service: Arc::new(Mutex::new(service)),
            fetcher,
            incoming_root: cache_root.join("incoming"),
        }
    }

    pub fn client(&self) -> Option<SharedArtifactCacheClient> {
        if !self.enabled {
            return None;
        }
        let client: Box<dyn ArtifactCacheClientPort> = Box::new(EngineArtifactCacheClient {
            service: Arc::clone(&self.service),
        });
        Some(Arc::new(Mutex::new(client)))
    }

    pub const fn enabled(&self) -> bool {
        self.enabled
    }

    pub const fn verify_on_hit(&self) -> bool {
        self.verify_on_hit
    }

    pub const fn allow_stale_on_transient_error(&self) -> bool {
        self.allow_stale_on_transient_error
    }

    pub fn stats(&self) -> Result<ArtifactCacheStats, String> {
        self.lock()?.stats()
    }

    pub fn list(&self) -> Result<Vec<ArtifactCacheRecord>, String> {
        self.lock()?.list_records()
    }

    pub fn verify(&mut self) -> Result<ArtifactCacheVerification, String> {
        self.require_enabled()?;
        self.lock()?.verify_all()
    }

    pub fn cleanup(&mut self) -> Result<ArtifactCacheStats, String> {
        self.require_enabled()?;
        self.lock()?.clean_to_quota()
    }

    pub fn set_pinned(&mut self, source_key: &str, pinned: bool) -> Result<bool, String> {
        self.require_enabled()?;
        self.lock()?.set_pinned(source_key, pinned)
    }

    pub fn remove(&mut self, source_key: &str) -> Result<bool, String> {
        self.require_enabled()?;
        self.lock()?.remove_source(source_key)
    }

    pub fn revalidate(&mut self, source_key: &str) -> Result<ArtifactCacheRevalidation, String> {
        self.require_enabled()?;
        self.lock()?.revalidate_source(source_key)
    }

    pub fn revalidate_all(&mut self) -> Result<ArtifactCacheRevalidationSummary, String> {
        self.require_enabled()?;
        self.lock()?.revalidate_all()
    }

    pub fn fetch_mutable(
        &mut self,
        source_key: &str,
        source_url: &str,
        pinned: bool,
    ) -> Result<ArtifactCacheRecord, String> {
        self.require_enabled()?;
        validate_source_key(source_key)?;
        self.acquire_source_lock(source_key)?;
        let result = self.fetch_mutable_locked(source_key, source_url, pinned);
        let release = self.release_source_lock(source_key);
        match (result, release) {
            (Ok(record), Ok(())) => Ok(record),
            (Err(error), Ok(())) => Err(error),
            (Ok(_), Err(error)) => Err(error),
            (Err(error), Err(release_error)) => Err(format!("{error}; source lock release failed: {release_error}")),
        }
    }

    fn fetch_mutable_locked(
        &mut self,
        source_key: &str,
        source_url: &str,
        pinned: bool,
    ) -> Result<ArtifactCacheRecord, String> {
        if let Some(existing) = self.lock()?.find_record(source_key)? {
            if existing.source_url != source_url {
                return Err(String::from("artifact source_key is already bound to a different URL; remove the existing entry before rebinding"));
            }
        }
        fs::create_dir_all(&self.incoming_root).map_err(|error| error.to_string())?;
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH).map(|value| value.as_nanos()).unwrap_or(0);
        let temp = self.incoming_root.join(format!("fetch-{}-{nonce}.part", std::process::id()));
        let fetched = match self.fetcher.fetch(source_url, &temp) {
            Ok(value) => value,
            Err(error) => { let _ = fs::remove_file(&temp); return Err(error); }
        };
        let request = ArtifactCacheRequest {
            source_key: source_key.to_owned(),
            source_url: source_url.to_owned(),
            immutable: false,
            pinned,
            validators: fetched.validators,
        };
        let stored = {
            let mut service = self.lock()?;
            ArtifactCacheClientPort::store(&mut *service, &request, &temp)
        };
        let _ = fs::remove_file(&temp);
        stored
    }

    fn acquire_source_lock(&self, source_key: &str) -> Result<(), String> {
        let mut service = self.lock()?;
        ArtifactCacheClientPort::acquire_source_lock(&mut *service, source_key)
    }

    fn release_source_lock(&self, source_key: &str) -> Result<(), String> {
        let mut service = self.lock()?;
        ArtifactCacheClientPort::release_source_lock(&mut *service, source_key)
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, EngineArtifactCacheService>, String> {
        self.service
            .lock()
            .map_err(|_| String::from("Artifact Cache service lock poisoned"))
    }

    fn require_enabled(&self) -> Result<(), String> {
        if self.enabled {
            Ok(())
        } else {
            Err(String::from("artifact cache is disabled by configuration"))
        }
    }
}

fn validate_source_key(source_key: &str) -> Result<(), String> {
    if source_key.is_empty() || source_key.len() > 200
        || !source_key.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':' | b'/'))
    {
        return Err(String::from("artifact source_key contains unsupported characters"));
    }
    Ok(())
}
