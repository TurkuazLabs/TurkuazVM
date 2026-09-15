// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/services/storage_service.rs
// # 📌 Amac: Sanal disk create, attach ve resize use-case is kurallarini yonetir
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: VM state, repository ve StoragePort islemlerini offline disk guvenligi ile orkestre eder
// # Bagimli Oldugu Katman: Repo | Tool

use crate::commands::create_disk_command::CreateDiskCommand;
use crate::commands::delete_disk_command::DeleteDiskCommand;
use crate::commands::resize_disk_command::ResizeDiskCommand;
use crate::domain::disk::{DiskAttachment, DiskDomainError, DiskId, DiskImage};
use crate::domain::virtual_machine::{VirtualMachine, VmDomainError, VmId};
use crate::domain::vm_state::VmState;
use crate::ports::storage_port::{StorageError, StoragePort};
use crate::ports::vm_repository_port::{VmRepositoryError, VmRepositoryPort};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageServiceError {
    VmDomain(VmDomainError),
    DiskDomain(DiskDomainError),
    Repository(VmRepositoryError),
    Storage(StorageError),
    VmMustBeStopped(VmState),
    RepositoryDivergedAfterStorageMutation(String),
}

pub struct StorageService<R, S>
where
    R: VmRepositoryPort,
    S: StoragePort,
{
    repository: R,
    storage: S,
}

impl<R, S> StorageService<R, S>
where
    R: VmRepositoryPort,
    S: StoragePort,
{
    pub const fn new(repository: R, storage: S) -> Self {
        Self { repository, storage }
    }

    pub fn create_and_attach_disk(
        &mut self,
        command: CreateDiskCommand,
    ) -> Result<VirtualMachine, StorageServiceError> {
        let vm_id = VmId::parse(command.vm_id).map_err(StorageServiceError::VmDomain)?;
        let mut machine = self
            .repository
            .get(&vm_id)
            .map_err(StorageServiceError::Repository)?;
        Self::require_stopped(&machine)?;

        let disk_id = DiskId::parse(command.disk_id).map_err(StorageServiceError::DiskDomain)?;
        let image = DiskImage::create(
            disk_id,
            command.format,
            command.virtual_size_bytes,
            command.relative_path,
        )
        .map_err(StorageServiceError::DiskDomain)?;

        self.storage
            .create_image(&vm_id, &image)
            .map_err(StorageServiceError::Storage)?;

        let attachment = DiskAttachment::new(image.clone(), command.bus, command.boot_index);
        if let Err(error) = machine.attach_disk(attachment) {
            let _ = self.storage.delete_image(&vm_id, &image);
            return Err(StorageServiceError::VmDomain(error));
        }

        if let Err(error) = self.repository.save(machine.clone()) {
            let _ = self.storage.delete_image(&vm_id, &image);
            return Err(StorageServiceError::Repository(error));
        }

        Ok(machine)
    }

    pub fn delete_disk(
        &mut self,
        command: DeleteDiskCommand,
    ) -> Result<VirtualMachine, StorageServiceError> {
        let vm_id = VmId::parse(command.vm_id).map_err(StorageServiceError::VmDomain)?;
        let disk_id = DiskId::parse(command.disk_id).map_err(StorageServiceError::DiskDomain)?;
        let mut machine = self
            .repository
            .get(&vm_id)
            .map_err(StorageServiceError::Repository)?;
        Self::require_stopped(&machine)?;

        let attachment = machine
            .detach_disk(&disk_id)
            .map_err(StorageServiceError::VmDomain)?;
        self.storage
            .delete_image(&vm_id, attachment.image())
            .map_err(StorageServiceError::Storage)?;

        self.repository
            .save(machine.clone())
            .map_err(|error| StorageServiceError::RepositoryDivergedAfterStorageMutation(format!("{error:?}")))?;
        Ok(machine)
    }

    pub fn resize_disk(
        &mut self,
        command: ResizeDiskCommand,
    ) -> Result<VirtualMachine, StorageServiceError> {
        let vm_id = VmId::parse(command.vm_id).map_err(StorageServiceError::VmDomain)?;
        let disk_id = DiskId::parse(command.disk_id).map_err(StorageServiceError::DiskDomain)?;
        let mut machine = self
            .repository
            .get(&vm_id)
            .map_err(StorageServiceError::Repository)?;
        Self::require_stopped(&machine)?;

        let image = machine
            .disk(&disk_id)
            .map_err(StorageServiceError::VmDomain)?
            .image()
            .clone();

        if command.new_virtual_size_bytes <= image.virtual_size_bytes() {
            return Err(StorageServiceError::DiskDomain(DiskDomainError::ResizeMustGrow));
        }

        let info = self
            .storage
            .resize_image(&vm_id, &image, command.new_virtual_size_bytes)
            .map_err(StorageServiceError::Storage)?;
        machine
            .grow_disk(&disk_id, info.virtual_size_bytes)
            .map_err(StorageServiceError::VmDomain)?;

        self.repository.save(machine.clone()).map_err(|error| {
            StorageServiceError::RepositoryDivergedAfterStorageMutation(format!("{error:?}"))
        })?;

        Ok(machine)
    }

    pub fn inspect_disk(
        &self,
        vm_id: &VmId,
        disk_id: &DiskId,
    ) -> Result<crate::ports::storage_port::StorageImageInfo, StorageServiceError> {
        let machine = self
            .repository
            .get(vm_id)
            .map_err(StorageServiceError::Repository)?;
        Self::require_stopped(&machine)?;
        let image = machine
            .disk(disk_id)
            .map_err(StorageServiceError::VmDomain)?
            .image();
        self.storage
            .inspect_image(vm_id, image)
            .map_err(StorageServiceError::Storage)
    }

    pub fn into_parts(self) -> (R, S) {
        (self.repository, self.storage)
    }

    fn require_stopped(machine: &VirtualMachine) -> Result<(), StorageServiceError> {
        if machine.state() != VmState::Stopped {
            return Err(StorageServiceError::VmMustBeStopped(machine.state()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use super::*;
    use crate::domain::disk::{DiskBus, DiskFormat};
    use crate::domain::hypervisor::AccelerationBackend;
    use crate::domain::virtual_machine::VmResourceConfig;
    use crate::ports::storage_port::StorageImageInfo;

    #[derive(Default)]
    struct FakeRepository {
        machines: HashMap<VmId, VirtualMachine>,
    }

    impl VmRepositoryPort for FakeRepository {
        fn insert(&mut self, machine: VirtualMachine) -> Result<(), VmRepositoryError> {
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

    #[derive(Default)]
    struct FakeStorage {
        sizes: RefCell<HashMap<String, u64>>,
    }

    impl StoragePort for FakeStorage {
        fn create_image(&self, _vm_id: &VmId, image: &DiskImage) -> Result<StorageImageInfo, StorageError> {
            self.sizes
                .borrow_mut()
                .insert(image.relative_path().to_owned(), image.virtual_size_bytes());
            Ok(StorageImageInfo {
                format: image.format(),
                virtual_size_bytes: image.virtual_size_bytes(),
                actual_size_bytes: Some(0),
            })
        }

        fn inspect_image(&self, _vm_id: &VmId, image: &DiskImage) -> Result<StorageImageInfo, StorageError> {
            let size = self
                .sizes
                .borrow()
                .get(image.relative_path())
                .copied()
                .ok_or_else(|| StorageError::NotFound(image.relative_path().to_owned()))?;
            Ok(StorageImageInfo {
                format: image.format(),
                virtual_size_bytes: size,
                actual_size_bytes: Some(0),
            })
        }

        fn resize_image(
            &self,
            _vm_id: &VmId,
            image: &DiskImage,
            new_virtual_size_bytes: u64,
        ) -> Result<StorageImageInfo, StorageError> {
            self.sizes
                .borrow_mut()
                .insert(image.relative_path().to_owned(), new_virtual_size_bytes);
            Ok(StorageImageInfo {
                format: image.format(),
                virtual_size_bytes: new_virtual_size_bytes,
                actual_size_bytes: Some(0),
            })
        }

        fn delete_image(&self, _vm_id: &VmId, image: &DiskImage) -> Result<(), StorageError> {
            self.sizes.borrow_mut().remove(image.relative_path());
            Ok(())
        }
    }

    fn stopped_machine() -> VirtualMachine {
        let mut machine = VirtualMachine::create(
            VmId::parse("storage-test").expect("vm id must be valid"),
            "Storage Test",
            VmResourceConfig {
                vcpu_count: 2,
                memory_mib: 2048,
            },
            AccelerationBackend::Tcg,
        )
        .expect("vm must be valid");
        machine
            .transition_to(VmState::Stopped)
            .expect("created vm must stop");
        machine
    }

    #[test]
    fn create_disk_attaches_to_stopped_vm() {
        let mut repository = FakeRepository::default();
        repository
            .insert(stopped_machine())
            .expect("insert must succeed");
        let storage = FakeStorage::default();
        let mut service = StorageService::new(repository, storage);

        let machine = service
            .create_and_attach_disk(CreateDiskCommand::new(
                "storage-test",
                "system",
                DiskFormat::Qcow2,
                1024,
                "disks/system.qcow2",
                DiskBus::Virtio,
                Some(1),
            ))
            .expect("disk create must succeed");

        assert_eq!(machine.disks().len(), 1);
    }

    #[test]
    fn resize_rejects_shrink() {
        let mut repository = FakeRepository::default();
        repository
            .insert(stopped_machine())
            .expect("insert must succeed");
        let storage = FakeStorage::default();
        let mut service = StorageService::new(repository, storage);
        service
            .create_and_attach_disk(CreateDiskCommand::new(
                "storage-test",
                "system",
                DiskFormat::Qcow2,
                1024,
                "disks/system.qcow2",
                DiskBus::Virtio,
                Some(1),
            ))
            .expect("disk create must succeed");

        let result = service.resize_disk(ResizeDiskCommand::new("storage-test", "system", 512));
        assert_eq!(
            result,
            Err(StorageServiceError::DiskDomain(DiskDomainError::ResizeMustGrow))
        );
    }
}
