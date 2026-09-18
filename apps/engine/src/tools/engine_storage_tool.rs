// # 📄 Dosya Yolu: /turkuazvm/apps/engine/src/tools/engine_storage_tool.rs
// # 📌 Amac: qemu-img bulunurlugunu snapshot ve clone portlari arkasinda normalize eder
// # 📌 Modul - Rust
// # Version: 0.41.6
// # Aciklama: qemu-img adapterine metadata data_root ve buyuk disk image_root koklerini ayri aktarir; unavailable davranisini korur
// # Bagimli Oldugu Katman: Service | Tool

use std::path::PathBuf;

use turkuazvm_core::domain::clone::CloneMode;
use turkuazvm_core::domain::disk::DiskImage;
use turkuazvm_core::domain::snapshot::SnapshotId;
use turkuazvm_core::domain::virtual_machine::VmId;
use turkuazvm_core::ports::clone_storage_port::{CloneStorageError, CloneStoragePort};
use turkuazvm_core::ports::snapshot_port::{SnapshotPort, SnapshotStorageError};
use turkuazvm_core::ports::storage_port::{StorageError, StorageImageInfo, StoragePort};
use turkuazvm_storage::tools::qemu_img_tool::QemuImgTool;

#[derive(Clone)]
pub enum EngineStorageTool {
    QemuImg(QemuImgTool),
    Unavailable,
}

impl EngineStorageTool {
    pub fn new(binary: Option<PathBuf>, data_root: PathBuf, image_root: PathBuf) -> Self {
        match binary {
            Some(binary) => Self::QemuImg(QemuImgTool::new(binary, data_root, image_root)),
            None => Self::Unavailable,
        }
    }

    fn unavailable_snapshot() -> SnapshotStorageError {
        SnapshotStorageError::ExecutionFailed(String::from("qemu-img is unavailable"))
    }

    fn unavailable_clone() -> CloneStorageError {
        CloneStorageError::ExecutionFailed(String::from("qemu-img is unavailable"))
    }

    fn unavailable_storage() -> StorageError {
        StorageError::ExecutionFailed(String::from("qemu-img is unavailable"))
    }
}


impl StoragePort for EngineStorageTool {
    fn create_image(&self, vm_id: &VmId, image: &DiskImage) -> Result<StorageImageInfo, StorageError> {
        match self {
            Self::QemuImg(tool) => tool.create_image(vm_id, image),
            Self::Unavailable => Err(Self::unavailable_storage()),
        }
    }

    fn inspect_image(&self, vm_id: &VmId, image: &DiskImage) -> Result<StorageImageInfo, StorageError> {
        match self {
            Self::QemuImg(tool) => tool.inspect_image(vm_id, image),
            Self::Unavailable => Err(Self::unavailable_storage()),
        }
    }

    fn resize_image(
        &self,
        vm_id: &VmId,
        image: &DiskImage,
        new_virtual_size_bytes: u64,
    ) -> Result<StorageImageInfo, StorageError> {
        match self {
            Self::QemuImg(tool) => tool.resize_image(vm_id, image, new_virtual_size_bytes),
            Self::Unavailable => Err(Self::unavailable_storage()),
        }
    }

    fn delete_image(&self, vm_id: &VmId, image: &DiskImage) -> Result<(), StorageError> {
        match self {
            Self::QemuImg(tool) => tool.delete_image(vm_id, image),
            Self::Unavailable => Err(Self::unavailable_storage()),
        }
    }
}

impl SnapshotPort for EngineStorageTool {
    fn create_snapshot(
        &self,
        vm_id: &VmId,
        image: &DiskImage,
        snapshot_id: &SnapshotId,
    ) -> Result<(), SnapshotStorageError> {
        match self {
            Self::QemuImg(tool) => tool.create_snapshot(vm_id, image, snapshot_id),
            Self::Unavailable => Err(Self::unavailable_snapshot()),
        }
    }

    fn restore_snapshot(
        &self,
        vm_id: &VmId,
        image: &DiskImage,
        snapshot_id: &SnapshotId,
    ) -> Result<(), SnapshotStorageError> {
        match self {
            Self::QemuImg(tool) => tool.restore_snapshot(vm_id, image, snapshot_id),
            Self::Unavailable => Err(Self::unavailable_snapshot()),
        }
    }

    fn delete_snapshot(
        &self,
        vm_id: &VmId,
        image: &DiskImage,
        snapshot_id: &SnapshotId,
    ) -> Result<(), SnapshotStorageError> {
        match self {
            Self::QemuImg(tool) => tool.delete_snapshot(vm_id, image, snapshot_id),
            Self::Unavailable => Err(Self::unavailable_snapshot()),
        }
    }
}

impl CloneStoragePort for EngineStorageTool {
    fn prepare_clone_target(&self, target_vm_id: &VmId) -> Result<(), CloneStorageError> {
        match self {
            Self::QemuImg(tool) => tool.prepare_clone_target(target_vm_id),
            Self::Unavailable => Err(Self::unavailable_clone()),
        }
    }

    fn clone_image(
        &self,
        source_vm_id: &VmId,
        source: &DiskImage,
        target_vm_id: &VmId,
        target: &DiskImage,
        mode: CloneMode,
    ) -> Result<StorageImageInfo, CloneStorageError> {
        match self {
            Self::QemuImg(tool) => {
                tool.clone_image(source_vm_id, source, target_vm_id, target, mode)
            }
            Self::Unavailable => Err(Self::unavailable_clone()),
        }
    }

    fn clone_machine_assets(
        &self,
        source_vm_id: &VmId,
        target_vm_id: &VmId,
    ) -> Result<(), CloneStorageError> {
        match self {
            Self::QemuImg(tool) => tool.clone_machine_assets(source_vm_id, target_vm_id),
            Self::Unavailable => Err(Self::unavailable_clone()),
        }
    }

    fn cleanup_clone(&self, target_vm_id: &VmId) -> Result<(), CloneStorageError> {
        match self {
            Self::QemuImg(tool) => tool.cleanup_clone(target_vm_id),
            Self::Unavailable => Err(Self::unavailable_clone()),
        }
    }
}
