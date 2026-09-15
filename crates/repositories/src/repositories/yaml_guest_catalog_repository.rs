// # 📄 Dosya Yolu: /turkuazvm/crates/repositories/src/repositories/yaml_guest_catalog_repository.rs
// # 📌 Amac: Guest Catalog kayitlarini versioned YAML dosyasindan yukler
// # 📌 Modul - Rust
// # Version: 0.39.4
// # Aciklama: Guest template, installer medya turu ve Linux Desktop/Server tercih politikasini YAML storage katmanindan typed domain modeline donusturur
// # Bagimli Oldugu Katman: Repo

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use turkuazvm_guest_catalog::domain::guest_template::{
    GuestFamily, GuestFirmwareHint, GuestProfileHint, GuestSourceKind, GuestTemplate,
    InstallerMediaKind, InstallerMediaMode, InstallerMediaSource, RecommendedVmResources,
};
use turkuazvm_guest_catalog::domain::linux_media_policy::LinuxMediaPolicy;
use turkuazvm_guest_catalog::ports::guest_catalog_repository_port::{
    GuestCatalogRepositoryError, GuestCatalogRepositoryPort,
};

const CATALOG_SCHEMA_VERSION: u16 = 4;

#[derive(Debug, Clone)]
pub struct YamlGuestCatalogRepository {
    path: PathBuf,
}

impl YamlGuestCatalogRepository {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

#[derive(Debug, Deserialize)]
struct CatalogManifest {
    schema_version: u16,
    #[serde(default)]
    linux_media_policies: Vec<LinuxMediaPolicyManifest>,
    templates: Vec<TemplateManifest>,
}

#[derive(Debug, Deserialize)]
struct TemplateManifest {
    id: String,
    family: GuestFamilyManifest,
    family_label: String,
    product_id: String,
    product_label: String,
    release_id: String,
    release_label: String,
    profile_id: String,
    profile_label: String,
    guest_profile: GuestProfileManifest,
    architecture: String,
    source_kind: GuestSourceManifest,
    source_label: String,
    firmware: GuestFirmwareManifest,
    recommended: RecommendedManifest,
    #[serde(default)]
    installer_media: Option<InstallerMediaManifest>,
    #[serde(default)]
    installer_media_options: Vec<InstallerMediaManifest>,
    #[serde(default)]
    description: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum GuestFamilyManifest {
    Windows,
    Linux,
    Android,
    Other,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum GuestProfileManifest {
    Generic,
    Linux,
    Windows,
    Android,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum GuestSourceManifest {
    Iso,
    AndroidImage,
    Manual,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum GuestFirmwareManifest {
    Bios,
    Uefi,
    QemuBootloader,
}


#[derive(Debug, Deserialize)]
struct InstallerMediaManifest {
    id: String,
    mode: InstallerMediaModeManifest,
    #[serde(default)]
    media_kind: InstallerMediaKindManifest,
    provider: String,
    label: String,
    architecture: String,
    #[serde(default)]
    recommended: bool,
    url: String,
    #[serde(default)]
    filename: Option<String>,
    #[serde(default)]
    checksum_url: Option<String>,
    #[serde(default)]
    size_bytes: Option<u64>,
    #[serde(default)]
    note: String,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
enum InstallerMediaKindManifest {
    #[default]
    Unknown,
    DesktopLive,
    ServerStandard,
    NetworkInstall,
    Boot,
    Minimal,
    Dvd,
    OfficialPage,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum InstallerMediaModeManifest {
    Direct,
    OfficialPage,
}

#[derive(Debug, Deserialize)]
struct LinuxMediaPolicyManifest {
    product_id: String,
    profile_id: String,
    preferred_media_kinds: Vec<InstallerMediaKindManifest>,
}

#[derive(Debug, Deserialize)]
struct RecommendedManifest {
    vcpu_count: u16,
    memory_mib: u64,
    disk_size_gib: u64,
}

impl GuestCatalogRepositoryPort for YamlGuestCatalogRepository {
    fn list(&self) -> Result<Vec<GuestTemplate>, GuestCatalogRepositoryError> {
        let content = fs::read_to_string(&self.path).map_err(storage)?;
        let manifest: CatalogManifest = serde_yaml_ng::from_str(&content).map_err(storage)?;
        if manifest.schema_version != CATALOG_SCHEMA_VERSION {
            return Err(GuestCatalogRepositoryError::Storage(format!(
                "Unsupported guest catalog schema: {}",
                manifest.schema_version
            )));
        }
        let templates = manifest
            .templates
            .into_iter()
            .map(to_domain)
            .collect::<Result<Vec<_>, _>>()?;
        validate_catalog(&templates)?;
        Ok(templates)
    }

    fn get(&self, id: &str) -> Result<GuestTemplate, GuestCatalogRepositoryError> {
        self.list()?
            .into_iter()
            .find(|template| template.id == id)
            .ok_or_else(|| GuestCatalogRepositoryError::NotFound(id.to_owned()))
    }

    fn linux_media_policies(&self) -> Result<Vec<LinuxMediaPolicy>, GuestCatalogRepositoryError> {
        let content = fs::read_to_string(&self.path).map_err(storage)?;
        let manifest: CatalogManifest = serde_yaml_ng::from_str(&content).map_err(storage)?;
        if manifest.schema_version != CATALOG_SCHEMA_VERSION {
            return Err(GuestCatalogRepositoryError::Storage(format!(
                "Unsupported guest catalog schema: {}",
                manifest.schema_version
            )));
        }
        manifest
            .linux_media_policies
            .into_iter()
            .map(to_linux_media_policy)
            .collect()
    }
}

fn to_domain(value: TemplateManifest) -> Result<GuestTemplate, GuestCatalogRepositoryError> {
    GuestTemplate::create(
        value.id,
        match value.family {
            GuestFamilyManifest::Windows => GuestFamily::Windows,
            GuestFamilyManifest::Linux => GuestFamily::Linux,
            GuestFamilyManifest::Android => GuestFamily::Android,
            GuestFamilyManifest::Other => GuestFamily::Other,
        },
        value.family_label,
        value.product_id,
        value.product_label,
        value.release_id,
        value.release_label,
        value.profile_id,
        value.profile_label,
        match value.guest_profile {
            GuestProfileManifest::Generic => GuestProfileHint::Generic,
            GuestProfileManifest::Linux => GuestProfileHint::Linux,
            GuestProfileManifest::Windows => GuestProfileHint::Windows,
            GuestProfileManifest::Android => GuestProfileHint::Android,
        },
        value.architecture,
        match value.source_kind {
            GuestSourceManifest::Iso => GuestSourceKind::Iso,
            GuestSourceManifest::AndroidImage => GuestSourceKind::AndroidImage,
            GuestSourceManifest::Manual => GuestSourceKind::Manual,
        },
        value.source_label,
        match value.firmware {
            GuestFirmwareManifest::Bios => GuestFirmwareHint::Bios,
            GuestFirmwareManifest::Uefi => GuestFirmwareHint::Uefi,
            GuestFirmwareManifest::QemuBootloader => GuestFirmwareHint::QemuBootloader,
        },
        RecommendedVmResources {
            vcpu_count: value.recommended.vcpu_count,
            memory_mib: value.recommended.memory_mib,
            disk_size_gib: value.recommended.disk_size_gib,
        },
        value.installer_media.map(to_installer_media).transpose()?,
        value
            .installer_media_options
            .into_iter()
            .map(to_installer_media)
            .collect::<Result<Vec<_>, _>>()?,
        value.description,
    )
    .map_err(|error| GuestCatalogRepositoryError::Storage(format!("{error:?}")))
}


fn to_installer_media(media: InstallerMediaManifest) -> Result<InstallerMediaSource, GuestCatalogRepositoryError> {
    InstallerMediaSource::create(
        media.id,
        match media.mode {
            InstallerMediaModeManifest::Direct => InstallerMediaMode::Direct,
            InstallerMediaModeManifest::OfficialPage => InstallerMediaMode::OfficialPage,
        },
        to_media_kind(media.media_kind),
        media.provider,
        media.label,
        media.architecture,
        media.recommended,
        media.url,
        media.filename,
        media.checksum_url,
        media.size_bytes,
        media.note,
    )
    .map_err(|error| GuestCatalogRepositoryError::Storage(format!("{error:?}")))
}

fn to_linux_media_policy(policy: LinuxMediaPolicyManifest) -> Result<LinuxMediaPolicy, GuestCatalogRepositoryError> {
    LinuxMediaPolicy::new(
        policy.product_id,
        policy.profile_id,
        policy.preferred_media_kinds.into_iter().map(to_media_kind).collect(),
    )
    .map_err(|error| GuestCatalogRepositoryError::Storage(format!("{error:?}")))
}

fn to_media_kind(kind: InstallerMediaKindManifest) -> InstallerMediaKind {
    match kind {
        InstallerMediaKindManifest::Unknown => InstallerMediaKind::Unknown,
        InstallerMediaKindManifest::DesktopLive => InstallerMediaKind::DesktopLive,
        InstallerMediaKindManifest::ServerStandard => InstallerMediaKind::ServerStandard,
        InstallerMediaKindManifest::NetworkInstall => InstallerMediaKind::NetworkInstall,
        InstallerMediaKindManifest::Boot => InstallerMediaKind::Boot,
        InstallerMediaKindManifest::Minimal => InstallerMediaKind::Minimal,
        InstallerMediaKindManifest::Dvd => InstallerMediaKind::Dvd,
        InstallerMediaKindManifest::OfficialPage => InstallerMediaKind::OfficialPage,
    }
}

fn validate_catalog(templates: &[GuestTemplate]) -> Result<(), GuestCatalogRepositoryError> {
    let mut ids = HashSet::new();
    for template in templates {
        if !ids.insert(template.id.as_str()) {
            return Err(GuestCatalogRepositoryError::Storage(format!(
                "Duplicate guest template id: {}",
                template.id
            )));
        }
    }
    Ok(())
}

fn storage(error: impl std::fmt::Display) -> GuestCatalogRepositoryError {
    GuestCatalogRepositoryError::Storage(error.to_string())
}
