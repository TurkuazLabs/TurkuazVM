// # 📄 Dosya Yolu: /turkuazvm/crates/android/src/services/android_package_service.rs
// # 📌 Amac: Android APK ve package lifecycle use-case is kurallarini yonetir
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Profile repository ile package tool portunu orkestre eder ve APK relative path kuralini uygular
// # Bagimli Oldugu Katman: Repo | Tool

use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::commands::install_apk_command::InstallApkCommand;
use crate::commands::package_action_command::PackageActionCommand;
use crate::domain::package::{AndroidPackageError, AndroidPackageInfo, AndroidPackageName};
use crate::domain::runtime_profile::{AndroidProfileError, AndroidVmId};
use crate::ports::android_package_port::{AndroidPackagePort, AndroidPackagePortError};
use crate::ports::android_profile_repository_port::{
    AndroidProfileRepositoryError, AndroidProfileRepositoryPort,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndroidPackageServiceError {
    ProfileDomain(AndroidProfileError),
    PackageDomain(AndroidPackageError),
    Repository(AndroidProfileRepositoryError),
    PackagePort(AndroidPackagePortError),
    PackageRootUnavailable,
    ApkNotFound,
    ApkOutsidePackageRoot,
}

pub struct AndroidPackageService<R, P>
where
    R: AndroidProfileRepositoryPort,
    P: AndroidPackagePort,
{
    repository: R,
    package: P,
    package_root: PathBuf,
}

impl<R, P> AndroidPackageService<R, P>
where
    R: AndroidProfileRepositoryPort,
    P: AndroidPackagePort,
{
    pub fn new(repository: R, package: P, package_root: PathBuf) -> Self {
        Self {
            repository,
            package,
            package_root,
        }
    }

    pub fn list_packages(&self, vm_id: &str) -> Result<Vec<AndroidPackageInfo>, AndroidPackageServiceError> {
        let profile = self.profile(vm_id)?;
        self.package
            .list_packages(&profile)
            .map_err(AndroidPackageServiceError::PackagePort)
    }

    pub fn install_apk(&self, command: InstallApkCommand) -> Result<(), AndroidPackageServiceError> {
        let profile = self.profile(&command.vm_id)?;
        let relative = validate_apk_relative_path(&command.relative_apk_path)
            .map_err(AndroidPackageServiceError::PackageDomain)?;
        let path = resolve_apk_path(&self.package_root, &relative)?;
        self.package
            .install_apk(&profile, &path)
            .map_err(AndroidPackageServiceError::PackagePort)
    }

    pub fn uninstall(&self, command: PackageActionCommand) -> Result<(), AndroidPackageServiceError> {
        let profile = self.profile(&command.vm_id)?;
        let package = AndroidPackageName::parse(command.package_name)
            .map_err(AndroidPackageServiceError::PackageDomain)?;
        self.package
            .uninstall_package(&profile, &package)
            .map_err(AndroidPackageServiceError::PackagePort)
    }

    pub fn launch(&self, command: PackageActionCommand) -> Result<(), AndroidPackageServiceError> {
        let profile = self.profile(&command.vm_id)?;
        let package = AndroidPackageName::parse(command.package_name)
            .map_err(AndroidPackageServiceError::PackageDomain)?;
        self.package
            .launch_package(&profile, &package)
            .map_err(AndroidPackageServiceError::PackagePort)
    }

    pub fn stop(&self, command: PackageActionCommand) -> Result<(), AndroidPackageServiceError> {
        let profile = self.profile(&command.vm_id)?;
        let package = AndroidPackageName::parse(command.package_name)
            .map_err(AndroidPackageServiceError::PackageDomain)?;
        self.package
            .stop_package(&profile, &package)
            .map_err(AndroidPackageServiceError::PackagePort)
    }

    fn profile(&self, vm_id: &str) -> Result<crate::domain::runtime_profile::AndroidRuntimeProfile, AndroidPackageServiceError> {
        let vm_id = AndroidVmId::parse(vm_id.to_owned())
            .map_err(AndroidPackageServiceError::ProfileDomain)?;
        self.repository
            .get(&vm_id)
            .map_err(AndroidPackageServiceError::Repository)
    }
}

fn validate_apk_relative_path(value: &str) -> Result<PathBuf, AndroidPackageError> {
    let path = Path::new(value);
    if value.trim().is_empty()
        || path.is_absolute()
        || !value.to_ascii_lowercase().ends_with(".apk")
        || path.components().any(|component| {
            matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_))
        })
    {
        return Err(AndroidPackageError::InvalidApkRelativePath);
    }
    Ok(path.to_path_buf())
}

fn resolve_apk_path(
    package_root: &Path,
    relative: &Path,
) -> Result<PathBuf, AndroidPackageServiceError> {
    let canonical_root = fs::canonicalize(package_root)
        .map_err(|_| AndroidPackageServiceError::PackageRootUnavailable)?;
    let candidate = package_root.join(relative);
    let canonical_candidate = fs::canonicalize(&candidate)
        .map_err(|_| AndroidPackageServiceError::ApkNotFound)?;
    if !canonical_candidate.starts_with(&canonical_root) || !canonical_candidate.is_file() {
        return Err(AndroidPackageServiceError::ApkOutsidePackageRoot);
    }
    Ok(canonical_candidate)
}

#[cfg(test)]
mod path_tests {
    use super::*;

    #[test]
    fn parent_directory_apk_path_is_rejected() {
        assert_eq!(
            validate_apk_relative_path("../escape.apk"),
            Err(AndroidPackageError::InvalidApkRelativePath)
        );
    }

    #[test]
    fn non_apk_extension_is_rejected() {
        assert_eq!(
            validate_apk_relative_path("packages/game.zip"),
            Err(AndroidPackageError::InvalidApkRelativePath)
        );
    }
}
