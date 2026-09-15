// # 📄 Dosya Yolu: /turkuazvm/apps/engine/src/tools/engine_hypervisor_runtime_tool.rs
// # 📌 Amac: VM runtime planina gore QEMU veya resmi Android SDK Emulator process adapterini secer
// # 📌 Modul - Rust
// # Version: 0.40.13
// # Aciklama: Genel VM'leri QEMU'da, SDK System Image Android VM'lerini Android Emulator runtime'inda calistirir
// # Bagimli Oldugu Katman: Service | Tool

use turkuazvm_core::domain::hypervisor::HypervisorInstallation;
use turkuazvm_core::domain::hypervisor_runtime_event::HypervisorRuntimeEvent;
use turkuazvm_core::domain::network::NetworkRuntimePlan;
use turkuazvm_core::domain::runtime_media::VmRuntimeMediaPlan;
use turkuazvm_core::domain::virtual_machine::{VirtualMachine, VmId};
use turkuazvm_core::ports::hypervisor_runtime_port::{
    HypervisorRuntimeError, HypervisorRuntimeInfo, HypervisorRuntimePort, HypervisorStopReport,
};
use turkuazvm_qemu::domain::qemu_runtime::QemuRuntimeSettings;
use turkuazvm_qemu::tools::qemu_runtime_tool::QemuRuntimeTool;
use turkuazvm_repositories::repositories::yaml_runtime_registry_repository::YamlRuntimeRegistryRepository;

use crate::tools::android_sdk_emulator_runtime_tool::AndroidSdkEmulatorRuntimeTool;

pub struct EngineHypervisorRuntimeTool {
    qemu: Option<QemuRuntimeTool<YamlRuntimeRegistryRepository>>,
    android_emulator: AndroidSdkEmulatorRuntimeTool,
}

impl EngineHypervisorRuntimeTool {
    pub fn new(installation: Option<HypervisorInstallation>, settings: QemuRuntimeSettings) -> Self {
        let android_emulator = AndroidSdkEmulatorRuntimeTool::new(
            settings.data_root.clone(),
            settings.startup_timeout,
            settings.startup_poll_interval,
        );
        let qemu = installation.map(|installation| {
            let registry = YamlRuntimeRegistryRepository::new(settings.data_root.clone());
            QemuRuntimeTool::new(installation.system_binary, settings, registry)
        });
        Self { qemu, android_emulator }
    }

    fn qemu_mut(&mut self) -> Result<&mut QemuRuntimeTool<YamlRuntimeRegistryRepository>, HypervisorRuntimeError> {
        self.qemu.as_mut().ok_or_else(|| HypervisorRuntimeError::LaunchFailed(String::from("QEMU runtime is unavailable")))
    }
}

impl HypervisorRuntimePort for EngineHypervisorRuntimeTool {
    fn start(
        &mut self,
        machine: &VirtualMachine,
        network_plan: &NetworkRuntimePlan,
        runtime_media: Option<&VmRuntimeMediaPlan>,
    ) -> Result<HypervisorRuntimeInfo, HypervisorRuntimeError> {
        if let Some(VmRuntimeMediaPlan::AndroidSdkEmulator(plan)) = runtime_media {
            return self.android_emulator.start(machine, plan);
        }
        self.qemu_mut()?.start(machine, network_plan, runtime_media)
    }

    fn stop(&mut self, vm_id: &VmId) -> Result<HypervisorStopReport, HypervisorRuntimeError> {
        if self.android_emulator.contains(vm_id) { return self.android_emulator.stop(vm_id); }
        match self.qemu.as_mut() {
            Some(runtime) => runtime.stop(vm_id),
            None => Err(HypervisorRuntimeError::NotRunning(vm_id.clone())),
        }
    }

    fn eject_installer_media(&mut self, vm_id: &VmId, media_id: &str) -> Result<(), HypervisorRuntimeError> {
        if self.android_emulator.contains(vm_id) {
            return Err(HypervisorRuntimeError::ControlFailed(String::from("Android SDK Emulator runtime has no removable installer media")));
        }
        match self.qemu.as_mut() {
            Some(runtime) => runtime.eject_installer_media(vm_id, media_id),
            None => Err(HypervisorRuntimeError::NotRunning(vm_id.clone())),
        }
    }

    fn maintenance(&mut self) -> Result<Vec<HypervisorRuntimeEvent>, HypervisorRuntimeError> {
        let mut events = self.android_emulator.maintenance()?;
        if let Some(runtime) = self.qemu.as_mut() { events.extend(runtime.maintenance()?); }
        Ok(events)
    }

    fn runtime_info(&self, vm_id: &VmId) -> Result<HypervisorRuntimeInfo, HypervisorRuntimeError> {
        if self.android_emulator.contains(vm_id) { return self.android_emulator.runtime_info(vm_id); }
        match self.qemu.as_ref() {
            Some(runtime) => runtime.runtime_info(vm_id),
            None => Err(HypervisorRuntimeError::NotRunning(vm_id.clone())),
        }
    }
}
