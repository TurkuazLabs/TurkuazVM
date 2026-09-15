// # 📄 Dosya Yolu: /turkuazvm/crates/android/src/commands/configure_android_runtime_command.rs
// # 📌 Amac: Android runtime configuration use-case girdisini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: VM, ADB portu ve device display profile bilgisini Service katmanina tasir
// # Bagimli Oldugu Katman: Controller | Service

use std::net::Ipv4Addr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigureAndroidRuntimeCommand {
    pub vm_id: String,
    pub adb_host_ip: Ipv4Addr,
    pub adb_host_port: u16,
    pub adb_guest_port: u16,
    pub width: u32,
    pub height: u32,
    pub density_dpi: u32,
    pub target_fps: u16,
}
