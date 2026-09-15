// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/services/vm_query_service.rs
// # 📌 Amac: VM listeleme ve tek VM okuma use-case'lerini repository portu uzerinden sunar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: UI ve API katmanlarini repository implementasyonundan ayirir
// # Bagimli Oldugu Katman: Repo

use crate::domain::virtual_machine::{VirtualMachine, VmDomainError, VmId};
use crate::ports::vm_repository_port::{VmRepositoryError, VmRepositoryPort};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmQueryError { Domain(VmDomainError), Repository(VmRepositoryError) }

pub struct VmQueryService<R>
where R: VmRepositoryPort,
{ repository: R }

impl<R> VmQueryService<R>
where R: VmRepositoryPort,
{
    pub const fn new(repository: R) -> Self { Self { repository } }
    pub fn list(&self) -> Result<Vec<VirtualMachine>, VmQueryError> { self.repository.list().map_err(VmQueryError::Repository) }
    pub fn get(&self, vm_id: impl Into<String>) -> Result<VirtualMachine, VmQueryError> {
        let id = VmId::parse(vm_id).map_err(VmQueryError::Domain)?;
        self.repository.get(&id).map_err(VmQueryError::Repository)
    }
}
