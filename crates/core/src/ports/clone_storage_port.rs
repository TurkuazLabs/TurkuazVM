// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/ports/clone_storage_port.rs
// # 📌 Amac: VM clone storage adapter contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Full/linked disk clone ve guest asset copy islemlerini qemu-img/filesystem detaylarindan ayirir
// # Bagimli Oldugu Katman: Service | Tool

use crate::domain::clone::CloneMode;
use crate::domain::disk::DiskImage;
use crate::domain::virtual_machine::VmId;
use crate::ports::storage_port::StorageImageInfo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloneStorageError {
    NotFound(String),
    AlreadyExists(String),
    UnsupportedFormat(String),
    ExecutionFailed(String),
    UnsafePath(String),
}

pub trait CloneStoragePort {
    fn prepare_clone_target(&self, target_vm_id: &VmId) -> Result<(), CloneStorageError>;

    fn clone_image(
        &self,
        source_vm_id: &VmId,
        source: &DiskImage,
        target_vm_id: &VmId,
        target: &DiskImage,
        mode: CloneMode,
    ) -> Result<StorageImageInfo, CloneStorageError>;

    fn clone_machine_assets(
        &self,
        source_vm_id: &VmId,
        target_vm_id: &VmId,
    ) -> Result<(), CloneStorageError>;

    fn cleanup_clone(&self, target_vm_id: &VmId) -> Result<(), CloneStorageError>;
}
