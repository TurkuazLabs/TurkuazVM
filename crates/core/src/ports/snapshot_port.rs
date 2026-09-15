// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/ports/snapshot_port.rs
// # 📌 Amac: Disk snapshot adapterleri icin infrastructure contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Core SnapshotService katmanini qemu-img internal snapshot detaylarindan ayirir
// # Bagimli Oldugu Katman: Service | Tool

use crate::domain::disk::DiskImage;
use crate::domain::snapshot::SnapshotId;
use crate::domain::virtual_machine::VmId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotStorageError {
    NotFound(String),
    AlreadyExists(String),
    UnsupportedFormat(String),
    ExecutionFailed(String),
    UnsafePath(String),
}

pub trait SnapshotPort {
    fn create_snapshot(
        &self,
        vm_id: &VmId,
        image: &DiskImage,
        snapshot_id: &SnapshotId,
    ) -> Result<(), SnapshotStorageError>;

    fn restore_snapshot(
        &self,
        vm_id: &VmId,
        image: &DiskImage,
        snapshot_id: &SnapshotId,
    ) -> Result<(), SnapshotStorageError>;

    fn delete_snapshot(
        &self,
        vm_id: &VmId,
        image: &DiskImage,
        snapshot_id: &SnapshotId,
    ) -> Result<(), SnapshotStorageError>;
}
