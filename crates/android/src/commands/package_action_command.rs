// # 📄 Dosya Yolu: /turkuazvm/crates/android/src/commands/package_action_command.rs
// # 📌 Amac: Android package lifecycle action girdisini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Uninstall, launch ve force-stop use-case'leri icin VM ve package kimligini tasir
// # Bagimli Oldugu Katman: Controller | Service

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageActionCommand {
    pub vm_id: String,
    pub package_name: String,
}
