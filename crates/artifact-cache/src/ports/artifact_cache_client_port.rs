// # 📄 Dosya Yolu: /turkuazvm/crates/artifact-cache/src/ports/artifact_cache_client_port.rs
// # 📌 Amac: Downloader Tool'larinin merkezi cache'i repository detaylarini bilmeden kullanmasini saglar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Restore/store islemlerini minimal Tool-facing port olarak tanimlar
// # Bagimli Oldugu Katman: Service | Tool

use std::path::Path;

use crate::domain::artifact_cache::{ArtifactCacheRecord, ArtifactCacheRequest};

pub trait ArtifactCacheClientPort: Send {
    fn acquire_source_lock(&mut self, source_key: &str) -> Result<(), String>;
    fn release_source_lock(&mut self, source_key: &str) -> Result<(), String>;
    fn restore(&mut self, request: &ArtifactCacheRequest, destination: &Path) -> Result<Option<ArtifactCacheRecord>, String>;
    fn store(&mut self, request: &ArtifactCacheRequest, source_path: &Path) -> Result<ArtifactCacheRecord, String>;
}
