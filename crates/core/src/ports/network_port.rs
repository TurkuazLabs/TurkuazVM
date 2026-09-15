// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/ports/network_port.rs
// # 📌 Amac: Host network preflight, runtime preparation, cleanup ve recovery adapter contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Core network use-case'lerini managed NAT/private fabric ve Windows/Linux TAP/bridge uygulama detaylarindan ayirir
// # Bagimli Oldugu Katman: Service | Tool

use crate::domain::network::{NetworkAttachment, NetworkMode, NetworkRuntimePlan};
use crate::domain::virtual_machine::{VirtualMachine, VmId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetworkCapabilities {
    pub managed_nat: bool,
    pub private_network: bool,
    pub user_nat: bool,
    pub bridge: bool,
    pub existing_tap: bool,
    pub managed_tap: bool,
    pub bridge_helper: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NetworkRecoveryReport {
    pub active_leases: usize,
    pub cleaned_leases: usize,
    pub failed_leases: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkError {
    UnsupportedMode(NetworkMode),
    InvalidConfiguration(String),
    PreflightFailed(String),
    PrivilegeRequired(String),
    PreparationFailed(String),
    RuntimeBindingFailed(String),
    CleanupFailed(String),
    RecoveryFailed(String),
}

pub trait NetworkPort {
    fn capabilities(&self) -> NetworkCapabilities;

    fn validate_attachment(&self, attachment: &NetworkAttachment) -> Result<(), NetworkError>;

    fn prepare_runtime(
        &mut self,
        machine: &VirtualMachine,
    ) -> Result<NetworkRuntimePlan, NetworkError>;

    fn bind_runtime_process(
        &mut self,
        vm_id: &VmId,
        process_id: u32,
    ) -> Result<(), NetworkError>;

    fn cleanup_runtime(&mut self, vm_id: &VmId) -> Result<(), NetworkError>;

    fn recover_runtime(&mut self) -> Result<NetworkRecoveryReport, NetworkError>;
}
