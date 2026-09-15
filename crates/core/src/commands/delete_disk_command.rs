// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/commands/delete_disk_command.rs
// # 📌 Amac: VM disk silme use-case girdisini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: VM ve disk kimligini Storage Service katmanina typed command olarak tasir
// # Bagimli Oldugu Katman: Controller | Service

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteDiskCommand {
    pub vm_id: String,
    pub disk_id: String,
}

impl DeleteDiskCommand {
    pub fn new(vm_id: impl Into<String>, disk_id: impl Into<String>) -> Self {
        Self {
            vm_id: vm_id.into(),
            disk_id: disk_id.into(),
        }
    }
}
