// # 📄 Dosya Yolu: /turkuazvm/crates/artifact-cache/src/domain/artifact_cache.rs
// # 📌 Amac: Artifact Cache kaynak, validator, record, stats ve revalidation modellerini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: SHA-256, ETag, Last-Modified, pinned, immutable/mutable ve LRU metadata kurallarini typed olarak tasir
// # Bagimli Oldugu Katman: Service | Repo | Tool

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ArtifactSourceValidators {
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

impl ArtifactSourceValidators {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.etag.is_none() && self.last_modified.is_none()
    }

    #[must_use]
    pub fn merge(&self, newer: &Self) -> Self {
        Self {
            etag: newer.etag.clone().or_else(|| self.etag.clone()),
            last_modified: newer.last_modified.clone().or_else(|| self.last_modified.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactCacheRequest {
    pub source_key: String,
    pub source_url: String,
    pub immutable: bool,
    pub pinned: bool,
    pub validators: ArtifactSourceValidators,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactCacheRecord {
    pub source_key: String,
    pub source_url: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub immutable: bool,
    pub pinned: bool,
    pub validators: ArtifactSourceValidators,
    pub created_at_unix_ms: u64,
    pub last_access_unix_ms: u64,
    pub last_revalidated_unix_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArtifactCacheStats {
    pub artifact_count: usize,
    pub pinned_count: usize,
    pub mutable_count: usize,
    pub used_bytes: u64,
    pub quota_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArtifactCacheVerification {
    pub checked: usize,
    pub valid: usize,
    pub invalid: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactCacheRevalidationState {
    Immutable,
    NotModified,
    RemoteModified,
    ValidationFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactCacheRevalidation {
    pub source_key: String,
    pub state: ArtifactCacheRevalidationState,
    pub validators: ArtifactSourceValidators,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArtifactCacheRevalidationSummary {
    pub checked: usize,
    pub immutable: usize,
    pub not_modified: usize,
    pub remote_modified: usize,
    pub failed: usize,
}
