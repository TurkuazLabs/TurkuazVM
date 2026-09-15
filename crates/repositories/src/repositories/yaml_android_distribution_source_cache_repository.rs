// # 📄 Dosya Yolu: /turkuazvm/crates/repositories/src/repositories/yaml_android_distribution_source_cache_repository.rs
// # 📌 Amac: Android resolved distribution source last-known-good kayitlarini YAML dosyasinda atomik olarak saklar
// # 📌 Modul - Rust
// # Version: 0.40.0
// # Aciklama: Release+mimari cache anahtariyla immutable CI build, artifact URL ve same-build host durumunu persist eder
// # Bagimli Oldugu Katman: Repo | Service

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use turkuazvm_android_image::domain::distribution_source::{
    AndroidDistributionSource, AndroidDistributionSourceError,
};
use turkuazvm_android_image::ports::android_distribution_source_cache_port::AndroidDistributionSourceCachePort;

const CACHE_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone)]
pub struct YamlAndroidDistributionSourceCacheRepository {
    path: PathBuf,
}

impl YamlAndroidDistributionSourceCacheRepository {
    pub const fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn read_cache(&self) -> Result<CacheFile, AndroidDistributionSourceError> {
        if !self.path.is_file() {
            return Ok(CacheFile { schema_version: CACHE_SCHEMA_VERSION, entries: BTreeMap::new() });
        }
        let content = fs::read_to_string(&self.path)
            .map_err(|error| AndroidDistributionSourceError::Cache(error.to_string()))?;
        let file: CacheFile = serde_yaml_ng::from_str(&content)
            .map_err(|error| AndroidDistributionSourceError::Cache(error.to_string()))?;
        if file.schema_version != CACHE_SCHEMA_VERSION {
            return Err(AndroidDistributionSourceError::Cache(format!(
                "unsupported Android source cache schema {}",
                file.schema_version
            )));
        }
        Ok(file)
    }

    fn write_cache(&self, file: &CacheFile) -> Result<(), AndroidDistributionSourceError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| AndroidDistributionSourceError::Cache(error.to_string()))?;
        }
        let body = serde_yaml_ng::to_string(file)
            .map_err(|error| AndroidDistributionSourceError::Cache(error.to_string()))?;
        let content = format!(
            "# 📄 Dosya Yolu: /turkuazvm/data/cache/android-distribution-sources.yml\n# 📌 Amac: Android runtime source resolver last-known-good kayitlarini saklar\n# 📌 Modul - YAML\n# Version: {}\n# Aciklama: Resmi Android CI discovery gecici kullanilamazsa daha once dogrulanmis immutable build kaydini kullanir\n# Bagimli Oldugu Katman: Repo | Service\n\n{}",
            env!("CARGO_PKG_VERSION"),
            body
        );
        atomic_write(&self.path, content.as_bytes())
            .map_err(AndroidDistributionSourceError::Cache)
    }
}

impl AndroidDistributionSourceCachePort for YamlAndroidDistributionSourceCacheRepository {
    fn load(&self, cache_key: &str) -> Result<Option<AndroidDistributionSource>, AndroidDistributionSourceError> {
        let file = self.read_cache()?;
        Ok(file.entries.get(cache_key).cloned().map(Into::into))
    }

    fn save(&self, source: &AndroidDistributionSource) -> Result<(), AndroidDistributionSourceError> {
        let mut file = self.read_cache()?;
        file.entries.insert(source.cache_key.clone(), SourceDto::from(source));
        self.write_cache(&file)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheFile {
    schema_version: u16,
    entries: BTreeMap<String, SourceDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SourceDto {
    cache_key: String,
    provider: String,
    base_url: String,
    branch: String,
    target: String,
    build_id: String,
    artifact_base_url: String,
    device_artifact_name: String,
    host_artifact_name: String,
    host_package_available: bool,
    allow_device_bootloader_fallback: bool,
    android_release: Option<String>,
    sdk_level: Option<u32>,
    resolved_at_unix: u64,
}

impl From<&AndroidDistributionSource> for SourceDto {
    fn from(source: &AndroidDistributionSource) -> Self {
        Self {
            cache_key: source.cache_key.clone(),
            provider: source.provider.clone(),
            base_url: source.base_url.clone(),
            branch: source.branch.clone(),
            target: source.target.clone(),
            build_id: source.build_id.clone(),
            artifact_base_url: source.artifact_base_url.clone(),
            device_artifact_name: source.device_artifact_name.clone(),
            host_artifact_name: source.host_artifact_name.clone(),
            host_package_available: source.host_package_available,
            allow_device_bootloader_fallback: source.allow_device_bootloader_fallback,
            android_release: source.android_release.clone(),
            sdk_level: source.sdk_level,
            resolved_at_unix: source.resolved_at_unix,
        }
    }
}

impl From<SourceDto> for AndroidDistributionSource {
    fn from(source: SourceDto) -> Self {
        Self {
            cache_key: source.cache_key,
            provider: source.provider,
            base_url: source.base_url,
            branch: source.branch,
            target: source.target,
            build_id: source.build_id,
            artifact_base_url: source.artifact_base_url,
            device_artifact_name: source.device_artifact_name,
            host_artifact_name: source.host_artifact_name,
            host_package_available: source.host_package_available,
            allow_device_bootloader_fallback: source.allow_device_bootloader_fallback,
            android_release: source.android_release,
            sdk_level: source.sdk_level,
            resolved_at_unix: source.resolved_at_unix,
        }
    }
}

fn atomic_write(path: &Path, content: &[u8]) -> Result<(), String> {
    let temp = PathBuf::from(format!("{}.tmp", path.display()));
    if temp.exists() {
        fs::remove_file(&temp).map_err(|error| error.to_string())?;
    }
    fs::write(&temp, content).map_err(|error| error.to_string())?;
    if path.exists() {
        fs::remove_file(path).map_err(|error| error.to_string())?;
    }
    fs::rename(&temp, path).map_err(|error| error.to_string())
}
