// # 📄 Dosya Yolu: /turkuazvm/crates/gpu/src/ports/host_gpu_probe_port.rs
// # 📌 Amac: Host GPU capability probe kontratini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Gaming GPU service katmanini Windows/Linux Vulkan probe detaylarindan ayirir
// # Bagimli Oldugu Katman: Service | Tool

use crate::domain::capability::HostGpuCapabilities;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostGpuProbeError {
    ProbeFailed(String),
}

pub trait HostGpuProbePort {
    fn probe_host_gpu(&self) -> Result<HostGpuCapabilities, HostGpuProbeError>;
}
