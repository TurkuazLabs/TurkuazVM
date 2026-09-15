// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/commands/prepare_guest_boot_command.rs
// # 📌 Amac: Guest boot hazirlama use-case girdisini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: ISO import, guest profile, firmware tercihi ve boot order bilgisini service katmanina tasir
// # Bagimli Oldugu Katman: Controller | Service

use std::path::PathBuf;

use crate::domain::guest_boot::{BootDevice, GuestProfile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirmwarePreference {
    Bios,
    Uefi,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallerIsoCommand {
    pub media_id: String,
    pub source_path: PathBuf,
    pub relative_path: String,
}

impl InstallerIsoCommand {
    pub fn new(
        media_id: impl Into<String>,
        source_path: PathBuf,
        relative_path: impl Into<String>,
    ) -> Self {
        Self {
            media_id: media_id.into(),
            source_path,
            relative_path: relative_path.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrepareGuestBootCommand {
    pub vm_id: String,
    pub profile: GuestProfile,
    pub firmware: FirmwarePreference,
    pub boot_devices: Vec<BootDevice>,
    pub boot_once: bool,
    pub installer_iso: Option<InstallerIsoCommand>,
}

impl PrepareGuestBootCommand {
    pub fn new(
        vm_id: impl Into<String>,
        profile: GuestProfile,
        firmware: FirmwarePreference,
        boot_devices: Vec<BootDevice>,
        boot_once: bool,
        installer_iso: Option<InstallerIsoCommand>,
    ) -> Self {
        Self {
            vm_id: vm_id.into(),
            profile,
            firmware,
            boot_devices,
            boot_once,
            installer_iso,
        }
    }
}
