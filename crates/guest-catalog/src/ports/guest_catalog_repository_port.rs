// # 📄 Dosya Yolu: /turkuazvm/crates/guest-catalog/src/ports/guest_catalog_repository_port.rs
// # 📌 Amac: Guest Catalog persistence adapter contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.39.4
// # Aciklama: Guest template ve Linux installer medya politika storage contractlarini Service katmanindan ayirir
// # Bagimli Oldugu Katman: Repo

use crate::domain::guest_template::GuestTemplate;
use crate::domain::linux_media_policy::LinuxMediaPolicy;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuestCatalogRepositoryError {
    NotFound(String),
    Storage(String),
}

pub trait GuestCatalogRepositoryPort {
    fn list(&self) -> Result<Vec<GuestTemplate>, GuestCatalogRepositoryError>;
    fn get(&self, id: &str) -> Result<GuestTemplate, GuestCatalogRepositoryError>;
    fn linux_media_policies(&self) -> Result<Vec<LinuxMediaPolicy>, GuestCatalogRepositoryError>;
}
