// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/ports/hypervisor_runtime_port.rs
// # 📌 Amac: VM process lifecycle hypervisor adapter contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.40.3
// # Aciklama: Service katmanini QEMU process, QMP, live installer media eject, native display ve host network runtime detaylarindan ayirir
// # Bagimli Oldugu Katman: Service | Tool

use crate::domain::display::DisplayRuntimeInfo;
use crate::domain::hypervisor_runtime_event::HypervisorRuntimeEvent;
use crate::domain::network::NetworkRuntimePlan;
use crate::domain::runtime_media::VmRuntimeMediaPlan;
use crate::domain::virtual_machine::{VirtualMachine, VmId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HypervisorRuntimeInfo {
    pub process_id: u32,
    pub display: Option<DisplayRuntimeInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HypervisorTerminationKind {
    Graceful,
    Forced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HypervisorStopReport {
    pub termination: HypervisorTerminationKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HypervisorRuntimeError {
    AlreadyRunning(VmId),
    NotRunning(VmId),
    LaunchFailed(String),
    ControlFailed(String),
    StopFailed(String),
}

pub trait HypervisorRuntimePort {
    fn start(
        &mut self,
        machine: &VirtualMachine,
        network_plan: &NetworkRuntimePlan,
        runtime_media: Option<&VmRuntimeMediaPlan>,
    ) -> Result<HypervisorRuntimeInfo, HypervisorRuntimeError>;

    fn stop(&mut self, vm_id: &VmId) -> Result<HypervisorStopReport, HypervisorRuntimeError>;

    fn eject_installer_media(
        &mut self,
        vm_id: &VmId,
        _media_id: &str,
    ) -> Result<(), HypervisorRuntimeError> {
        Err(HypervisorRuntimeError::ControlFailed(format!(
            "live installer media eject is unavailable for {}",
            vm_id.as_str()
        )))
    }

    fn runtime_info(&self, vm_id: &VmId) -> Result<HypervisorRuntimeInfo, HypervisorRuntimeError> {
        Err(HypervisorRuntimeError::NotRunning(vm_id.clone()))
    }

    fn maintenance(&mut self) -> Result<Vec<HypervisorRuntimeEvent>, HypervisorRuntimeError> {
        Ok(Vec::new())
    }
}
