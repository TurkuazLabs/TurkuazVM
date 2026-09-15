// # 📄 Dosya Yolu: /turkuazvm/apps/engine/src/services/guest_catalog_application_service.rs
// # 📌 Amac: Guest Catalog bounded contextini Engine seviyesinde orkestre eder
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: YAML OS template katalog sorgularini application use-case olarak sunar
// # Bagimli Oldugu Katman: Service | Repo

use std::path::PathBuf;

use turkuazvm_guest_catalog::domain::guest_template::GuestTemplate;
use turkuazvm_guest_catalog::services::guest_catalog_service::GuestCatalogService;
use turkuazvm_repositories::repositories::yaml_guest_catalog_repository::YamlGuestCatalogRepository;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuestCatalogApplicationError {
    Catalog(String),
}

type CatalogService = GuestCatalogService<YamlGuestCatalogRepository>;

pub struct GuestCatalogApplicationService {
    service: CatalogService,
}

impl GuestCatalogApplicationService {
    pub fn new(path: PathBuf) -> Self {
        Self {
            service: GuestCatalogService::new(YamlGuestCatalogRepository::new(path)),
        }
    }

    pub fn list(&self) -> Result<Vec<GuestTemplate>, GuestCatalogApplicationError> {
        self.service
            .list()
            .map_err(|error| GuestCatalogApplicationError::Catalog(format!("{error:?}")))
    }

    pub fn get(&self, id: &str) -> Result<GuestTemplate, GuestCatalogApplicationError> {
        self.service
            .get(id)
            .map_err(|error| GuestCatalogApplicationError::Catalog(format!("{error:?}")))
    }
}
