// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/domain/guest_boot.rs
// # 📌 Amac: Guest OS boot, ISO media, firmware ve boot order domain modellerini tanimlar
// # 📌 Modul - Rust
// # Version: 0.40.3
// # Aciklama: BIOS/UEFI, installer ISO, boot sirasi ve tek-oturumluk kurulum medyasi yasam dongusu invariantlarini QEMU detaylarindan bagimsiz tutar
// # Bagimli Oldugu Katman: Service | Repo

use std::path::{Component, Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuestProfile {
    Generic,
    Linux,
    Windows,
    Android,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BootDevice {
    Disk,
    Cdrom,
    Network,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootOrder {
    devices: Vec<BootDevice>,
    apply_once: bool,
}

impl BootOrder {
    pub fn create(devices: Vec<BootDevice>, apply_once: bool) -> Result<Self, GuestBootDomainError> {
        if devices.is_empty() {
            return Err(GuestBootDomainError::EmptyBootOrder);
        }

        for (index, device) in devices.iter().enumerate() {
            if devices[..index].contains(device) {
                return Err(GuestBootDomainError::DuplicateBootDevice(*device));
            }
        }

        Ok(Self {
            devices,
            apply_once,
        })
    }

    pub fn devices(&self) -> &[BootDevice] {
        &self.devices
    }

    pub const fn apply_once(&self) -> bool {
        self.apply_once
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MediaId(String);

impl MediaId {
    pub fn parse(value: impl Into<String>) -> Result<Self, GuestBootDomainError> {
        let value = value.into();
        let valid = !value.is_empty()
            && value.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
            });
        if !valid {
            return Err(GuestBootDomainError::InvalidMediaId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IsoAttachment {
    id: MediaId,
    relative_path: String,
}

impl IsoAttachment {
    pub fn create(
        id: MediaId,
        relative_path: impl Into<String>,
    ) -> Result<Self, GuestBootDomainError> {
        let relative_path = relative_path.into();
        validate_relative_path(&relative_path)?;
        if !relative_path.to_ascii_lowercase().ends_with(".iso") {
            return Err(GuestBootDomainError::IsoExtensionRequired);
        }
        Ok(Self { id, relative_path })
    }

    pub fn id(&self) -> &MediaId {
        &self.id
    }

    pub fn relative_path(&self) -> &str {
        &self.relative_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UefiFirmware {
    code_relative_path: String,
    vars_relative_path: String,
}

impl UefiFirmware {
    pub fn create(
        code_relative_path: impl Into<String>,
        vars_relative_path: impl Into<String>,
    ) -> Result<Self, GuestBootDomainError> {
        let code_relative_path = code_relative_path.into();
        let vars_relative_path = vars_relative_path.into();
        validate_relative_path(&code_relative_path)?;
        validate_relative_path(&vars_relative_path)?;
        Ok(Self {
            code_relative_path,
            vars_relative_path,
        })
    }

    pub fn code_relative_path(&self) -> &str {
        &self.code_relative_path
    }

    pub fn vars_relative_path(&self) -> &str {
        &self.vars_relative_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FirmwareSelection {
    Bios,
    Uefi(UefiFirmware),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuestBootConfiguration {
    profile: GuestProfile,
    firmware: FirmwareSelection,
    boot_order: BootOrder,
    installer_iso: Option<IsoAttachment>,
    catalog_template_id: Option<String>,
}

impl GuestBootConfiguration {
    pub fn create(
        profile: GuestProfile,
        firmware: FirmwareSelection,
        boot_order: BootOrder,
        installer_iso: Option<IsoAttachment>,
        catalog_template_id: Option<String>,
    ) -> Result<Self, GuestBootDomainError> {
        let has_cdrom = boot_order.devices().contains(&BootDevice::Cdrom);
        if has_cdrom && installer_iso.is_none() {
            return Err(GuestBootDomainError::CdromBootRequiresIso);
        }
        if installer_iso.is_some() && !has_cdrom {
            return Err(GuestBootDomainError::InstallerIsoRequiresCdromBoot);
        }
        if let Some(value) = catalog_template_id.as_deref() {
            let valid = !value.is_empty() && value.len() <= 120 && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'));
            if !valid {
                return Err(GuestBootDomainError::InvalidCatalogTemplateId);
            }
        }
        Ok(Self {
            profile,
            firmware,
            boot_order,
            installer_iso,
            catalog_template_id,
        })
    }

    pub const fn profile(&self) -> GuestProfile {
        self.profile
    }

    pub const fn firmware(&self) -> &FirmwareSelection {
        &self.firmware
    }

    pub const fn boot_order(&self) -> &BootOrder {
        &self.boot_order
    }

    pub const fn installer_iso(&self) -> Option<&IsoAttachment> {
        self.installer_iso.as_ref()
    }

    pub fn catalog_template_id(&self) -> Option<&str> {
        self.catalog_template_id.as_deref()
    }

    pub fn expire_one_shot_installer_media(&mut self) -> Result<bool, GuestBootDomainError> {
        if self.installer_iso.is_none() || !self.boot_order.apply_once() {
            return Ok(false);
        }

        let mut next_devices = self
            .boot_order
            .devices()
            .iter()
            .copied()
            .filter(|device| *device != BootDevice::Cdrom)
            .collect::<Vec<_>>();
        if next_devices.is_empty() {
            next_devices.push(BootDevice::Disk);
        }

        self.boot_order = BootOrder::create(next_devices, false)?;
        self.installer_iso = None;
        Ok(true)
    }
}

impl GuestBootConfiguration {
    pub fn default_for_profile(profile: GuestProfile, catalog_template_id: Option<String>) -> Result<Self, GuestBootDomainError> {
        if let Some(value) = catalog_template_id.as_deref() {
            let valid = !value.is_empty() && value.len() <= 120 && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'));
            if !valid {
                return Err(GuestBootDomainError::InvalidCatalogTemplateId);
            }
        }
        Ok(Self {
            profile,
            firmware: FirmwareSelection::Bios,
            boot_order: BootOrder {
                devices: vec![BootDevice::Disk],
                apply_once: false,
            },
            installer_iso: None,
            catalog_template_id,
        })
    }
}

impl Default for GuestBootConfiguration {
    fn default() -> Self {
        Self {
            profile: GuestProfile::Generic,
            firmware: FirmwareSelection::Bios,
            boot_order: BootOrder {
                devices: vec![BootDevice::Disk],
                apply_once: false,
            },
            installer_iso: None,
            catalog_template_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuestBootDomainError {
    InvalidCatalogTemplateId,
    InvalidMediaId,
    InvalidRelativePath(String),
    IsoExtensionRequired,
    EmptyBootOrder,
    DuplicateBootDevice(BootDevice),
    CdromBootRequiresIso,
    InstallerIsoRequiresCdromBoot,
}

fn validate_relative_path(value: &str) -> Result<(), GuestBootDomainError> {
    if value.trim().is_empty() {
        return Err(GuestBootDomainError::InvalidRelativePath(value.to_owned()));
    }

    let path = Path::new(value);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(GuestBootDomainError::InvalidRelativePath(value.to_owned()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cdrom_boot_requires_iso() {
        let order = BootOrder::create(vec![BootDevice::Cdrom, BootDevice::Disk], true)
            .expect("boot order must be valid");
        assert_eq!(
            GuestBootConfiguration::create(
                GuestProfile::Linux,
                FirmwareSelection::Bios,
                order,
                None,
                None,
            ),
            Err(GuestBootDomainError::CdromBootRequiresIso)
        );
    }

    #[test]
    fn installer_iso_path_must_be_relative_iso() {
        let id = MediaId::parse("installer").expect("media id must be valid");
        assert!(IsoAttachment::create(id.clone(), "../installer.iso").is_err());
        assert!(IsoAttachment::create(id.clone(), "media/installer.img").is_err());
        assert!(IsoAttachment::create(id, "media/installer.iso").is_ok());
    }

    #[test]
    fn one_shot_installer_expires_to_disk_boot_after_runtime_session() {
        let order = BootOrder::create(vec![BootDevice::Cdrom, BootDevice::Disk], true)
            .expect("boot order must be valid");
        let installer = IsoAttachment::create(
            MediaId::parse("installer").expect("media id must be valid"),
            "media/installer.iso",
        )
        .expect("installer attachment must be valid");
        let mut configuration = GuestBootConfiguration::create(
            GuestProfile::Linux,
            FirmwareSelection::Bios,
            order,
            Some(installer),
            Some(String::from("fedora-44-server")),
        )
        .expect("guest boot configuration must be valid");

        assert!(configuration
            .expire_one_shot_installer_media()
            .expect("one-shot media expiry must succeed"));
        assert!(configuration.installer_iso().is_none());
        assert_eq!(configuration.boot_order().devices(), &[BootDevice::Disk]);
        assert!(!configuration.boot_order().apply_once());
        assert_eq!(configuration.catalog_template_id(), Some("fedora-44-server"));
    }

    #[test]
    fn persistent_boot_configuration_is_not_modified_by_session_expiry() {
        let order = BootOrder::create(vec![BootDevice::Cdrom, BootDevice::Disk], false)
            .expect("boot order must be valid");
        let installer = IsoAttachment::create(
            MediaId::parse("installer").expect("media id must be valid"),
            "media/installer.iso",
        )
        .expect("installer attachment must be valid");
        let mut configuration = GuestBootConfiguration::create(
            GuestProfile::Linux,
            FirmwareSelection::Bios,
            order,
            Some(installer),
            None,
        )
        .expect("guest boot configuration must be valid");

        assert!(!configuration
            .expire_one_shot_installer_media()
            .expect("persistent media check must succeed"));
        assert!(configuration.installer_iso().is_some());
        assert_eq!(
            configuration.boot_order().devices(),
            &[BootDevice::Cdrom, BootDevice::Disk]
        );
    }

    #[test]
    fn duplicate_boot_device_is_rejected() {
        assert_eq!(
            BootOrder::create(vec![BootDevice::Disk, BootDevice::Disk], false),
            Err(GuestBootDomainError::DuplicateBootDevice(BootDevice::Disk))
        );
    }
}
