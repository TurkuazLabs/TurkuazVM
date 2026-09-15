// # 📄 Dosya Yolu: /turkuazvm/crates/android/src/services/android_runtime_service.rs
// # 📌 Amac: Android runtime profile, readiness ve display use-case is kurallarini yonetir
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Profile persistence ile device bridge portunu orkestre eder
// # Bagimli Oldugu Katman: Repo | Tool

use crate::commands::configure_android_runtime_command::ConfigureAndroidRuntimeCommand;
use crate::domain::device::{AndroidBridgeCapabilities, AndroidDeviceReport};
use crate::domain::runtime_profile::{
    AndroidDisplayProfile, AndroidProfileError, AndroidRuntimeProfile, AndroidVmId,
};
use crate::ports::android_device_port::{AndroidDeviceError, AndroidDevicePort};
use crate::ports::android_profile_repository_port::{
    AndroidProfileRepositoryError, AndroidProfileRepositoryPort,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndroidRuntimeServiceError {
    Domain(AndroidProfileError),
    Repository(AndroidProfileRepositoryError),
    Device(AndroidDeviceError),
}

pub struct AndroidRuntimeService<R, D>
where
    R: AndroidProfileRepositoryPort,
    D: AndroidDevicePort,
{
    repository: R,
    device: D,
}

impl<R, D> AndroidRuntimeService<R, D>
where
    R: AndroidProfileRepositoryPort,
    D: AndroidDevicePort,
{
    pub const fn new(repository: R, device: D) -> Self {
        Self { repository, device }
    }

    pub fn configure(
        &mut self,
        command: ConfigureAndroidRuntimeCommand,
    ) -> Result<AndroidRuntimeProfile, AndroidRuntimeServiceError> {
        let profile = AndroidRuntimeProfile::create(
            AndroidVmId::parse(command.vm_id).map_err(AndroidRuntimeServiceError::Domain)?,
            command.adb_host_ip,
            command.adb_host_port,
            command.adb_guest_port,
            AndroidDisplayProfile::create(
                command.width,
                command.height,
                command.density_dpi,
                command.target_fps,
            )
            .map_err(AndroidRuntimeServiceError::Domain)?,
        )
        .map_err(AndroidRuntimeServiceError::Domain)?;
        self.repository
            .save(profile.clone())
            .map_err(AndroidRuntimeServiceError::Repository)?;
        Ok(profile)
    }

    pub fn profile(&self, vm_id: &str) -> Result<AndroidRuntimeProfile, AndroidRuntimeServiceError> {
        let vm_id = AndroidVmId::parse(vm_id.to_owned()).map_err(AndroidRuntimeServiceError::Domain)?;
        self.repository
            .get(&vm_id)
            .map_err(AndroidRuntimeServiceError::Repository)
    }

    pub fn inspect(&self, vm_id: &str) -> Result<AndroidDeviceReport, AndroidRuntimeServiceError> {
        let profile = self.profile(vm_id)?;
        self.device
            .inspect(&profile)
            .map_err(AndroidRuntimeServiceError::Device)
    }

    pub fn wait_until_ready(
        &self,
        vm_id: &str,
    ) -> Result<AndroidDeviceReport, AndroidRuntimeServiceError> {
        let profile = self.profile(vm_id)?;
        self.device
            .wait_until_ready(&profile)
            .map_err(AndroidRuntimeServiceError::Device)
    }

    pub fn apply_display(&self, vm_id: &str) -> Result<(), AndroidRuntimeServiceError> {
        let profile = self.profile(vm_id)?;
        self.device
            .apply_display(&profile)
            .map_err(AndroidRuntimeServiceError::Device)
    }

    pub fn capabilities(&self) -> AndroidBridgeCapabilities {
        self.device.capabilities()
    }

    pub fn into_parts(self) -> (R, D) {
        (self.repository, self.device)
    }
}
