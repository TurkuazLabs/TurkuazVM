// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/ports/firmware_port.rs
// # 📌 Amac: UEFI firmware hazirlama adapter contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: OVMF benzeri firmware dosyalarini core domain'den ayirir ve rollback sahipligini typed sonuc ile tasir
// # Bagimli Oldugu Katman: Service | Tool

use crate::domain::guest_boot::UefiFirmware;
use crate::domain::virtual_machine::VmId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FirmwareError {
    SourceNotFound(String),
    PrepareFailed(String),
    CleanupFailed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedUefiFirmware {
    pub firmware: UefiFirmware,
    pub vars_created: bool,
}

pub trait FirmwarePort {
    fn prepare_uefi(&self, vm_id: &VmId) -> Result<PreparedUefiFirmware, FirmwareError>;

    fn cleanup_uefi(
        &self,
        vm_id: &VmId,
        prepared: &PreparedUefiFirmware,
    ) -> Result<(), FirmwareError>;
}
