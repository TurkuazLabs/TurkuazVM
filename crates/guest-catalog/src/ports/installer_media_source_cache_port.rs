// # 📄 Dosya Yolu: /turkuazvm/crates/guest-catalog/src/ports/installer_media_source_cache_port.rs
// # 📌 Amac: Linux installer medya last-known-good source cache persistence contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.40.0
// # Aciklama: Online discovery gecici basarisiz oldugunda daha once dogrulanmis source kaydinin okunup yazilmasini soyutlar
// # Bagimli Oldugu Katman: Service | Repo

use crate::domain::installer_media_source::{InstallerMediaSourceError, ResolvedInstallerMediaSource};

pub trait InstallerMediaSourceCachePort: Send {
    fn load(&self, cache_key: &str) -> Result<Option<ResolvedInstallerMediaSource>, InstallerMediaSourceError>;
    fn save(&self, source: &ResolvedInstallerMediaSource) -> Result<(), InstallerMediaSourceError>;
}
