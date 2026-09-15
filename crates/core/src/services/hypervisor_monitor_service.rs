// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/services/hypervisor_monitor_service.rs
// # 📌 Amac: Calisan hypervisor kontrol kanalini application use-case olarak inceler
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Monitor portunu kullanarak runtime surumu ve desteklenen komutlari dondurur
// # Bagimli Oldugu Katman: Tool

use crate::domain::hypervisor_control::HypervisorControlReport;
use crate::ports::hypervisor_monitor_port::{HypervisorMonitorError, HypervisorMonitorPort};

pub struct HypervisorMonitorService<M>
where
    M: HypervisorMonitorPort,
{
    monitor: M,
}

impl<M> HypervisorMonitorService<M>
where
    M: HypervisorMonitorPort,
{
    pub const fn new(monitor: M) -> Self {
        Self { monitor }
    }

    pub fn inspect(&mut self) -> Result<HypervisorControlReport, HypervisorMonitorError> {
        self.monitor.inspect_control_plane()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::hypervisor_control::HypervisorRuntimeVersion;

    struct FakeMonitor;

    impl HypervisorMonitorPort for FakeMonitor {
        fn inspect_control_plane(
            &mut self,
        ) -> Result<HypervisorControlReport, HypervisorMonitorError> {
            Ok(HypervisorControlReport {
                version: HypervisorRuntimeVersion {
                    major: 10,
                    minor: 0,
                    micro: 1,
                    package: String::new(),
                },
                supported_commands: vec![String::from("query-version")],
            })
        }
    }

    #[test]
    fn inspect_delegates_to_monitor_port() {
        let mut service = HypervisorMonitorService::new(FakeMonitor);
        let report = service.inspect().expect("fake monitor should succeed");

        assert_eq!(report.version.major, 10);
        assert_eq!(report.supported_commands.len(), 1);
    }
}
