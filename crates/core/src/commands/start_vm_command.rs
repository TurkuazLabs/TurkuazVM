// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/commands/start_vm_command.rs
// # 📌 Amac: VM baslatma use-case girdisini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Baslatilacak VM kimligini ve istege bagli runtime medya planini Service katmanina tasir
// # Bagimli Oldugu Katman: Controller | Service

use crate::domain::runtime_media::VmRuntimeMediaPlan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartVmCommand {
    pub vm_id: String,
    pub runtime_media: Option<VmRuntimeMediaPlan>,
}

impl StartVmCommand {
    pub fn new(vm_id: impl Into<String>) -> Self {
        Self {
            vm_id: vm_id.into(),
            runtime_media: None,
        }
    }

    pub fn with_runtime_media(mut self, runtime_media: VmRuntimeMediaPlan) -> Self {
        self.runtime_media = Some(runtime_media);
        self
    }
}
