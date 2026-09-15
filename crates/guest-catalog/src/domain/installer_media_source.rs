// # 📄 Dosya Yolu: /turkuazvm/crates/guest-catalog/src/domain/installer_media_source.rs
// # 📌 Amac: Linux installer medya source resolver icin typed policy, request ve resolved source modellerini tanimlar
// # 📌 Modul - Rust
// # Version: 0.40.0
// # Aciklama: Sabit ISO URL bagimliligini resmi index discovery, mirror adaylari ve last-known-good cache modeliyle degistirir
// # Bagimli Oldugu Katman: Service | Repo | Tool

use std::collections::BTreeMap;

use crate::domain::guest_template::{InstallerMediaKind, InstallerMediaSource};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerMediaChecksumStrategy {
    FixedName,
    FileSuffix,
    Discover,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerMediaDiscoveryMode {
    DirectoryIndex,
    OfficialPageMirrors,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallerMediaProviderRule {
    pub index_path_templates: Vec<String>,
    pub filename_tokens: Vec<String>,
    pub filename_excludes: Vec<String>,
    pub checksum_strategy: InstallerMediaChecksumStrategy,
    pub checksum_value: String,
    pub checksum_tokens: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallerMediaProviderPolicy {
    pub discovery_mode: InstallerMediaDiscoveryMode,
    pub base_urls: Vec<String>,
    pub rules: BTreeMap<String, InstallerMediaProviderRule>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallerMediaResolverPolicy {
    pub use_catalog_fallback: bool,
    pub providers: BTreeMap<String, InstallerMediaProviderPolicy>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallerMediaSourceRequest {
    pub guest_template_id: String,
    pub media_id: String,
    pub provider: String,
    pub release_id: String,
    pub architecture: String,
    pub media_kind: InstallerMediaKind,
    pub catalog_source: InstallerMediaSource,
}

impl InstallerMediaSourceRequest {
    pub fn cache_key(&self) -> String {
        format!(
            "{}::{}::{}::{}::{}",
            self.provider,
            self.release_id,
            self.architecture,
            self.media_kind.code(),
            self.media_id
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerMediaSourceOrigin {
    OnlineDiscovery,
    LastKnownGoodCache,
    CatalogFallback,
}

impl InstallerMediaSourceOrigin {
    pub const fn code(self) -> &'static str {
        match self {
            Self::OnlineDiscovery => "online_discovery",
            Self::LastKnownGoodCache => "last_known_good_cache",
            Self::CatalogFallback => "catalog_fallback",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedInstallerMediaSource {
    pub cache_key: String,
    pub provider: String,
    pub media_kind: InstallerMediaKind,
    pub architecture: String,
    pub filename: String,
    pub download_urls: Vec<String>,
    pub checksum_urls: Vec<String>,
    pub size_bytes: Option<u64>,
    pub origin: InstallerMediaSourceOrigin,
    pub resolved_at_unix: u64,
}

impl ResolvedInstallerMediaSource {
    pub fn matches_request(&self, request: &InstallerMediaSourceRequest) -> bool {
        self.cache_key == request.cache_key()
            && self.provider == request.provider
            && self.media_kind == request.media_kind
            && self.architecture == request.architecture
            && !self.filename.trim().is_empty()
            && !self.download_urls.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallerMediaSourceError {
    Policy(String),
    Provider(String),
    Cache(String),
    Unavailable(String),
}
