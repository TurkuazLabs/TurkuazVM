// # 📄 Dosya Yolu: /turkuazvm/crates/repositories/src/repositories/yaml_installer_media_source_cache_repository.rs
// # 📌 Amac: Linux installer media resolved source last-known-good kayitlarini YAML dosyasinda saklar
// # 📌 Modul - Rust
// # Version: 0.40.0
// # Aciklama: Provider+release+mimari+media turu cache anahtariyla mirror URL, checksum URL ve filename bilgisini persist eder
// # Bagimli Oldugu Katman: Repo | Service

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use turkuazvm_guest_catalog::domain::guest_template::InstallerMediaKind;
use turkuazvm_guest_catalog::domain::installer_media_source::{
    InstallerMediaSourceError, InstallerMediaSourceOrigin, ResolvedInstallerMediaSource,
};
use turkuazvm_guest_catalog::ports::installer_media_source_cache_port::InstallerMediaSourceCachePort;

const CACHE_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone)]
pub struct YamlInstallerMediaSourceCacheRepository {
    path: PathBuf,
}

impl YamlInstallerMediaSourceCacheRepository {
    pub const fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn read_cache(&self) -> Result<CacheFile, InstallerMediaSourceError> {
        if !self.path.is_file() {
            return Ok(CacheFile { schema_version: CACHE_SCHEMA_VERSION, entries: BTreeMap::new() });
        }
        let content = fs::read_to_string(&self.path)
            .map_err(|error| InstallerMediaSourceError::Cache(error.to_string()))?;
        let file: CacheFile = serde_yaml_ng::from_str(&content)
            .map_err(|error| InstallerMediaSourceError::Cache(error.to_string()))?;
        if file.schema_version != CACHE_SCHEMA_VERSION {
            return Err(InstallerMediaSourceError::Cache(format!(
                "unsupported installer media source cache schema {}",
                file.schema_version
            )));
        }
        Ok(file)
    }

    fn write_cache(&self, file: &CacheFile) -> Result<(), InstallerMediaSourceError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| InstallerMediaSourceError::Cache(error.to_string()))?;
        }
        let body = serde_yaml_ng::to_string(file)
            .map_err(|error| InstallerMediaSourceError::Cache(error.to_string()))?;
        let content = format!(
            "# 📄 Dosya Yolu: /turkuazvm/data/cache/installer-media-sources.yml\n# 📌 Amac: Linux installer media runtime source resolver last-known-good kayitlarini saklar\n# 📌 Modul - YAML\n# Version: {}\n# Aciklama: Resmi repository discovery gecici kullanilamazsa daha once dogrulanmis mirror ve checksum kaydini kullanir\n# Bagimli Oldugu Katman: Repo | Service\n\n{}",
            env!("CARGO_PKG_VERSION"),
            body
        );
        atomic_write(&self.path, content.as_bytes()).map_err(InstallerMediaSourceError::Cache)
    }
}

impl InstallerMediaSourceCachePort for YamlInstallerMediaSourceCacheRepository {
    fn load(&self, cache_key: &str) -> Result<Option<ResolvedInstallerMediaSource>, InstallerMediaSourceError> {
        let file = self.read_cache()?;
        file.entries.get(cache_key).cloned().map(TryInto::try_into).transpose()
    }

    fn save(&self, source: &ResolvedInstallerMediaSource) -> Result<(), InstallerMediaSourceError> {
        let mut file = self.read_cache()?;
        file.entries.insert(source.cache_key.clone(), SourceDto::from(source));
        self.write_cache(&file)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheFile {
    schema_version: u16,
    entries: BTreeMap<String, SourceDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SourceDto {
    cache_key: String,
    provider: String,
    media_kind: String,
    architecture: String,
    filename: String,
    download_urls: Vec<String>,
    checksum_urls: Vec<String>,
    size_bytes: Option<u64>,
    resolved_at_unix: u64,
}

impl From<&ResolvedInstallerMediaSource> for SourceDto {
    fn from(source: &ResolvedInstallerMediaSource) -> Self {
        Self {
            cache_key: source.cache_key.clone(),
            provider: source.provider.clone(),
            media_kind: source.media_kind.code().to_owned(),
            architecture: source.architecture.clone(),
            filename: source.filename.clone(),
            download_urls: source.download_urls.clone(),
            checksum_urls: source.checksum_urls.clone(),
            size_bytes: source.size_bytes,
            resolved_at_unix: source.resolved_at_unix,
        }
    }
}

impl TryFrom<SourceDto> for ResolvedInstallerMediaSource {
    type Error = InstallerMediaSourceError;

    fn try_from(source: SourceDto) -> Result<Self, Self::Error> {
        Ok(Self {
            cache_key: source.cache_key,
            provider: source.provider,
            media_kind: parse_media_kind(&source.media_kind)?,
            architecture: source.architecture,
            filename: source.filename,
            download_urls: source.download_urls,
            checksum_urls: source.checksum_urls,
            size_bytes: source.size_bytes,
            origin: InstallerMediaSourceOrigin::LastKnownGoodCache,
            resolved_at_unix: source.resolved_at_unix,
        })
    }
}

fn parse_media_kind(value: &str) -> Result<InstallerMediaKind, InstallerMediaSourceError> {
    match value {
        "desktop_live" => Ok(InstallerMediaKind::DesktopLive),
        "server_standard" => Ok(InstallerMediaKind::ServerStandard),
        "network_install" => Ok(InstallerMediaKind::NetworkInstall),
        "boot" => Ok(InstallerMediaKind::Boot),
        "minimal" => Ok(InstallerMediaKind::Minimal),
        "dvd" => Ok(InstallerMediaKind::Dvd),
        other => Err(InstallerMediaSourceError::Cache(format!("unsupported cached media kind {other}"))),
    }
}

fn atomic_write(path: &Path, content: &[u8]) -> Result<(), String> {
    let temp = PathBuf::from(format!("{}.tmp", path.display()));
    if temp.exists() {
        fs::remove_file(&temp).map_err(|error| error.to_string())?;
    }
    fs::write(&temp, content).map_err(|error| error.to_string())?;
    if path.exists() {
        fs::remove_file(path).map_err(|error| error.to_string())?;
    }
    fs::rename(&temp, path).map_err(|error| error.to_string())
}
