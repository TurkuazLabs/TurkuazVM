// # 📄 Dosya Yolu: /turkuazvm/crates/android/src/commands/install_apk_command.rs
// # 📌 Amac: Android APK install use-case girdisini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: VM kimligi ve Engine package root altindaki relative APK yolunu Service katmanina tasir
// # Bagimli Oldugu Katman: Controller | Service

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallApkCommand {
    pub vm_id: String,
    pub relative_apk_path: String,
}
