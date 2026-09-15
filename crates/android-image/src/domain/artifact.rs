// # 📄 Dosya Yolu: /turkuazvm/crates/android-image/src/domain/artifact.rs
// # 📌 Amac: Android image bundle artifact rollerini ve metadata bilgisini tanimlar
// # 📌 Modul - Rust
// # Version: 0.40.13
// # Aciklama: Cuttlefish ve Android SDK Emulator bundle artifactlarini runtime tipinden bagimsiz typed rollerle dogrular
// # Bagimli Oldugu Katman: Service | Repo | Tool | View

use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AndroidImageArtifactRole {
    Boot,
    InitBoot,
    VendorBoot,
    Super,
    Userdata,
    Vbmeta,
    VbmetaSystem,
    Metadata,
    Misc,
    Bootloader,
    CompositeDisk,
    Kernel,
    Ramdisk,
    System,
    Other,
}

impl AndroidImageArtifactRole {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Boot => "boot",
            Self::InitBoot => "init_boot",
            Self::VendorBoot => "vendor_boot",
            Self::Super => "super",
            Self::Userdata => "userdata",
            Self::Vbmeta => "vbmeta",
            Self::VbmetaSystem => "vbmeta_system",
            Self::Metadata => "metadata",
            Self::Misc => "misc",
            Self::Bootloader => "bootloader",
            Self::CompositeDisk => "composite_disk",
            Self::Kernel => "kernel",
            Self::Ramdisk => "ramdisk",
            Self::System => "system",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidImageArtifact {
    pub role: AndroidImageArtifactRole,
    pub relative_path: String,
    pub size_bytes: u64,
    pub sha256: String,
}

impl AndroidImageArtifact {
    pub fn create(
        role: AndroidImageArtifactRole,
        relative_path: String,
        size_bytes: u64,
        sha256: String,
    ) -> Result<Self, AndroidImageArtifactError> {
        if relative_path.trim().is_empty() || relative_path.starts_with('/') || relative_path.contains("..") {
            return Err(AndroidImageArtifactError::InvalidRelativePath);
        }
        if size_bytes == 0 {
            return Err(AndroidImageArtifactError::EmptyArtifact);
        }
        if sha256.len() != 64 || !sha256.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(AndroidImageArtifactError::InvalidSha256);
        }
        Ok(Self { role, relative_path, size_bytes, sha256: sha256.to_ascii_lowercase() })
    }
}

pub fn validate_boot_candidate(artifacts: &[AndroidImageArtifact]) -> Result<(), AndroidImageArtifactError> {
    let roles = artifacts.iter().map(|artifact| artifact.role).collect::<HashSet<_>>();
    if !roles.contains(&AndroidImageArtifactRole::Boot) {
        return Err(AndroidImageArtifactError::MissingBootArtifact);
    }
    if !roles.contains(&AndroidImageArtifactRole::Super) {
        return Err(AndroidImageArtifactError::MissingSuperArtifact);
    }
    if !roles.contains(&AndroidImageArtifactRole::Userdata) {
        return Err(AndroidImageArtifactError::MissingUserdataArtifact);
    }
    if !roles.contains(&AndroidImageArtifactRole::Bootloader) {
        return Err(AndroidImageArtifactError::MissingBootloaderArtifact);
    }
    if !roles.contains(&AndroidImageArtifactRole::CompositeDisk) {
        return Err(AndroidImageArtifactError::MissingCompositeDiskArtifact);
    }
    Ok(())
}

pub fn validate_sdk_emulator_candidate(artifacts: &[AndroidImageArtifact]) -> Result<(), AndroidImageArtifactError> {
    let roles = artifacts.iter().map(|artifact| artifact.role).collect::<HashSet<_>>();
    if !roles.contains(&AndroidImageArtifactRole::Kernel) {
        return Err(AndroidImageArtifactError::MissingKernelArtifact);
    }
    if !roles.contains(&AndroidImageArtifactRole::Ramdisk) {
        return Err(AndroidImageArtifactError::MissingRamdiskArtifact);
    }
    if !roles.contains(&AndroidImageArtifactRole::System) {
        return Err(AndroidImageArtifactError::MissingSystemArtifact);
    }
    if !roles.contains(&AndroidImageArtifactRole::Userdata) {
        return Err(AndroidImageArtifactError::MissingUserdataArtifact);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndroidImageArtifactError {
    InvalidRelativePath,
    EmptyArtifact,
    InvalidSha256,
    MissingBootArtifact,
    MissingSuperArtifact,
    MissingUserdataArtifact,
    MissingBootloaderArtifact,
    MissingCompositeDiskArtifact,
    MissingKernelArtifact,
    MissingRamdiskArtifact,
    MissingSystemArtifact,
}
