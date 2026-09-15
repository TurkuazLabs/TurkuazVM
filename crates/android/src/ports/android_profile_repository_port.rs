// # 📄 Dosya Yolu: /turkuazvm/crates/android/src/ports/android_profile_repository_port.rs
// # 📌 Amac: Android runtime profile persistence contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Service katmanini YAML veya baska storage implementasyonlarindan ayirir
// # Bagimli Oldugu Katman: Service | Repo

use crate::domain::runtime_profile::{AndroidRuntimeProfile, AndroidVmId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndroidProfileRepositoryError {
    NotFound(AndroidVmId),
    Storage(String),
}

pub trait AndroidProfileRepositoryPort {
    fn get(&self, vm_id: &AndroidVmId) -> Result<AndroidRuntimeProfile, AndroidProfileRepositoryError>;
    fn save(&mut self, profile: AndroidRuntimeProfile) -> Result<(), AndroidProfileRepositoryError>;
    fn delete(&mut self, vm_id: &AndroidVmId) -> Result<(), AndroidProfileRepositoryError>;
    fn list(&self) -> Result<Vec<AndroidRuntimeProfile>, AndroidProfileRepositoryError>;
}
