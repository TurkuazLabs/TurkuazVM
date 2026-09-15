// # 📄 Dosya Yolu: /turkuazvm/crates/guest-catalog/src/ports/installer_media_source_resolver_port.rs
// # 📌 Amac: Engine download Service icin Linux installer medya source resolution contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.40.0
// # Aciklama: Online discovery, cache ve katalog fallback kararini application katmanindan ayirir
// # Bagimli Oldugu Katman: Service

use crate::domain::installer_media_source::{
    InstallerMediaSourceError, InstallerMediaSourceRequest, ResolvedInstallerMediaSource,
};

pub trait InstallerMediaSourceResolverPort: Send {
    fn resolve(
        &mut self,
        request: &InstallerMediaSourceRequest,
    ) -> Result<ResolvedInstallerMediaSource, InstallerMediaSourceError>;
}
