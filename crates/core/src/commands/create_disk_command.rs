// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/commands/create_disk_command.rs
// # 📌 Amac: Yeni sanal disk olusturma ve VM'ye baglama use-case girdisini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Controller tarafindan StorageService katmanina tasinan disk olusturma verisini modeller
// # Bagimli Oldugu Katman: Controller | Service

use crate::domain::disk::{DiskBus, DiskFormat};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateDiskCommand {
    pub vm_id: String,
    pub disk_id: String,
    pub format: DiskFormat,
    pub virtual_size_bytes: u64,
    pub relative_path: String,
    pub bus: DiskBus,
    pub boot_index: Option<u8>,
}

impl CreateDiskCommand {
    pub fn new(
        vm_id: impl Into<String>,
        disk_id: impl Into<String>,
        format: DiskFormat,
        virtual_size_bytes: u64,
        relative_path: impl Into<String>,
        bus: DiskBus,
        boot_index: Option<u8>,
    ) -> Self {
        Self {
            vm_id: vm_id.into(),
            disk_id: disk_id.into(),
            format,
            virtual_size_bytes,
            relative_path: relative_path.into(),
            bus,
            boot_index,
        }
    }
}
