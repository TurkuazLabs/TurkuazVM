// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/services/host_capability_service.rs
// # 📌 Amac: Host ve hypervisor capability bilgisini tek application use-case icinde birlestirir
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Platforma gore acceleration stratejisini secerek capability raporu uretir
// # Bagimli Oldugu Katman: Repo | Tool

use crate::domain::host::{HostInfo, HostPlatform};
use crate::domain::hypervisor::{AccelerationBackend, HostCapabilityReport};
use crate::ports::host_probe_port::HostProbePort;
use crate::ports::hypervisor_probe_port::HypervisorProbePort;

pub struct HostCapabilityService<H, V>
where
    H: HostProbePort,
    V: HypervisorProbePort,
{
    host_probe: H,
    hypervisor_probe: V,
}

impl<H, V> HostCapabilityService<H, V>
where
    H: HostProbePort,
    V: HypervisorProbePort,
{
    pub const fn new(host_probe: H, hypervisor_probe: V) -> Self {
        Self {
            host_probe,
            hypervisor_probe,
        }
    }

    pub fn inspect(&self) -> HostCapabilityReport {
        let host = self.host_probe.probe_host();
        let preferred_acceleration = Self::preferred_acceleration(&host);
        let hypervisor = self.hypervisor_probe.probe().ok();

        HostCapabilityReport {
            host,
            hypervisor,
            preferred_acceleration,
        }
    }

    fn preferred_acceleration(host: &HostInfo) -> AccelerationBackend {
        match host.platform {
            HostPlatform::Windows => AccelerationBackend::Whpx,
            HostPlatform::Linux => AccelerationBackend::Kvm,
            HostPlatform::Unsupported(_) => AccelerationBackend::Tcg,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::domain::host::{HostArchitecture, HostPlatform};
    use crate::domain::hypervisor::{HypervisorInstallation, HypervisorKind};
    use crate::ports::hypervisor_probe_port::HypervisorProbeError;

    struct FakeHostProbe {
        host: HostInfo,
    }

    impl HostProbePort for FakeHostProbe {
        fn probe_host(&self) -> HostInfo {
            self.host.clone()
        }
    }

    struct FakeHypervisorProbe;

    impl HypervisorProbePort for FakeHypervisorProbe {
        fn probe(&self) -> Result<HypervisorInstallation, HypervisorProbeError> {
            Ok(HypervisorInstallation {
                kind: HypervisorKind::Qemu,
                system_binary: PathBuf::from("qemu-system-x86_64"),
                disk_binary: Some(PathBuf::from("qemu-img")),
                version_text: String::from("QEMU test"),
            })
        }
    }

    #[test]
    fn windows_prefers_whpx() {
        let service = HostCapabilityService::new(
            FakeHostProbe {
                host: HostInfo {
                    platform: HostPlatform::Windows,
                    architecture: HostArchitecture::X86_64,
                },
            },
            FakeHypervisorProbe,
        );

        let report = service.inspect();

        assert_eq!(report.preferred_acceleration, AccelerationBackend::Whpx);
    }

    #[test]
    fn linux_prefers_kvm() {
        let service = HostCapabilityService::new(
            FakeHostProbe {
                host: HostInfo {
                    platform: HostPlatform::Linux,
                    architecture: HostArchitecture::X86_64,
                },
            },
            FakeHypervisorProbe,
        );

        let report = service.inspect();

        assert_eq!(report.preferred_acceleration, AccelerationBackend::Kvm);
    }
}
