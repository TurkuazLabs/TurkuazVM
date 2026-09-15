// # 📄 Dosya Yolu: /turkuazvm/crates/android/src/ports/android_device_port.rs
// # 📌 Amac: Android device bridge ve display control contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Service katmanini ADB process ve transport detaylarindan ayirir
// # Bagimli Oldugu Katman: Service | Tool

use crate::domain::device::{AndroidBridgeCapabilities, AndroidDeviceReport};
use crate::domain::runtime_profile::AndroidRuntimeProfile;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndroidDeviceError {
    BridgeUnavailable,
    ConnectFailed(String),
    CommandFailed(String),
    CommandTimeout,
    DeviceNotReady,
    InvalidResponse(String),
}

pub trait AndroidDevicePort {
    fn capabilities(&self) -> AndroidBridgeCapabilities;
    fn inspect(&self, profile: &AndroidRuntimeProfile) -> Result<AndroidDeviceReport, AndroidDeviceError>;
    fn wait_until_ready(&self, profile: &AndroidRuntimeProfile) -> Result<AndroidDeviceReport, AndroidDeviceError>;
    fn apply_display(&self, profile: &AndroidRuntimeProfile) -> Result<(), AndroidDeviceError>;
}
