// # 📄 Dosya Yolu: /turkuazvm/crates/qemu/src/domain/qemu_runtime.rs
// # 📌 Amac: QEMU process lifecycle ve display adapter ayarlarini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: QMP, native RFB display, Gaming GPU backend, data root ve process timeout ayarlarini inline config kullanmadan tasir
// # Bagimli Oldugu Katman: Tool

use std::net::IpAddr;
use std::path::PathBuf;
use std::time::Duration;

use turkuazvm_gpu::domain::profile::GpuBackend;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QemuDisplayMode {
    NativeRfb,
    Default,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QemuDisplayRuntimePlan {
    pub mode: QemuDisplayMode,
    pub rfb_display_number: Option<u16>,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QemuGpuRuntimeSettings {
    pub backend: GpuBackend,
    pub hostmem_mib: u64,
    pub experimental: bool,
}

#[derive(Debug, Clone)]
pub struct QemuRuntimeSettings {
    pub qmp_bind_ip: IpAddr,
    pub qmp_connect_timeout: Duration,
    pub qmp_read_timeout: Duration,
    pub qmp_write_timeout: Duration,
    pub startup_timeout: Duration,
    pub startup_poll_interval: Duration,
    pub shutdown_timeout: Duration,
    pub shutdown_poll_interval: Duration,
    pub display_mode: QemuDisplayMode,
    pub gpu: QemuGpuRuntimeSettings,
    pub rfb_bind_ip: IpAddr,
    pub rfb_display_min: u16,
    pub rfb_display_max: u16,
    pub data_root: PathBuf,
}
