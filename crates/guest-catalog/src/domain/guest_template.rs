// # 📄 Dosya Yolu: /turkuazvm/crates/guest-catalog/src/domain/guest_template.rs
// # 📌 Amac: VM kurulum katalogundaki OS template, onerilen kaynak ve installer medya modellerini tanimlar
// # 📌 Modul - Rust
// # Version: 0.40.1
// # Aciklama: Installer medyasini turune gore siniflandirir ve resolver tarafindan yonetilebilen medya kabiliyetini typed domain olarak sunar
// # Bagimli Oldugu Katman: Service | Repo | View

const MAX_ID_LEN: usize = 120;
const MAX_LABEL_LEN: usize = 160;
const MAX_URL_LEN: usize = 2048;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GuestFamily {
    Windows,
    Linux,
    Android,
    Other,
}

impl GuestFamily {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Windows => "windows",
            Self::Linux => "linux",
            Self::Android => "android",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GuestProfileHint {
    Generic,
    Linux,
    Windows,
    Android,
}

impl GuestProfileHint {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Generic => "generic",
            Self::Linux => "linux",
            Self::Windows => "windows",
            Self::Android => "android",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GuestSourceKind {
    Iso,
    AndroidImage,
    Manual,
}

impl GuestSourceKind {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Iso => "iso",
            Self::AndroidImage => "android_image",
            Self::Manual => "manual",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GuestFirmwareHint {
    Bios,
    Uefi,
    QemuBootloader,
}

impl GuestFirmwareHint {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Bios => "bios",
            Self::Uefi => "uefi",
            Self::QemuBootloader => "qemu_bootloader",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstallerMediaMode {
    Direct,
    OfficialPage,
}

impl InstallerMediaMode {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::OfficialPage => "official_page",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstallerMediaKind {
    Unknown,
    DesktopLive,
    ServerStandard,
    NetworkInstall,
    Boot,
    Minimal,
    Dvd,
    OfficialPage,
}

impl InstallerMediaKind {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::DesktopLive => "desktop_live",
            Self::ServerStandard => "server_standard",
            Self::NetworkInstall => "network_install",
            Self::Boot => "boot",
            Self::Minimal => "minimal",
            Self::Dvd => "dvd",
            Self::OfficialPage => "official_page",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallerMediaSource {
    pub id: String,
    pub mode: InstallerMediaMode,
    pub media_kind: InstallerMediaKind,
    pub provider: String,
    pub label: String,
    pub architecture: String,
    pub recommended: bool,
    pub url: String,
    pub filename: Option<String>,
    pub checksum_url: Option<String>,
    pub size_bytes: Option<u64>,
    pub note: String,
}

impl InstallerMediaSource {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        id: String,
        mode: InstallerMediaMode,
        media_kind: InstallerMediaKind,
        provider: String,
        label: String,
        architecture: String,
        recommended: bool,
        url: String,
        filename: Option<String>,
        checksum_url: Option<String>,
        size_bytes: Option<u64>,
        note: String,
    ) -> Result<Self, GuestCatalogDomainError> {
        validate_id(&id)?;
        validate_label(&provider)?;
        validate_label(&label)?;
        validate_label(&architecture)?;
        validate_https_url(&url)?;
        if let Some(checksum_url) = checksum_url.as_deref() {
            validate_https_url(checksum_url)?;
        }
        if let Some(filename) = filename.as_deref() {
            validate_iso_filename(filename)?;
        }
        if matches!(mode, InstallerMediaMode::Direct) && filename.is_none() {
            return Err(GuestCatalogDomainError::InvalidInstallerMedia);
        }
        if size_bytes == Some(0) {
            return Err(GuestCatalogDomainError::InvalidInstallerMedia);
        }
        Ok(Self {
            id,
            mode,
            media_kind,
            provider,
            label,
            architecture,
            recommended,
            url,
            filename,
            checksum_url,
            size_bytes,
            note: note.trim().to_owned(),
        })
    }

    pub fn managed_download_supported(&self) -> bool {
        match self.mode {
            InstallerMediaMode::Direct => true,
            InstallerMediaMode::OfficialPage => !matches!(
                self.media_kind,
                InstallerMediaKind::Unknown | InstallerMediaKind::OfficialPage
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecommendedVmResources {
    pub vcpu_count: u16,
    pub memory_mib: u64,
    pub disk_size_gib: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuestTemplate {
    pub id: String,
    pub family: GuestFamily,
    pub family_label: String,
    pub product_id: String,
    pub product_label: String,
    pub release_id: String,
    pub release_label: String,
    pub profile_id: String,
    pub profile_label: String,
    pub guest_profile: GuestProfileHint,
    pub architecture: String,
    pub source_kind: GuestSourceKind,
    pub source_label: String,
    pub firmware: GuestFirmwareHint,
    pub recommended: RecommendedVmResources,
    pub installer_media: Option<InstallerMediaSource>,
    pub installer_media_options: Vec<InstallerMediaSource>,
    pub description: String,
}

impl GuestTemplate {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        id: String,
        family: GuestFamily,
        family_label: String,
        product_id: String,
        product_label: String,
        release_id: String,
        release_label: String,
        profile_id: String,
        profile_label: String,
        guest_profile: GuestProfileHint,
        architecture: String,
        source_kind: GuestSourceKind,
        source_label: String,
        firmware: GuestFirmwareHint,
        recommended: RecommendedVmResources,
        installer_media: Option<InstallerMediaSource>,
        installer_media_options: Vec<InstallerMediaSource>,
        description: String,
    ) -> Result<Self, GuestCatalogDomainError> {
        for value in [&id, &product_id, &release_id, &profile_id] {
            validate_id(value)?;
        }
        for value in [
            &family_label,
            &product_label,
            &release_label,
            &profile_label,
            &architecture,
            &source_label,
        ] {
            validate_label(value)?;
        }
        if recommended.vcpu_count == 0
            || recommended.memory_mib < 512
            || recommended.disk_size_gib == 0
        {
            return Err(GuestCatalogDomainError::InvalidRecommendedResources);
        }
        if !matches!(source_kind, GuestSourceKind::Iso | GuestSourceKind::Manual)
            && (installer_media.is_some() || !installer_media_options.is_empty())
        {
            return Err(GuestCatalogDomainError::InvalidInstallerMedia);
        }
        let mut media_ids = std::collections::HashSet::new();
        if let Some(media) = installer_media.as_ref() {
            if !media_ids.insert(media.id.as_str()) {
                return Err(GuestCatalogDomainError::InvalidInstallerMedia);
            }
        }
        for media in &installer_media_options {
            if !media_ids.insert(media.id.as_str()) {
                return Err(GuestCatalogDomainError::InvalidInstallerMedia);
            }
        }
        Ok(Self {
            id,
            family,
            family_label,
            product_id,
            product_label,
            release_id,
            release_label,
            profile_id,
            profile_label,
            guest_profile,
            architecture,
            source_kind,
            source_label,
            firmware,
            recommended,
            installer_media,
            installer_media_options,
            description: description.trim().to_owned(),
        })
    }

    pub fn installer_media_source(&self, media_id: &str) -> Option<&InstallerMediaSource> {
        self.installer_media
            .iter()
            .chain(self.installer_media_options.iter())
            .find(|media| media.id == media_id)
    }

    pub fn installer_media_sources(&self) -> impl Iterator<Item = &InstallerMediaSource> {
        self.installer_media.iter().chain(self.installer_media_options.iter())
    }
}

fn validate_id(value: &str) -> Result<(), GuestCatalogDomainError> {
    let valid = !value.is_empty()
        && value.len() <= MAX_ID_LEN
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'));
    valid
        .then_some(())
        .ok_or(GuestCatalogDomainError::InvalidIdentifier)
}

fn validate_label(value: &str) -> Result<(), GuestCatalogDomainError> {
    let trimmed = value.trim();
    (!trimmed.is_empty() && trimmed.len() <= MAX_LABEL_LEN)
        .then_some(())
        .ok_or(GuestCatalogDomainError::InvalidLabel)
}

fn validate_https_url(value: &str) -> Result<(), GuestCatalogDomainError> {
    let trimmed = value.trim();
    (trimmed.starts_with("https://") && trimmed.len() <= MAX_URL_LEN && !trimmed.contains(char::is_whitespace))
        .then_some(())
        .ok_or(GuestCatalogDomainError::InvalidInstallerMedia)
}

fn validate_iso_filename(value: &str) -> Result<(), GuestCatalogDomainError> {
    let valid = value.len() <= 180
        && value.to_ascii_lowercase().ends_with(".iso")
        && !value.contains('/')
        && !value.contains('\\')
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'));
    valid
        .then_some(())
        .ok_or(GuestCatalogDomainError::InvalidInstallerMedia)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuestCatalogDomainError {
    InvalidIdentifier,
    InvalidLabel,
    InvalidRecommendedResources,
    InvalidInstallerMedia,
}
