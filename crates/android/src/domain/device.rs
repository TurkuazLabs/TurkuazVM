// # 📄 Dosya Yolu: /turkuazvm/crates/android/src/domain/device.rs
// # 📌 Amac: Android device readiness, ABI ve bridge capability modellerini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: ADB adapterinden gelen cihaz durumunu dis arac semantiginden bagimsiz modeller
// # Bagimli Oldugu Katman: Service | Tool

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndroidAbi {
    X86,
    X86_64,
    ArmeabiV7a,
    Arm64V8a,
    Unknown(String),
}

impl AndroidAbi {
    pub fn parse(value: &str) -> Self {
        match value.trim() {
            "x86" => Self::X86,
            "x86_64" => Self::X86_64,
            "armeabi-v7a" => Self::ArmeabiV7a,
            "arm64-v8a" => Self::Arm64V8a,
            other => Self::Unknown(other.to_owned()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::X86 => "x86",
            Self::X86_64 => "x86_64",
            Self::ArmeabiV7a => "armeabi-v7a",
            Self::Arm64V8a => "arm64-v8a",
            Self::Unknown(value) => value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndroidConnectionState {
    Disconnected,
    Offline,
    Unauthorized,
    Booting,
    Ready,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidDeviceReport {
    pub serial: String,
    pub connection_state: AndroidConnectionState,
    pub boot_completed: bool,
    pub sdk_level: Option<u32>,
    pub abi: Option<AndroidAbi>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidBridgeCapabilities {
    pub available: bool,
    pub version_text: Option<String>,
    pub package_management: bool,
    pub display_override: bool,
    pub input_injection: bool,
}

impl AndroidBridgeCapabilities {
    pub fn unavailable() -> Self {
        Self {
            available: false,
            version_text: None,
            package_management: false,
            display_override: false,
            input_injection: false,
        }
    }
}
