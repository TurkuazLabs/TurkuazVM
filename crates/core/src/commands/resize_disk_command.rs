// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/commands/resize_disk_command.rs
// # 📌 Amac: Sanal disk buyutme use-case girdisini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Controller tarafindan StorageService'e tasinan disk resize istegini modeller
// # Bagimli Oldugu Katman: Controller | Service

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResizeDiskCommand {
    pub vm_id: String,
    pub disk_id: String,
    pub new_virtual_size_bytes: u64,
}

impl ResizeDiskCommand {
    pub fn new(
        vm_id: impl Into<String>,
        disk_id: impl Into<String>,
        new_virtual_size_bytes: u64,
    ) -> Self {
        Self {
            vm_id: vm_id.into(),
            disk_id: disk_id.into(),
            new_virtual_size_bytes,
        }
    }
}
