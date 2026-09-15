// # 📄 Dosya Yolu: /turkuazvm/crates/artifact-cache/src/ports/artifact_cache_repository_port.rs
// # 📌 Amac: Artifact Cache storage adapter contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Content-addressed payload, source index, restore, verification ve silme I/O detaylarini Service'ten ayirir
// # Bagimli Oldugu Katman: Service | Repo

use std::path::Path;

use crate::domain::artifact_cache::{ArtifactCacheRecord, ArtifactCacheRequest};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactCacheRepositoryError {
    Io(String),
    Parse(String),
    Invalid(String),
    UnsupportedFutureSchema(u16),
    Lock(String),
}

pub trait ArtifactCacheRepositoryPort {
    fn acquire_source_lock(&mut self, source_key: &str) -> Result<(), ArtifactCacheRepositoryError>;
    fn release_source_lock(&mut self, source_key: &str) -> Result<(), ArtifactCacheRepositoryError>;
    fn find_by_source(&self, source_key: &str) -> Result<Option<ArtifactCacheRecord>, ArtifactCacheRepositoryError>;
    fn list(&self) -> Result<Vec<ArtifactCacheRecord>, ArtifactCacheRepositoryError>;
    fn import_file(
        &mut self,
        request: &ArtifactCacheRequest,
        source_path: &Path,
        now_unix_ms: u64,
    ) -> Result<ArtifactCacheRecord, ArtifactCacheRepositoryError>;
    fn restore_file(
        &self,
        record: &ArtifactCacheRecord,
        destination: &Path,
        full_hash_verify: bool,
    ) -> Result<bool, ArtifactCacheRepositoryError>;
    fn touch(&mut self, record: &ArtifactCacheRecord, now_unix_ms: u64) -> Result<(), ArtifactCacheRepositoryError>;
    fn save_record(&mut self, record: &ArtifactCacheRecord) -> Result<(), ArtifactCacheRepositoryError>;
    fn remove(&mut self, record: &ArtifactCacheRecord) -> Result<(), ArtifactCacheRepositoryError>;
    fn verify(&self, record: &ArtifactCacheRecord) -> Result<bool, ArtifactCacheRepositoryError>;
}
