// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/services/vm_configuration_service.rs
// # 📌 Amac: Offline VM icin temel kaynak guncelleme ve kalici silme use-case kurallarini yonetir
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Ad vCPU RAM degisikliklerini stopped durumda, silmeyi stopped veya error durumda fail-closed uygular
// # Bagimli Oldugu Katman: Repo

use crate::commands::delete_vm_command::DeleteVmCommand;
use crate::commands::update_vm_command::UpdateVmCommand;
use crate::domain::virtual_machine::{VirtualMachine, VmDomainError, VmId, VmResourceConfig};
use crate::domain::vm_state::VmState;
use crate::ports::vm_repository_port::{VmRepositoryError, VmRepositoryPort};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmConfigurationServiceError {
    VmDomain(VmDomainError),
    Repository(VmRepositoryError),
    VmMustBeStopped(VmState),
    VmMustBeOffline(VmState),
}

pub struct VmConfigurationService<R>
where
    R: VmRepositoryPort,
{
    repository: R,
}

impl<R> VmConfigurationService<R>
where
    R: VmRepositoryPort,
{
    pub const fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn update_vm(
        &mut self,
        command: UpdateVmCommand,
    ) -> Result<VirtualMachine, VmConfigurationServiceError> {
        let vm_id = VmId::parse(command.vm_id).map_err(VmConfigurationServiceError::VmDomain)?;
        let mut machine = self
            .repository
            .get(&vm_id)
            .map_err(VmConfigurationServiceError::Repository)?;
        Self::require_stopped(&machine)?;
        machine
            .reconfigure(
                command.name,
                VmResourceConfig {
                    vcpu_count: command.vcpu_count,
                    memory_mib: command.memory_mib,
                },
            )
            .map_err(VmConfigurationServiceError::VmDomain)?;
        self.repository
            .save(machine.clone())
            .map_err(VmConfigurationServiceError::Repository)?;
        Ok(machine)
    }

    pub fn delete_vm(&mut self, command: DeleteVmCommand) -> Result<(), VmConfigurationServiceError> {
        let vm_id = VmId::parse(command.vm_id).map_err(VmConfigurationServiceError::VmDomain)?;
        let machine = self
            .repository
            .get(&vm_id)
            .map_err(VmConfigurationServiceError::Repository)?;
        Self::require_offline(&machine)?;
        self.repository
            .delete(&vm_id)
            .map_err(VmConfigurationServiceError::Repository)
    }

    fn require_stopped(machine: &VirtualMachine) -> Result<(), VmConfigurationServiceError> {
        if machine.state() != VmState::Stopped {
            return Err(VmConfigurationServiceError::VmMustBeStopped(machine.state()));
        }
        Ok(())
    }
    fn require_offline(machine: &VirtualMachine) -> Result<(), VmConfigurationServiceError> {
        if !matches!(machine.state(), VmState::Stopped | VmState::Error) {
            return Err(VmConfigurationServiceError::VmMustBeOffline(machine.state()));
        }
        Ok(())
    }

}
