// # 📄 Dosya Yolu: /turkuazvm/crates/artifact-cache/src/services/artifact_cache_service.rs
// # 📌 Amac: Artifact Cache hit, integrity, mutable revalidation, pin, quota ve LRU is kurallarini uygular
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Immutable hizli HIT, ETag/Last-Modified conditional revalidation, offline stale policy ve cache yonetimini saglar
// # Bagimli Oldugu Katman: Repo | Tool

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::domain::artifact_cache::{
    ArtifactCacheRecord, ArtifactCacheRequest, ArtifactCacheRevalidation,
    ArtifactCacheRevalidationState, ArtifactCacheRevalidationSummary, ArtifactCacheStats,
    ArtifactCacheVerification,
};
use crate::ports::artifact_cache_client_port::ArtifactCacheClientPort;
use crate::ports::artifact_cache_repository_port::ArtifactCacheRepositoryPort;
use crate::ports::artifact_source_validation_port::{
    ArtifactSourceValidationOutcome, ArtifactSourceValidationPort, NoopArtifactSourceValidationPort,
};

pub struct ArtifactCacheService<R, V = NoopArtifactSourceValidationPort>
where
    R: ArtifactCacheRepositoryPort,
    V: ArtifactSourceValidationPort,
{
    repository: R,
    validator: V,
    quota_bytes: u64,
    verify_on_hit: bool,
    allow_stale_on_transient_error: bool,
}

impl<R> ArtifactCacheService<R, NoopArtifactSourceValidationPort>
where
    R: ArtifactCacheRepositoryPort,
{
    pub const fn new(repository: R, quota_bytes: u64, verify_on_hit: bool) -> Self {
        Self {
            repository,
            validator: NoopArtifactSourceValidationPort,
            quota_bytes,
            verify_on_hit,
            allow_stale_on_transient_error: true,
        }
    }
}

impl<R, V> ArtifactCacheService<R, V>
where
    R: ArtifactCacheRepositoryPort,
    V: ArtifactSourceValidationPort,
{
    pub const fn with_validator(
        repository: R,
        validator: V,
        quota_bytes: u64,
        verify_on_hit: bool,
        allow_stale_on_transient_error: bool,
    ) -> Self {
        Self {
            repository,
            validator,
            quota_bytes,
            verify_on_hit,
            allow_stale_on_transient_error,
        }
    }

    pub const fn verify_on_hit(&self) -> bool {
        self.verify_on_hit
    }

    pub const fn allow_stale_on_transient_error(&self) -> bool {
        self.allow_stale_on_transient_error
    }

    pub fn stats(&self) -> Result<ArtifactCacheStats, String> {
        let records = self.repository.list().map_err(debug_error)?;
        Ok(ArtifactCacheStats {
            artifact_count: records.len(),
            pinned_count: records.iter().filter(|record| record.pinned).count(),
            mutable_count: records.iter().filter(|record| !record.immutable).count(),
            used_bytes: unique_payload_bytes(&records),
            quota_bytes: self.quota_bytes,
        })
    }

    pub fn list_records(&self) -> Result<Vec<ArtifactCacheRecord>, String> {
        self.repository.list().map_err(debug_error)
    }

    pub fn find_record(&self, source_key: &str) -> Result<Option<ArtifactCacheRecord>, String> {
        self.repository.find_by_source(source_key).map_err(debug_error)
    }

    pub fn verify_all(&mut self) -> Result<ArtifactCacheVerification, String> {
        let records = self.repository.list().map_err(debug_error)?;
        let mut valid = 0_usize;
        let mut invalid = 0_usize;
        for record in &records {
            if self.repository.verify(record).map_err(debug_error)? {
                valid += 1;
            } else {
                invalid += 1;
                self.repository.remove(record).map_err(debug_error)?;
            }
        }
        Ok(ArtifactCacheVerification { checked: records.len(), valid, invalid })
    }

    pub fn clean_to_quota(&mut self) -> Result<ArtifactCacheStats, String> {
        let mut records = self.repository.list().map_err(debug_error)?;
        records.sort_by_key(|record| record.last_access_unix_ms);
        while unique_payload_bytes(&records) > self.quota_bytes {
            let Some(index) = records.iter().position(|record| !record.pinned) else { break; };
            let record = records.remove(index);
            self.repository.remove(&record).map_err(debug_error)?;
        }
        self.stats()
    }

    pub fn set_pinned(&mut self, source_key: &str, pinned: bool) -> Result<bool, String> {
        let Some(mut record) = self.repository.find_by_source(source_key).map_err(debug_error)? else { return Ok(false); };
        record.pinned = pinned;
        record.last_access_unix_ms = now_unix_ms();
        self.repository.save_record(&record).map_err(debug_error)?;
        Ok(true)
    }

    pub fn remove_source(&mut self, source_key: &str) -> Result<bool, String> {
        let Some(record) = self.repository.find_by_source(source_key).map_err(debug_error)? else { return Ok(false); };
        self.repository.remove(&record).map_err(debug_error)?;
        Ok(true)
    }

    pub fn revalidate_source(&mut self, source_key: &str) -> Result<ArtifactCacheRevalidation, String> {
        let Some(mut record) = self.repository.find_by_source(source_key).map_err(debug_error)? else {
            return Err(format!("artifact cache source not found: {source_key}"));
        };
        if record.immutable {
            return Ok(ArtifactCacheRevalidation {
                source_key: record.source_key,
                state: ArtifactCacheRevalidationState::Immutable,
                validators: record.validators,
                detail: None,
            });
        }
        let now = now_unix_ms();
        match self.validator.revalidate(&record.source_url, &record.validators) {
            Ok(ArtifactSourceValidationOutcome::NotModified { validators }) => {
                record.validators = record.validators.merge(&validators);
                record.last_revalidated_unix_ms = Some(now);
                self.repository.save_record(&record).map_err(debug_error)?;
                Ok(ArtifactCacheRevalidation {
                    source_key: record.source_key,
                    state: ArtifactCacheRevalidationState::NotModified,
                    validators: record.validators,
                    detail: None,
                })
            }
            Ok(ArtifactSourceValidationOutcome::Modified { validators }) => {
                record.last_revalidated_unix_ms = Some(now);
                self.repository.save_record(&record).map_err(debug_error)?;
                Ok(ArtifactCacheRevalidation {
                    source_key: record.source_key,
                    state: ArtifactCacheRevalidationState::RemoteModified,
                    validators,
                    detail: Some(String::from("remote source changed; last-known-good payload retained until successful replacement")),
                })
            }
            Err(error) => Ok(ArtifactCacheRevalidation {
                source_key: record.source_key,
                state: ArtifactCacheRevalidationState::ValidationFailed,
                validators: record.validators,
                detail: Some(format!("{error:?}")),
            }),
        }
    }

    pub fn revalidate_all(&mut self) -> Result<ArtifactCacheRevalidationSummary, String> {
        let records = self.repository.list().map_err(debug_error)?;
        let mut summary = ArtifactCacheRevalidationSummary {
            checked: records.len(),
            immutable: 0,
            not_modified: 0,
            remote_modified: 0,
            failed: 0,
        };
        for record in records {
            match self.revalidate_source(&record.source_key)?.state {
                ArtifactCacheRevalidationState::Immutable => summary.immutable += 1,
                ArtifactCacheRevalidationState::NotModified => summary.not_modified += 1,
                ArtifactCacheRevalidationState::RemoteModified => summary.remote_modified += 1,
                ArtifactCacheRevalidationState::ValidationFailed => summary.failed += 1,
            }
        }
        Ok(summary)
    }

    fn prepare_mutable_restore(&mut self, record: &mut ArtifactCacheRecord) -> Result<bool, String> {
        if record.immutable {
            return Ok(true);
        }
        let now = now_unix_ms();
        match self.validator.revalidate(&record.source_url, &record.validators) {
            Ok(ArtifactSourceValidationOutcome::NotModified { validators }) => {
                record.validators = record.validators.merge(&validators);
                record.last_revalidated_unix_ms = Some(now);
                self.repository.save_record(record).map_err(debug_error)?;
                Ok(true)
            }
            Ok(ArtifactSourceValidationOutcome::Modified { .. }) => Ok(false),
            Err(crate::ports::artifact_source_validation_port::ArtifactSourceValidationError::Transient(_))
                if self.allow_stale_on_transient_error => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

impl<R, V> ArtifactCacheClientPort for ArtifactCacheService<R, V>
where
    R: ArtifactCacheRepositoryPort + Send,
    V: ArtifactSourceValidationPort,
{
    fn acquire_source_lock(&mut self, source_key: &str) -> Result<(), String> {
        self.repository.acquire_source_lock(source_key).map_err(debug_error)
    }

    fn release_source_lock(&mut self, source_key: &str) -> Result<(), String> {
        self.repository.release_source_lock(source_key).map_err(debug_error)
    }

    fn restore(&mut self, request: &ArtifactCacheRequest, destination: &Path) -> Result<Option<ArtifactCacheRecord>, String> {
        let Some(mut record) = self.repository.find_by_source(&request.source_key).map_err(debug_error)? else { return Ok(None); };
        if record.source_url != request.source_url || record.immutable != request.immutable {
            return Ok(None);
        }
        if !self.prepare_mutable_restore(&mut record)? {
            return Ok(None);
        }
        if !self.repository.restore_file(&record, destination, self.verify_on_hit).map_err(debug_error)? {
            self.repository.remove(&record).map_err(debug_error)?;
            return Ok(None);
        }
        record.last_access_unix_ms = now_unix_ms();
        self.repository.touch(&record, record.last_access_unix_ms).map_err(debug_error)?;
        Ok(Some(record))
    }

    fn store(&mut self, request: &ArtifactCacheRequest, source_path: &Path) -> Result<ArtifactCacheRecord, String> {
        let record = self.repository.import_file(request, source_path, now_unix_ms()).map_err(debug_error)?;
        let _ = self.clean_to_quota()?;
        Ok(record)
    }
}

fn unique_payload_bytes(records: &[ArtifactCacheRecord]) -> u64 {
    let mut seen = std::collections::HashSet::new();
    records.iter().filter_map(|record| {
        if seen.insert(record.sha256.as_str()) { Some(record.size_bytes) } else { None }
    }).sum()
}

fn now_unix_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_millis() as u64).unwrap_or(0)
}

fn debug_error(error: impl std::fmt::Debug) -> String { format!("{error:?}") }
