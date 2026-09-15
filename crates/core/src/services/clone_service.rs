// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/services/clone_service.rs
// # 📌 Amac: Full ve linked VM clone use-case'lerini transaction kurallari ile yonetir
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Disk clone, guest asset copy, yeni VM aggregate ve cleanup compensation islemlerini orkestre eder
// # Bagimli Oldugu Katman: Repo | Tool

use crate::commands::clone_vm_command::CloneVmCommand;
use crate::domain::clone::CloneMode;
use crate::domain::disk::{DiskAttachment, DiskFormat, DiskId, DiskImage};
use crate::domain::virtual_machine::{VirtualMachine, VmDomainError, VmId};
use crate::domain::vm_state::VmState;
use crate::ports::clone_storage_port::{CloneStorageError, CloneStoragePort};
use crate::ports::vm_repository_port::{VmRepositoryError, VmRepositoryPort};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloneServiceError {
    VmDomain(VmDomainError),
    Repository(VmRepositoryError),
    Storage(CloneStorageError),
    SourceMustBeStopped(VmState),
    SourceEqualsTarget,
    LinkedCloneRequiresQcow2(DiskId),
    CleanupFailed(String),
}

pub struct CloneService<R, C>
where
    R: VmRepositoryPort,
    C: CloneStoragePort,
{
    repository: R,
    storage: C,
}

impl<R, C> CloneService<R, C>
where
    R: VmRepositoryPort,
    C: CloneStoragePort,
{
    pub const fn new(repository: R, storage: C) -> Self {
        Self { repository, storage }
    }

    pub fn clone_vm(
        &mut self,
        command: CloneVmCommand,
    ) -> Result<VirtualMachine, CloneServiceError> {
        let source_id = VmId::parse(command.source_vm_id).map_err(CloneServiceError::VmDomain)?;
        let target_id = VmId::parse(command.target_vm_id).map_err(CloneServiceError::VmDomain)?;
        if source_id == target_id {
            return Err(CloneServiceError::SourceEqualsTarget);
        }

        let source = self
            .repository
            .get(&source_id)
            .map_err(CloneServiceError::Repository)?;
        if source.state() != VmState::Stopped {
            return Err(CloneServiceError::SourceMustBeStopped(source.state()));
        }
        match self.repository.get(&target_id) {
            Ok(_) => return Err(CloneServiceError::Repository(VmRepositoryError::AlreadyExists(target_id))),
            Err(VmRepositoryError::NotFound(_)) => {}
            Err(error) => return Err(CloneServiceError::Repository(error)),
        }

        if command.mode == CloneMode::Linked {
            for attachment in source.disks() {
                if attachment.image().format() != DiskFormat::Qcow2 {
                    return Err(CloneServiceError::LinkedCloneRequiresQcow2(
                        attachment.image().id().clone(),
                    ));
                }
            }
        }

        self.storage
            .prepare_clone_target(&target_id)
            .map_err(CloneServiceError::Storage)?;

        let mut cloned_disks = Vec::with_capacity(source.disks().len());
        for attachment in source.disks() {
            let image = DiskImage::create(
                attachment.image().id().clone(),
                attachment.image().format(),
                attachment.image().virtual_size_bytes(),
                attachment.image().relative_path().to_owned(),
            )
            .map_err(|error| CloneServiceError::VmDomain(VmDomainError::Disk(error)))?;

            if let Err(error) = self.storage.clone_image(
                &source_id,
                attachment.image(),
                &target_id,
                &image,
                command.mode,
            ) {
                return self.fail_with_cleanup(&target_id, CloneServiceError::Storage(error));
            }
            cloned_disks.push(DiskAttachment::new(
                image,
                attachment.bus(),
                attachment.boot_index(),
            ));
        }

        if let Err(error) = self.storage.clone_machine_assets(&source_id, &target_id) {
            return self.fail_with_cleanup(&target_id, CloneServiceError::Storage(error));
        }

        let target = match VirtualMachine::reconstitute(
            target_id.clone(),
            command.target_name,
            source.resources().clone(),
            source.acceleration(),
            VmState::Stopped,
            None,
            source.guest_boot().clone(),
            cloned_disks,
            Vec::new(),
            Vec::new(),
        ) {
            Ok(target) => target,
            Err(error) => {
                return self.fail_with_cleanup(
                    &target_id,
                    CloneServiceError::VmDomain(error),
                );
            }
        };

        if let Err(error) = self.repository.insert(target.clone()) {
            return self.fail_with_cleanup(&target_id, CloneServiceError::Repository(error));
        }

        Ok(target)
    }

    fn fail_with_cleanup<T>(
        &self,
        target_id: &VmId,
        original: CloneServiceError,
    ) -> Result<T, CloneServiceError> {
        match self.storage.cleanup_clone(target_id) {
            Ok(()) => Err(original),
            Err(cleanup) => Err(CloneServiceError::CleanupFailed(format!(
                "original={original:?}; cleanup={cleanup:?}"
            ))),
        }
    }
}
