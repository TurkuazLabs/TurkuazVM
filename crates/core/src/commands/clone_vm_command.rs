// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/commands/clone_vm_command.rs
// # 📌 Amac: Full veya linked VM clone use-case girdisini tasir
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Kaynak/hedef VM bilgisi ile typed CloneMode secimini Service katmanina tasir
// # Bagimli Oldugu Katman: Service

use crate::domain::clone::CloneMode;

pub struct CloneVmCommand {
    pub source_vm_id: String,
    pub target_vm_id: String,
    pub target_name: String,
    pub mode: CloneMode,
}

impl CloneVmCommand {
    pub fn new(
        source_vm_id: impl Into<String>,
        target_vm_id: impl Into<String>,
        target_name: impl Into<String>,
        mode: CloneMode,
    ) -> Self {
        Self {
            source_vm_id: source_vm_id.into(),
            target_vm_id: target_vm_id.into(),
            target_name: target_name.into(),
            mode,
        }
    }
}
