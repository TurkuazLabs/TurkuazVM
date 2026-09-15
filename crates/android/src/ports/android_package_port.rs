// # 📄 Dosya Yolu: /turkuazvm/crates/android/src/ports/android_package_port.rs
// # 📌 Amac: Android package lifecycle tool contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: APK install ve package lifecycle islemlerini ADB command syntaxindan ayirir
// # Bagimli Oldugu Katman: Service | Tool

use std::path::Path;

use crate::domain::package::{AndroidPackageInfo, AndroidPackageName};
use crate::domain::runtime_profile::AndroidRuntimeProfile;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndroidPackagePortError {
    BridgeUnavailable,
    CommandFailed(String),
    CommandTimeout,
    ApkNotFound,
    ApkOutsidePackageRoot,
    LaunchActivityNotFound,
}

pub trait AndroidPackagePort {
    fn list_packages(&self, profile: &AndroidRuntimeProfile) -> Result<Vec<AndroidPackageInfo>, AndroidPackagePortError>;
    fn install_apk(&self, profile: &AndroidRuntimeProfile, apk_path: &Path) -> Result<(), AndroidPackagePortError>;
    fn uninstall_package(&self, profile: &AndroidRuntimeProfile, package: &AndroidPackageName) -> Result<(), AndroidPackagePortError>;
    fn launch_package(&self, profile: &AndroidRuntimeProfile, package: &AndroidPackageName) -> Result<(), AndroidPackagePortError>;
    fn stop_package(&self, profile: &AndroidRuntimeProfile, package: &AndroidPackageName) -> Result<(), AndroidPackagePortError>;
}
