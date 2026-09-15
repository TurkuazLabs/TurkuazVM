// # 📄 Dosya Yolu: /turkuazvm/crates/android/src/ports/android_guest_agent_port.rs
// # 📌 Amac: Android Guest Agent infrastructure adapter contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: ADB forward, TCP ve wire protocol detaylarini Android application servisinden ayirir
// # Bagimli Oldugu Katman: Service | Tool

use crate::domain::guest_agent::{AndroidGuestAgentReport, AndroidTouchContact};
use crate::domain::runtime_profile::AndroidRuntimeProfile;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndroidGuestAgentPortError {
    BridgeUnavailable,
    PortExhausted,
    ForwardFailed(String),
    ProvisionFailed(String),
    ConnectFailed(String),
    Protocol(String),
    Timeout,
}

pub trait AndroidGuestAgentPort {
    fn provision_and_wait_ready(&mut self, profile: &AndroidRuntimeProfile) -> Result<AndroidGuestAgentReport, AndroidGuestAgentPortError>;
    fn inspect(&mut self, profile: &AndroidRuntimeProfile) -> Result<AndroidGuestAgentReport, AndroidGuestAgentPortError>;
    fn apply_touch_frame(&mut self, profile: &AndroidRuntimeProfile, contacts: &[AndroidTouchContact]) -> Result<(), AndroidGuestAgentPortError>;
    fn reset_input(&mut self, profile: &AndroidRuntimeProfile) -> Result<(), AndroidGuestAgentPortError>;
    fn read_clipboard(&mut self, profile: &AndroidRuntimeProfile) -> Result<String, AndroidGuestAgentPortError>;
    fn write_clipboard(&mut self, profile: &AndroidRuntimeProfile, text: &str) -> Result<(), AndroidGuestAgentPortError>;
}
