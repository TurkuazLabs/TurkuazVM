// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/services/snapshot_service.rs
// # 📌 Amac: Offline VM snapshot create/list/restore/delete use-case'lerini yonetir
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: QCOW2 disk snapshot islemlerini repository metadata ve rollback kurallari ile orkestre eder
// # Bagimli Oldugu Katman: Repo | Tool

use crate::commands::create_snapshot_command::CreateSnapshotCommand;
use crate::commands::delete_snapshot_command::DeleteSnapshotCommand;
use crate::commands::restore_snapshot_command::RestoreSnapshotCommand;
use crate::domain::disk::{DiskFormat, DiskId};
use crate::domain::snapshot::{SnapshotDomainError, SnapshotId, SnapshotRecord};
use crate::domain::virtual_machine::{VirtualMachine, VmDomainError, VmId};
use crate::domain::vm_state::VmState;
use crate::ports::snapshot_port::{SnapshotPort, SnapshotStorageError};
use crate::ports::vm_repository_port::{VmRepositoryError, VmRepositoryPort};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotServiceError {
    VmDomain(VmDomainError),
    SnapshotDomain(SnapshotDomainError),
    Repository(VmRepositoryError),
    Storage(SnapshotStorageError),
    VmMustBeStopped(VmState),
    NoDisks,
    Qcow2Required(DiskId),
    PartialRestore { restored_disks: Vec<DiskId>, detail: String },
    PartialDelete { deleted_disks: Vec<DiskId>, detail: String },
    RepositoryDivergedAfterDelete(String),
}

pub struct SnapshotService<R, S>
where
    R: VmRepositoryPort,
    S: SnapshotPort,
{
    repository: R,
    storage: S,
}

impl<R, S> SnapshotService<R, S>
where
    R: VmRepositoryPort,
    S: SnapshotPort,
{
    pub const fn new(repository: R, storage: S) -> Self {
        Self { repository, storage }
    }

    pub fn create_snapshot(
        &mut self,
        command: CreateSnapshotCommand,
    ) -> Result<SnapshotRecord, SnapshotServiceError> {
        let vm_id = VmId::parse(command.vm_id).map_err(SnapshotServiceError::VmDomain)?;
        let snapshot_id = SnapshotId::parse(command.snapshot_id)
            .map_err(SnapshotServiceError::SnapshotDomain)?;
        let mut machine = self
            .repository
            .get(&vm_id)
            .map_err(SnapshotServiceError::Repository)?;
        Self::require_stopped(&machine)?;
        Self::require_qcow2_disks(&machine)?;

        if machine.snapshot(&snapshot_id).is_ok() {
            return Err(SnapshotServiceError::SnapshotDomain(
                SnapshotDomainError::SnapshotAlreadyExists(snapshot_id),
            ));
        }

        let disk_ids = machine
            .disks()
            .iter()
            .map(|attachment| attachment.image().id().clone())
            .collect::<Vec<_>>();
        let record = SnapshotRecord::create(
            snapshot_id.clone(),
            command.name,
            command.created_at_unix_ms,
            disk_ids,
        )
        .map_err(SnapshotServiceError::SnapshotDomain)?;

        let mut created = Vec::new();
        for attachment in machine.disks() {
            if let Err(error) = self.storage.create_snapshot(
                &vm_id,
                attachment.image(),
                &snapshot_id,
            ) {
                for disk_id in created.iter().rev() {
                    if let Ok(attachment) = machine.disk(disk_id) {
                        let _ = self.storage.delete_snapshot(
                            &vm_id,
                            attachment.image(),
                            &snapshot_id,
                        );
                    }
                }
                return Err(SnapshotServiceError::Storage(error));
            }
            created.push(attachment.image().id().clone());
        }

        machine
            .add_snapshot(record.clone())
            .map_err(SnapshotServiceError::SnapshotDomain)?;
        let rollback_disks = machine.disks().to_vec();
        if let Err(error) = self.repository.save(machine) {
            for attachment in rollback_disks {
                let _ = self
                    .storage
                    .delete_snapshot(&vm_id, attachment.image(), &snapshot_id);
            }
            return Err(SnapshotServiceError::Repository(error));
        }

        Ok(record)
    }

    pub fn list_snapshots(&self, vm_id: &str) -> Result<Vec<SnapshotRecord>, SnapshotServiceError> {
        let vm_id = VmId::parse(vm_id.to_owned()).map_err(SnapshotServiceError::VmDomain)?;
        let machine = self
            .repository
            .get(&vm_id)
            .map_err(SnapshotServiceError::Repository)?;
        Ok(machine.snapshots().to_vec())
    }

    pub fn restore_snapshot(
        &mut self,
        command: RestoreSnapshotCommand,
    ) -> Result<SnapshotRecord, SnapshotServiceError> {
        let vm_id = VmId::parse(command.vm_id).map_err(SnapshotServiceError::VmDomain)?;
        let snapshot_id = SnapshotId::parse(command.snapshot_id)
            .map_err(SnapshotServiceError::SnapshotDomain)?;
        let machine = self
            .repository
            .get(&vm_id)
            .map_err(SnapshotServiceError::Repository)?;
        Self::require_stopped(&machine)?;
        let record = machine
            .snapshot(&snapshot_id)
            .map_err(SnapshotServiceError::SnapshotDomain)?
            .clone();

        let mut restored = Vec::new();
        for disk_id in record.disk_ids() {
            let attachment = machine
                .disk(disk_id)
                .map_err(SnapshotServiceError::VmDomain)?;
            if let Err(error) = self.storage.restore_snapshot(
                &vm_id,
                attachment.image(),
                &snapshot_id,
            ) {
                return Err(SnapshotServiceError::PartialRestore {
                    restored_disks: restored,
                    detail: format!("{error:?}"),
                });
            }
            restored.push(disk_id.clone());
        }

        Ok(record)
    }

    pub fn delete_snapshot(
        &mut self,
        command: DeleteSnapshotCommand,
    ) -> Result<(), SnapshotServiceError> {
        let vm_id = VmId::parse(command.vm_id).map_err(SnapshotServiceError::VmDomain)?;
        let snapshot_id = SnapshotId::parse(command.snapshot_id)
            .map_err(SnapshotServiceError::SnapshotDomain)?;
        let mut machine = self
            .repository
            .get(&vm_id)
            .map_err(SnapshotServiceError::Repository)?;
        Self::require_stopped(&machine)?;
        let record = machine
            .snapshot(&snapshot_id)
            .map_err(SnapshotServiceError::SnapshotDomain)?
            .clone();

        let mut deleted = Vec::new();
        for disk_id in record.disk_ids() {
            let attachment = machine
                .disk(disk_id)
                .map_err(SnapshotServiceError::VmDomain)?;
            if let Err(error) = self.storage.delete_snapshot(
                &vm_id,
                attachment.image(),
                &snapshot_id,
            ) {
                return Err(SnapshotServiceError::PartialDelete {
                    deleted_disks: deleted,
                    detail: format!("{error:?}"),
                });
            }
            deleted.push(disk_id.clone());
        }

        machine
            .remove_snapshot(&snapshot_id)
            .map_err(SnapshotServiceError::SnapshotDomain)?;
        self.repository.save(machine).map_err(|error| {
            SnapshotServiceError::RepositoryDivergedAfterDelete(format!("{error:?}"))
        })
    }

    fn require_stopped(machine: &VirtualMachine) -> Result<(), SnapshotServiceError> {
        if machine.state() != VmState::Stopped {
            return Err(SnapshotServiceError::VmMustBeStopped(machine.state()));
        }
        Ok(())
    }

    fn require_qcow2_disks(machine: &VirtualMachine) -> Result<(), SnapshotServiceError> {
        if machine.disks().is_empty() {
            return Err(SnapshotServiceError::NoDisks);
        }
        for attachment in machine.disks() {
            if attachment.image().format() != DiskFormat::Qcow2 {
                return Err(SnapshotServiceError::Qcow2Required(
                    attachment.image().id().clone(),
                ));
            }
        }
        Ok(())
    }
}
