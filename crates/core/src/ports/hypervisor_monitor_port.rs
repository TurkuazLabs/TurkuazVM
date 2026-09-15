// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/ports/hypervisor_monitor_port.rs
// # 📌 Amac: Calisan hypervisor kontrol kanalini inceleyen adapter contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Core katmanini QMP transport ve JSON protokol detaylarindan ayirir
// # Bagimli Oldugu Katman: Service | Tool

use crate::domain::hypervisor_control::HypervisorControlReport;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HypervisorMonitorError {
    ConnectionFailed(String),
    ConnectionClosed,
    TransportFailed(String),
    ProtocolViolation(String),
    CommandFailed {
        class: String,
        description: String,
    },
}

pub trait HypervisorMonitorPort {
    fn inspect_control_plane(
        &mut self,
    ) -> Result<HypervisorControlReport, HypervisorMonitorError>;
}
