// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/ports/hypervisor_probe_port.rs
// # 📌 Amac: Hypervisor kurulumunu ve surumunu bulan adapter icin port contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Core katmaninin QEMU proses ve PATH detaylarini bilmesini engeller
// # Bagimli Oldugu Katman: Service | Tool

use crate::domain::hypervisor::HypervisorInstallation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HypervisorProbeError {
    NotFound,
    ExecutionFailed(String),
}

pub trait HypervisorProbePort {
    fn probe(&self) -> Result<HypervisorInstallation, HypervisorProbeError>;
}
