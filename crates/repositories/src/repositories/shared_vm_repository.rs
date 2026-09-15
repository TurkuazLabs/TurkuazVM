// # 📄 Dosya Yolu: /turkuazvm/crates/repositories/src/repositories/shared_vm_repository.rs
// # 📌 Amac: Tek repository instance'ini birden fazla application service arasinda guvenli olarak paylastirir
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Arc Mutex ile VmRepositoryPort adapterini clone edilebilir hale getirir
// # Bagimli Oldugu Katman: Service | Repo

use std::sync::{Arc, Mutex};

use turkuazvm_core::domain::virtual_machine::{VirtualMachine, VmId};
use turkuazvm_core::ports::vm_repository_port::{VmRepositoryError, VmRepositoryPort};

pub struct SharedVmRepository<R>
where
    R: VmRepositoryPort,
{
    inner: Arc<Mutex<R>>,
}

impl<R> Clone for SharedVmRepository<R>
where
    R: VmRepositoryPort,
{
    fn clone(&self) -> Self {
        Self { inner: Arc::clone(&self.inner) }
    }
}

impl<R> SharedVmRepository<R>
where
    R: VmRepositoryPort,
{
    pub fn new(repository: R) -> Self {
        Self { inner: Arc::new(Mutex::new(repository)) }
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, R>, VmRepositoryError> {
        self.inner.lock().map_err(|_| VmRepositoryError::StorageFailed(String::from("VM repository lock poisoned")))
    }
}

impl<R> VmRepositoryPort for SharedVmRepository<R>
where
    R: VmRepositoryPort,
{
    fn insert(&mut self, machine: VirtualMachine) -> Result<(), VmRepositoryError> { self.lock()?.insert(machine) }
    fn get(&self, vm_id: &VmId) -> Result<VirtualMachine, VmRepositoryError> { self.lock()?.get(vm_id) }
    fn list(&self) -> Result<Vec<VirtualMachine>, VmRepositoryError> { self.lock()?.list() }
    fn save(&mut self, machine: VirtualMachine) -> Result<(), VmRepositoryError> { self.lock()?.save(machine) }
    fn delete(&mut self, vm_id: &VmId) -> Result<(), VmRepositoryError> { self.lock()?.delete(vm_id) }
}
