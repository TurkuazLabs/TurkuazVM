// # 📄 Dosya Yolu: /turkuazvm/crates/android/src/domain/runtime_profile.rs
// # 📌 Amac: Android VM runtime profile ve display invariantlarini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: ADB endpoint, ekran boyutu, density ve hedef FPS bilgisini typed domain modeli olarak tutar
// # Bagimli Oldugu Katman: Service | Repo

use std::net::Ipv4Addr;

const MIN_WIDTH: u32 = 480;
const MAX_WIDTH: u32 = 7680;
const MIN_HEIGHT: u32 = 480;
const MAX_HEIGHT: u32 = 4320;
const MIN_DENSITY_DPI: u32 = 120;
const MAX_DENSITY_DPI: u32 = 1000;
const MIN_TARGET_FPS: u16 = 30;
const MAX_TARGET_FPS: u16 = 240;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AndroidVmId(String);

impl AndroidVmId {
    pub fn parse(value: impl Into<String>) -> Result<Self, AndroidProfileError> {
        let value = value.into();
        let valid = !value.is_empty()
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'));
        if !valid {
            return Err(AndroidProfileError::InvalidVmId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidDisplayProfile {
    width: u32,
    height: u32,
    density_dpi: u32,
    target_fps: u16,
}

impl AndroidDisplayProfile {
    pub fn create(
        width: u32,
        height: u32,
        density_dpi: u32,
        target_fps: u16,
    ) -> Result<Self, AndroidProfileError> {
        if !(MIN_WIDTH..=MAX_WIDTH).contains(&width) {
            return Err(AndroidProfileError::InvalidWidth(width));
        }
        if !(MIN_HEIGHT..=MAX_HEIGHT).contains(&height) {
            return Err(AndroidProfileError::InvalidHeight(height));
        }
        if !(MIN_DENSITY_DPI..=MAX_DENSITY_DPI).contains(&density_dpi) {
            return Err(AndroidProfileError::InvalidDensityDpi(density_dpi));
        }
        if !(MIN_TARGET_FPS..=MAX_TARGET_FPS).contains(&target_fps) {
            return Err(AndroidProfileError::InvalidTargetFps(target_fps));
        }
        Ok(Self {
            width,
            height,
            density_dpi,
            target_fps,
        })
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub const fn density_dpi(&self) -> u32 {
        self.density_dpi
    }

    pub const fn target_fps(&self) -> u16 {
        self.target_fps
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidRuntimeProfile {
    vm_id: AndroidVmId,
    adb_host_ip: Ipv4Addr,
    adb_host_port: u16,
    adb_guest_port: u16,
    display: AndroidDisplayProfile,
}

impl AndroidRuntimeProfile {
    pub fn create(
        vm_id: AndroidVmId,
        adb_host_ip: Ipv4Addr,
        adb_host_port: u16,
        adb_guest_port: u16,
        display: AndroidDisplayProfile,
    ) -> Result<Self, AndroidProfileError> {
        if !adb_host_ip.is_loopback() {
            return Err(AndroidProfileError::AdbMustUseLoopback);
        }
        if adb_host_port == 0 {
            return Err(AndroidProfileError::InvalidAdbHostPort);
        }
        if adb_guest_port == 0 {
            return Err(AndroidProfileError::InvalidAdbGuestPort);
        }
        Ok(Self {
            vm_id,
            adb_host_ip,
            adb_host_port,
            adb_guest_port,
            display,
        })
    }

    pub fn vm_id(&self) -> &AndroidVmId {
        &self.vm_id
    }

    pub const fn adb_host_ip(&self) -> Ipv4Addr {
        self.adb_host_ip
    }

    pub const fn adb_host_port(&self) -> u16 {
        self.adb_host_port
    }

    pub const fn adb_guest_port(&self) -> u16 {
        self.adb_guest_port
    }

    pub const fn display(&self) -> &AndroidDisplayProfile {
        &self.display
    }

    pub fn adb_serial(&self) -> String {
        format!("{}:{}", self.adb_host_ip, self.adb_host_port)
    }

    pub fn replace_display(&mut self, display: AndroidDisplayProfile) {
        self.display = display;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndroidProfileError {
    InvalidVmId,
    InvalidWidth(u32),
    InvalidHeight(u32),
    InvalidDensityDpi(u32),
    InvalidTargetFps(u16),
    AdbMustUseLoopback,
    InvalidAdbHostPort,
    InvalidAdbGuestPort,
}
