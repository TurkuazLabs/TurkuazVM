// # 📄 Dosya Yolu: /turkuazvm/crates/gpu/src/ports/hypervisor_gpu_probe_port.rs
// # 📌 Amac: Hypervisor GPU backend capability probe kontratini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Gaming GPU service katmanini QEMU device/property discovery ayrintilarindan ayirir
// # Bagimli Oldugu Katman: Service | Tool

use crate::domain::capability::HypervisorGpuCapabilities;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HypervisorGpuProbeError {
    HypervisorUnavailable,
    ProbeFailed(String),
}

pub trait HypervisorGpuProbePort {
    fn probe_hypervisor_gpu(&self) -> Result<HypervisorGpuCapabilities, HypervisorGpuProbeError>;
}
