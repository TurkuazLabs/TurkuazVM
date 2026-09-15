// # 📄 Dosya Yolu: /turkuazvm/crates/android-image/src/commands/android_image_commands.rs
// # 📌 Amac: Android image registry ve assignment use-case girdilerini tanimlar
// # 📌 Modul - Rust
// # Version: 0.36.0
// # Aciklama: Controller/API verisini typed Service command modellerine tasir
// # Bagimli Oldugu Katman: Controller | Service

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefineAndroidImageCommand {
    pub image_id: String,
    pub name: String,
    pub requested_release: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrepareAndroidImageBuildCommand {
    pub image_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterAndroidImageBuildCommand {
    pub image_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignAndroidImageCommand {
    pub vm_id: String,
    pub image_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallAndroidImageDistributionCommand {
    pub image_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelAndroidImageDistributionCommand {
    pub image_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanupAndroidImageDistributionCommand {
    pub image_id: String,
}
