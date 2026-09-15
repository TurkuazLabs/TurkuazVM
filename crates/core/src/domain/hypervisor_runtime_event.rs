// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/domain/hypervisor_runtime_event.rs
// # 📌 Amac: Hypervisor maintenance sirasinda uretilen runtime olaylarini domain modeli olarak tanimlar
// # 📌 Modul - Rust
// # Version: 0.40.3
// # Aciklama: Guest reset, QMP olaylari ve beklenmedik process cikisini Service katmanina typed olarak tasir
// # Bagimli Oldugu Katman: Service | Tool

use crate::domain::virtual_machine::VmId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HypervisorRuntimeEventKind {
    Qmp { name: String },
    GuestReset,
    ProcessExited,
    ControlReattached,
    ControlUnavailable { detail: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HypervisorRuntimeEvent {
    pub vm_id: VmId,
    pub kind: HypervisorRuntimeEventKind,
}
