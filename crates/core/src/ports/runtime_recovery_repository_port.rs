// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/ports/runtime_recovery_repository_port.rs
// # 📌 Amac: Runtime journal ve recovery queue persistence contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Service katmanini YAML journal, cursor, quarantine ve queue storage detaylarindan ayirir
// # Bagimli Oldugu Katman: Service | Repo

use crate::domain::runtime_recovery::{RuntimeJournalEvent, RuntimeRecoveryRequest};
use crate::domain::virtual_machine::VmId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeRecoveryRepositoryError {
    Io(String),
    Parse(String),
    Invalid(String),
}

pub trait RuntimeRecoveryRepositoryPort {
    fn append_event(&mut self, event: &RuntimeJournalEvent) -> Result<u64, RuntimeRecoveryRepositoryError>;
    fn pending_requests(&self) -> Result<Vec<RuntimeRecoveryRequest>, RuntimeRecoveryRepositoryError>;
    fn save_request(&mut self, request: &RuntimeRecoveryRequest) -> Result<(), RuntimeRecoveryRepositoryError>;
    fn remove_request(&mut self, vm_id: &VmId) -> Result<(), RuntimeRecoveryRepositoryError>;
}
