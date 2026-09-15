// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/domain/hypervisor.rs
// # 📌 Amac: Hypervisor ve acceleration capability domain modellerini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: QEMU gibi adapterlerin ortak capability sonucunu core katmanina tasir
// # Bagimli Oldugu Katman: Service | Tool

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HypervisorKind {
    Qemu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccelerationBackend {
    Whpx,
    Kvm,
    Tcg,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HypervisorInstallation {
    pub kind: HypervisorKind,
    pub system_binary: PathBuf,
    pub disk_binary: Option<PathBuf>,
    pub version_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostCapabilityReport {
    pub host: crate::domain::host::HostInfo,
    pub hypervisor: Option<HypervisorInstallation>,
    pub preferred_acceleration: AccelerationBackend,
}
