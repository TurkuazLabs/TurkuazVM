// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/ports/vm_repository_port.rs
// # 📌 Amac: VM aggregate persistence adapterleri icin repository contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Service katmanini memory, YAML veya SQLite persistence implementasyonlarindan ayirir
// # Bagimli Oldugu Katman: Service | Repo

use crate::domain::virtual_machine::{VirtualMachine, VmId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmRepositoryError {
    NotFound(VmId),
    AlreadyExists(VmId),
    StorageFailed(String),
}

pub trait VmRepositoryPort {
    fn insert(&mut self, machine: VirtualMachine) -> Result<(), VmRepositoryError>;
    fn get(&self, vm_id: &VmId) -> Result<VirtualMachine, VmRepositoryError>;
    fn list(&self) -> Result<Vec<VirtualMachine>, VmRepositoryError>;
    fn save(&mut self, machine: VirtualMachine) -> Result<(), VmRepositoryError>;
    fn delete(&mut self, vm_id: &VmId) -> Result<(), VmRepositoryError>;
}
