// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/commands/delete_vm_command.rs
// # 📌 Amac: VM silme use-case girdisini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Silinecek VM kimligini VM Configuration Service katmanina typed command olarak tasir
// # Bagimli Oldugu Katman: Controller | Service

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteVmCommand {
    pub vm_id: String,
}

impl DeleteVmCommand {
    pub fn new(vm_id: impl Into<String>) -> Self {
        Self { vm_id: vm_id.into() }
    }
}
