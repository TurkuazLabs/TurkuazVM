// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/commands/create_vm_command.rs
// # 📌 Amac: Yeni VM olusturma use-case girdisini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Controller tarafindan Service katmanina tasinan VM olusturma verisini modeller
// # Bagimli Oldugu Katman: Controller | Service

use crate::domain::guest_boot::GuestProfile;
use crate::domain::hypervisor::AccelerationBackend;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateVmCommand {
    pub vm_id: String,
    pub name: String,
    pub vcpu_count: u16,
    pub memory_mib: u64,
    pub acceleration: AccelerationBackend,
    pub guest_profile: GuestProfile,
    pub guest_template_id: Option<String>,
}

impl CreateVmCommand {
    pub fn new(
        vm_id: impl Into<String>,
        name: impl Into<String>,
        vcpu_count: u16,
        memory_mib: u64,
        acceleration: AccelerationBackend,
        guest_profile: GuestProfile,
        guest_template_id: Option<String>,
    ) -> Self {
        Self {
            vm_id: vm_id.into(),
            name: name.into(),
            vcpu_count,
            memory_mib,
            acceleration,
            guest_profile,
            guest_template_id,
        }
    }
}
