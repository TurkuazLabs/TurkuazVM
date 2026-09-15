// # 📄 Dosya Yolu: /turkuazvm/crates/repositories/src/repositories/in_memory_vm_repository.rs
// # 📌 Amac: VM aggregate icin gecici in-memory repository adapteri saglar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: v0.1.x lifecycle testleri icin HashMap tabanli persistence implementasyonu sunar
// # Bagimli Oldugu Katman: Repo

use std::collections::HashMap;

use turkuazvm_core::domain::virtual_machine::{VirtualMachine, VmId};
use turkuazvm_core::ports::vm_repository_port::{VmRepositoryError, VmRepositoryPort};

#[derive(Default)]
pub struct InMemoryVmRepository {
    machines: HashMap<VmId, VirtualMachine>,
}

impl InMemoryVmRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl VmRepositoryPort for InMemoryVmRepository {
    fn insert(&mut self, machine: VirtualMachine) -> Result<(), VmRepositoryError> {
        if self.machines.contains_key(machine.id()) {
            return Err(VmRepositoryError::AlreadyExists(machine.id().clone()));
        }

        self.machines.insert(machine.id().clone(), machine);
        Ok(())
    }

    fn get(&self, vm_id: &VmId) -> Result<VirtualMachine, VmRepositoryError> {
        self.machines
            .get(vm_id)
            .cloned()
            .ok_or_else(|| VmRepositoryError::NotFound(vm_id.clone()))
    }

    fn list(&self) -> Result<Vec<VirtualMachine>, VmRepositoryError> {
        Ok(self.machines.values().cloned().collect())
    }

    fn save(&mut self, machine: VirtualMachine) -> Result<(), VmRepositoryError> {
        if !self.machines.contains_key(machine.id()) {
            return Err(VmRepositoryError::NotFound(machine.id().clone()));
        }

        self.machines.insert(machine.id().clone(), machine);
        Ok(())
    }

    fn delete(&mut self, vm_id: &VmId) -> Result<(), VmRepositoryError> {
        self.machines
            .remove(vm_id)
            .map(|_| ())
            .ok_or_else(|| VmRepositoryError::NotFound(vm_id.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use turkuazvm_core::domain::hypervisor::AccelerationBackend;
    use turkuazvm_core::domain::virtual_machine::{VmResourceConfig, VmId};

    fn machine() -> VirtualMachine {
        VirtualMachine::create(
            VmId::parse("repo-test").expect("id must be valid"),
            "Repo Test",
            VmResourceConfig {
                vcpu_count: 2,
                memory_mib: 1024,
            },
            AccelerationBackend::Tcg,
        )
        .expect("machine must be valid")
    }

    #[test]
    fn insert_and_get_round_trip() {
        let mut repository = InMemoryVmRepository::new();
        let machine = machine();
        let id = machine.id().clone();
        repository.insert(machine).expect("insert must succeed");

        assert_eq!(repository.get(&id).expect("machine must exist").id(), &id);
    }
}
