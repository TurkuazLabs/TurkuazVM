// # 📄 Dosya Yolu: /turkuazvm/crates/android/src/commands/android_input_command.rs
// # 📌 Amac: Android input injection use-case girdisini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: VM kimligi ve typed input action bilgisini Service katmanina tasir
// # Bagimli Oldugu Katman: Controller | Service

use crate::domain::input::AndroidInputAction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidInputCommand {
    pub vm_id: String,
    pub action: AndroidInputAction,
}
