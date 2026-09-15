// # 📄 Dosya Yolu: /turkuazvm/crates/guest-catalog/src/ports/installer_media_source_provider_port.rs
// # 📌 Amac: Resmi Linux dagitim indexlerinden concrete ISO kaynagi kesfeden Tool contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.40.0
// # Aciklama: Service katmanini Fedora, Debian, Ubuntu ve Rocky HTTP/index ayrintilarindan ayirir
// # Bagimli Oldugu Katman: Service | Tool

use crate::domain::installer_media_source::{
    InstallerMediaProviderPolicy, InstallerMediaSourceError, InstallerMediaSourceRequest,
    ResolvedInstallerMediaSource,
};

pub trait InstallerMediaSourceProviderPort: Send {
    fn resolve(
        &self,
        request: &InstallerMediaSourceRequest,
        policy: &InstallerMediaProviderPolicy,
    ) -> Result<ResolvedInstallerMediaSource, InstallerMediaSourceError>;
}
