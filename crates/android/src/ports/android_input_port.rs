// # 📄 Dosya Yolu: /turkuazvm/crates/android/src/ports/android_input_port.rs
// # 📌 Amac: Android guest input injection tool contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Gaming input service'ini ADB shell input implementationindan ayirir
// # Bagimli Oldugu Katman: Service | Tool

use crate::domain::input::AndroidInputAction;
use crate::domain::runtime_profile::AndroidRuntimeProfile;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndroidInputPortError {
    BridgeUnavailable,
    CommandFailed(String),
    CommandTimeout,
}

pub trait AndroidInputPort {
    fn inject(&self, profile: &AndroidRuntimeProfile, action: &AndroidInputAction) -> Result<(), AndroidInputPortError>;
}
