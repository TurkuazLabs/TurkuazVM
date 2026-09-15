// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/domain/display.rs
// # 📌 Amac: Native VM display session ve input capture runtime modellerini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Display transportunu QEMU/VNC ayrintisina baglamadan endpoint, framebuffer ve input capture yeteneklerini tasir
// # Bagimli Oldugu Katman: Service | Tool | View

use std::net::SocketAddr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayTransport {
    Rfb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputCaptureMode {
    Absolute,
    Relative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayCapabilities {
    pub native_window: bool,
    pub fullscreen: bool,
    pub absolute_pointer: bool,
    pub relative_pointer_capture: bool,
    pub keyboard: bool,
    pub gamepad_observation: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayRuntimeInfo {
    pub transport: DisplayTransport,
    pub endpoint: SocketAddr,
    pub local_only: bool,
    pub capabilities: DisplayCapabilities,
}
