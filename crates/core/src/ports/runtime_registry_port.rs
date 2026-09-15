// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/ports/runtime_registry_port.rs
// # 📌 Amac: Kalici QEMU runtime registration storage adapter contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: QEMU Tool katmanini YAML/dosya persistence detayindan ayirir ve restart reattach state'ini Repo katmanina tasir
// # Bagimli Oldugu Katman: Repo | Tool

use crate::domain::runtime_registration::RuntimeRegistration;
use crate::domain::virtual_machine::VmId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeRegistryError {
    Io(String),
    Parse(String),
    Invalid(String),
}

pub trait RuntimeRegistryPort {
    fn list(&self) -> Result<Vec<RuntimeRegistration>, RuntimeRegistryError>;
    fn save(&self, registration: &RuntimeRegistration) -> Result<(), RuntimeRegistryError>;
    fn remove(&self, vm_id: &VmId) -> Result<(), RuntimeRegistryError>;
}
