// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/services/guest_boot_service.rs
// # 📌 Amac: Guest boot media ve firmware hazirlama use-case kurallarini orkestre eder
// # 📌 Modul - Rust
// # Version: 0.32.1
// # Aciklama: Stopped VM icin ISO import/cikarma, BIOS/UEFI preparation, boot order ve persistence akisini yonetir
// # Bagimli Oldugu Katman: Repo | Tool

use crate::commands::eject_installer_media_command::EjectInstallerMediaCommand;
use crate::commands::prepare_guest_boot_command::{FirmwarePreference, PrepareGuestBootCommand};
use crate::domain::guest_boot::{
    BootDevice, BootOrder, FirmwareSelection, GuestBootConfiguration, GuestBootDomainError,
    IsoAttachment, MediaId,
};
use crate::domain::virtual_machine::{VirtualMachine, VmDomainError, VmId};
use crate::domain::vm_state::VmState;
use crate::ports::firmware_port::{FirmwareError, FirmwarePort, PreparedUefiFirmware};
use crate::ports::media_port::{MediaError, MediaImportState, MediaPort};
use crate::ports::vm_repository_port::{VmRepositoryError, VmRepositoryPort};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuestBootServiceError {
    VmDomain(VmDomainError),
    GuestDomain(GuestBootDomainError),
    Repository(VmRepositoryError),
    Media(MediaError),
    Firmware(FirmwareError),
    VmMustBeStopped(VmState),
}

pub struct GuestBootService<R, M, F>
where
    R: VmRepositoryPort,
    M: MediaPort,
    F: FirmwarePort,
{
    repository: R,
    media: M,
    firmware: F,
}

impl<R, M, F> GuestBootService<R, M, F>
where
    R: VmRepositoryPort,
    M: MediaPort,
    F: FirmwarePort,
{
    pub const fn new(repository: R, media: M, firmware: F) -> Self {
        Self {
            repository,
            media,
            firmware,
        }
    }

    pub fn prepare_guest_boot(
        &mut self,
        command: PrepareGuestBootCommand,
    ) -> Result<VirtualMachine, GuestBootServiceError> {
        let vm_id = VmId::parse(command.vm_id).map_err(GuestBootServiceError::VmDomain)?;
        let mut machine = self
            .repository
            .get(&vm_id)
            .map_err(GuestBootServiceError::Repository)?;
        Self::require_stopped(&machine)?;

        let boot_order = BootOrder::create(command.boot_devices, command.boot_once)
            .map_err(GuestBootServiceError::GuestDomain)?;

        let installer_iso = command
            .installer_iso
            .map(|iso| {
                let media_id = MediaId::parse(iso.media_id)
                    .map_err(GuestBootServiceError::GuestDomain)?;
                let attachment = IsoAttachment::create(media_id, iso.relative_path)
                    .map_err(GuestBootServiceError::GuestDomain)?;
                Ok((iso.source_path, attachment))
            })
            .transpose()?;

        let media_import_state = if let Some((source_path, attachment)) = &installer_iso {
            Some(
                self.media
                    .import_iso(&vm_id, source_path, attachment)
                    .map_err(GuestBootServiceError::Media)?,
            )
        } else {
            None
        };

        let prepared_uefi = match command.firmware {
            FirmwarePreference::Bios => None,
            FirmwarePreference::Uefi => match self.firmware.prepare_uefi(&vm_id) {
                Ok(prepared) => Some(prepared),
                Err(error) => {
                    Self::rollback_prepared(
                        &self.media,
                        &self.firmware,
                        &vm_id,
                        &installer_iso,
                        media_import_state,
                        None,
                    );
                    return Err(GuestBootServiceError::Firmware(error));
                }
            },
        };
        let firmware = prepared_uefi
            .as_ref()
            .map(|prepared| FirmwareSelection::Uefi(prepared.firmware.clone()))
            .unwrap_or(FirmwareSelection::Bios);

        let configuration = GuestBootConfiguration::create(
            command.profile,
            firmware,
            boot_order,
            installer_iso
                .as_ref()
                .map(|(_, attachment)| attachment.clone()),
            machine.guest_boot().catalog_template_id().map(str::to_owned),
        )
        .map_err(|error| {
            Self::rollback_prepared(
                &self.media,
                &self.firmware,
                &vm_id,
                &installer_iso,
                media_import_state,
                prepared_uefi.as_ref(),
            );
            GuestBootServiceError::GuestDomain(error)
        })?;

        machine.configure_guest_boot(configuration);

        if let Err(error) = self.repository.save(machine.clone()) {
            Self::rollback_prepared(
                &self.media,
                &self.firmware,
                &vm_id,
                &installer_iso,
                media_import_state,
                prepared_uefi.as_ref(),
            );
            return Err(GuestBootServiceError::Repository(error));
        }

        if let (Some((_, attachment)), Some(state)) = (&installer_iso, media_import_state) {
            let _ = self.media.commit_iso(&vm_id, attachment, state);
        }

        Ok(machine)
    }

    pub fn eject_installer_media(
        &mut self,
        command: EjectInstallerMediaCommand,
    ) -> Result<VirtualMachine, GuestBootServiceError> {
        let vm_id = VmId::parse(command.vm_id).map_err(GuestBootServiceError::VmDomain)?;
        let mut machine = self
            .repository
            .get(&vm_id)
            .map_err(GuestBootServiceError::Repository)?;
        Self::require_stopped(&machine)?;

        let boot_order = BootOrder::create(vec![BootDevice::Disk], false)
            .map_err(GuestBootServiceError::GuestDomain)?;
        let configuration = GuestBootConfiguration::create(
            machine.guest_boot().profile(),
            machine.guest_boot().firmware().clone(),
            boot_order,
            None,
            machine.guest_boot().catalog_template_id().map(str::to_owned),
        )
        .map_err(GuestBootServiceError::GuestDomain)?;

        machine.configure_guest_boot(configuration);
        self.repository
            .save(machine.clone())
            .map_err(GuestBootServiceError::Repository)?;
        Ok(machine)
    }

    pub fn into_parts(self) -> (R, M, F) {
        (self.repository, self.media, self.firmware)
    }

    fn require_stopped(machine: &VirtualMachine) -> Result<(), GuestBootServiceError> {
        if machine.state() != VmState::Stopped {
            return Err(GuestBootServiceError::VmMustBeStopped(machine.state()));
        }
        Ok(())
    }

    fn rollback_prepared(
        media: &M,
        firmware_port: &F,
        vm_id: &VmId,
        installer_iso: &Option<(std::path::PathBuf, IsoAttachment)>,
        media_import_state: Option<MediaImportState>,
        prepared_uefi: Option<&PreparedUefiFirmware>,
    ) {
        if let (Some((_, attachment)), Some(state)) = (installer_iso, media_import_state) {
            let _ = media.rollback_iso(vm_id, attachment, state);
        }
        if let Some(prepared) = prepared_uefi {
            let _ = firmware_port.cleanup_uefi(vm_id, prepared);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::path::Path;

    use super::*;
    use crate::commands::eject_installer_media_command::EjectInstallerMediaCommand;
    use crate::commands::prepare_guest_boot_command::InstallerIsoCommand;
    use crate::domain::guest_boot::{GuestProfile, UefiFirmware};
    use crate::domain::hypervisor::AccelerationBackend;
    use crate::domain::virtual_machine::VmResourceConfig;

    #[derive(Default)]
    struct FakeRepository {
        machines: HashMap<VmId, VirtualMachine>,
    }

    impl VmRepositoryPort for FakeRepository {
        fn insert(&mut self, machine: VirtualMachine) -> Result<(), VmRepositoryError> {
            self.machines.insert(machine.id().clone(), machine);
            Ok(())
        }

        fn get(&self, vm_id: &VmId) -> Result<VirtualMachine, VmRepositoryError> {
            self.machines
                .get(vm_id)
                .cloned()
                .ok_or_else(|| VmRepositoryError::NotFound(vm_id.clone()))
        }

        fn list(&self) -> Result<Vec<VirtualMachine>, VmRepositoryError> {
            Ok(self.machines.values().cloned().collect())
        }

        fn save(&mut self, machine: VirtualMachine) -> Result<(), VmRepositoryError> {
            self.machines.insert(machine.id().clone(), machine);
            Ok(())
        }

        fn delete(&mut self, vm_id: &VmId) -> Result<(), VmRepositoryError> {
            self.machines
                .remove(vm_id)
                .map(|_| ())
                .ok_or_else(|| VmRepositoryError::NotFound(vm_id.clone()))
        }
    }

    #[derive(Default)]
    struct FakeMedia {
        imported: RefCell<Vec<String>>,
    }

    impl MediaPort for FakeMedia {
        fn import_iso(
            &self,
            _vm_id: &VmId,
            _source_path: &Path,
            attachment: &IsoAttachment,
        ) -> Result<MediaImportState, MediaError> {
            self.imported
                .borrow_mut()
                .push(attachment.relative_path().to_owned());
            Ok(MediaImportState::Created)
        }

        fn commit_iso(
            &self,
            _vm_id: &VmId,
            _attachment: &IsoAttachment,
            _state: MediaImportState,
        ) -> Result<(), MediaError> {
            Ok(())
        }

        fn rollback_iso(
            &self,
            _vm_id: &VmId,
            attachment: &IsoAttachment,
            _state: MediaImportState,
        ) -> Result<(), MediaError> {
            self.imported
                .borrow_mut()
                .retain(|path| path != attachment.relative_path());
            Ok(())
        }

        fn delete_iso(&self, _vm_id: &VmId, attachment: &IsoAttachment) -> Result<(), MediaError> {
            self.imported
                .borrow_mut()
                .retain(|path| path != attachment.relative_path());
            Ok(())
        }
    }

    struct FakeFirmware;

    impl FirmwarePort for FakeFirmware {
        fn prepare_uefi(&self, _vm_id: &VmId) -> Result<PreparedUefiFirmware, FirmwareError> {
            let firmware = UefiFirmware::create(
                "firmware/OVMF_CODE.fd",
                "firmware/OVMF_VARS.fd",
            )
            .map_err(|error| FirmwareError::PrepareFailed(format!("{error:?}")))?;
            Ok(PreparedUefiFirmware {
                firmware,
                vars_created: true,
            })
        }

        fn cleanup_uefi(
            &self,
            _vm_id: &VmId,
            _prepared: &PreparedUefiFirmware,
        ) -> Result<(), FirmwareError> {
            Ok(())
        }
    }

    fn stopped_machine() -> VirtualMachine {
        let mut machine = VirtualMachine::create(
            VmId::parse("guest-test").expect("vm id must be valid"),
            "Guest Test",
            VmResourceConfig {
                vcpu_count: 4,
                memory_mib: 4096,
            },
            AccelerationBackend::Tcg,
        )
        .expect("vm must be valid");
        machine
            .transition_to(VmState::Stopped)
            .expect("created vm must stop");
        machine
    }

    #[test]
    fn prepares_linux_uefi_installer_boot() {
        let mut repository = FakeRepository::default();
        repository
            .insert(stopped_machine())
            .expect("insert must succeed");
        let media = FakeMedia::default();
        let firmware = FakeFirmware;
        let mut service = GuestBootService::new(repository, media, firmware);

        let machine = service
            .prepare_guest_boot(PrepareGuestBootCommand::new(
                "guest-test",
                GuestProfile::Linux,
                FirmwarePreference::Uefi,
                vec![BootDevice::Cdrom, BootDevice::Disk],
                true,
                Some(InstallerIsoCommand::new(
                    "installer",
                    std::path::PathBuf::from("ubuntu.iso"),
                    "media/installer.iso",
                )),
            ))
            .expect("guest boot preparation must succeed");

        assert_eq!(machine.guest_boot().profile(), GuestProfile::Linux);
        assert!(matches!(
            machine.guest_boot().firmware(),
            FirmwareSelection::Uefi(_)
        ));
        assert!(machine.guest_boot().installer_iso().is_some());
        assert!(machine.guest_boot().boot_order().apply_once());
    }

    #[test]
    fn ejects_installer_without_deleting_iso_and_persists_disk_boot() {
        let mut repository = FakeRepository::default();
        repository
            .insert(stopped_machine())
            .expect("insert must succeed");
        let media = FakeMedia::default();
        let firmware = FakeFirmware;
        let mut service = GuestBootService::new(repository, media, firmware);

        service
            .prepare_guest_boot(PrepareGuestBootCommand::new(
                "guest-test",
                GuestProfile::Linux,
                FirmwarePreference::Bios,
                vec![BootDevice::Cdrom, BootDevice::Disk],
                true,
                Some(InstallerIsoCommand::new(
                    "installer",
                    std::path::PathBuf::from("debian.iso"),
                    "media/installer.iso",
                )),
            ))
            .expect("installer preparation must succeed");

        let machine = service
            .eject_installer_media(EjectInstallerMediaCommand::new("guest-test"))
            .expect("installer media eject must succeed");

        assert!(machine.guest_boot().installer_iso().is_none());
        assert_eq!(machine.guest_boot().boot_order().devices(), &[BootDevice::Disk]);
        assert!(!machine.guest_boot().boot_order().apply_once());
        assert_eq!(machine.guest_boot().profile(), GuestProfile::Linux);
        let (_, media, _) = service.into_parts();
        assert_eq!(media.imported.borrow().len(), 1);
    }
}
