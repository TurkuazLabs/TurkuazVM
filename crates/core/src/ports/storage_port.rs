// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/ports/storage_port.rs
// # 📌 Amac: Sanal disk image islemleri icin storage adapter contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Core service katmanini qemu-img ve host filesystem detaylarindan ayirir
// # Bagimli Oldugu Katman: Service | Tool

use crate::domain::disk::{DiskFormat, DiskImage};
use crate::domain::virtual_machine::VmId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageImageInfo {
    pub format: DiskFormat,
    pub virtual_size_bytes: u64,
    pub actual_size_bytes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageError {
    NotFound(String),
    AlreadyExists(String),
    ExecutionFailed(String),
    InvalidOutput(String),
    UnsafePath(String),
}

pub trait StoragePort {
    fn create_image(&self, vm_id: &VmId, image: &DiskImage) -> Result<StorageImageInfo, StorageError>;
    fn inspect_image(&self, vm_id: &VmId, image: &DiskImage) -> Result<StorageImageInfo, StorageError>;
    fn resize_image(
        &self,
        vm_id: &VmId,
        image: &DiskImage,
        new_virtual_size_bytes: u64,
    ) -> Result<StorageImageInfo, StorageError>;
    fn delete_image(&self, vm_id: &VmId, image: &DiskImage) -> Result<(), StorageError>;
}
