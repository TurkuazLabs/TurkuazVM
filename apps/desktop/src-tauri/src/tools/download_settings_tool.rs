// # 📄 Dosya Yolu: /turkuazvm/apps/desktop/src-tauri/src/tools/download_settings_tool.rs
// # 📌 Amac: Merkezi download-sources.yml dosyasini guvenli ve atomik olarak okur/gunceller
// # 📌 Modul - Rust
// # Version: 0.40.13
// # Aciklama: Schema 5 Linux ve Android Source Resolver policy alanlarini kayipsiz koruyarak Desktop Service icin persistence adapteri olarak yonetir
// # Bagimli Oldugu Katman: Service | Tool

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

const SETTINGS_RELATIVE_PATH: &str = "config/download-sources.yml";
const SETTINGS_SCHEMA_VERSION: u16 = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadSettings {
    pub installer_media_path: String,
    pub android_images_path: String,
    pub artifact_cache_path: String,
    pub android_ci_base_url: String,
    pub official_fallback: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadSettingsUpdate {
    pub installer_media_path: String,
    pub android_images_path: String,
    pub artifact_cache_path: String,
    pub android_ci_base_url: String,
    pub official_fallback: bool,
}

pub struct DownloadSettingsTool;

impl DownloadSettingsTool {
    pub fn read(project_root: &Path) -> Result<DownloadSettings, String> {
        let path = project_root.join(SETTINGS_RELATIVE_PATH);
        let file = read_file(&path)?;
        Ok(DownloadSettings {
            installer_media_path: file.paths.installer_media,
            android_images_path: file.paths.android_images,
            artifact_cache_path: file.paths.artifact_cache,
            android_ci_base_url: file.sources.android_ci.base_url,
            official_fallback: file.sources.android_ci.use_official_fallback,
        })
    }

    pub fn update(project_root: &Path, update: DownloadSettingsUpdate) -> Result<DownloadSettings, String> {
        let installer_media = validate_path(&update.installer_media_path, "Kurulum medyasi yolu")?;
        let android_images = validate_path(&update.android_images_path, "Android image yolu")?;
        let artifact_cache = validate_path(&update.artifact_cache_path, "Onbellek yolu")?;
        let android_ci_base_url = validate_base_url(&update.android_ci_base_url)?;

        let path = project_root.join(SETTINGS_RELATIVE_PATH);
        let mut file = read_file(&path)?;
        file.paths.installer_media = installer_media.clone();
        file.paths.android_images = android_images.clone();
        file.paths.artifact_cache = artifact_cache.clone();
        file.sources.android_ci.base_url = android_ci_base_url.clone();
        file.sources.android_ci.use_official_fallback = update.official_fallback;
        write_file(&path, &file)?;

        Ok(DownloadSettings {
            installer_media_path: installer_media,
            android_images_path: android_images,
            artifact_cache_path: artifact_cache,
            android_ci_base_url,
            official_fallback: update.official_fallback,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DownloadSourcesFile {
    schema_version: u16,
    paths: DownloadPaths,
    sources: DownloadProviders,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DownloadPaths {
    installer_media: String,
    installer_media_source_cache: String,
    android_images: String,
    artifact_cache: String,
    android_source_cache: String,
    android_sdk_tools: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DownloadProviders {
    linux_media: LinuxMediaProviderCatalog,
    android_release_policy: BTreeMap<String, String>,
    android_sdk: AndroidSdkProvider,
    android_ci: AndroidCiProvider,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LinuxMediaProviderCatalog {
    use_catalog_fallback: bool,
    providers: BTreeMap<String, LinuxMediaProvider>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LinuxMediaProvider {
    discovery_mode: String,
    base_urls: Vec<String>,
    rules: BTreeMap<String, LinuxMediaRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LinuxMediaRule {
    index_path_templates: Vec<String>,
    filename_tokens: Vec<String>,
    filename_excludes: Vec<String>,
    checksum_strategy: String,
    checksum_value: String,
    checksum_tokens: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AndroidSdkProvider {
    repository_base_url: String,
    package_index_url: String,
    emulator_package_path: String,
    architecture: String,
    emulator_console_port_min: u16,
    emulator_console_port_max: u16,
    variant_priority: Vec<String>,
    system_image_indexes: BTreeMap<String, String>,
    api_levels: BTreeMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AndroidCiProvider {
    base_url: String,
    official_base_url: String,
    use_official_fallback: bool,
    default_branch: String,
    default_target: String,
    branch_templates: Vec<String>,
    target_candidates: Vec<String>,
    channels: BTreeMap<String, AndroidCiChannel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AndroidCiChannel {
    expected_sdk: u32,
    #[serde(default)]
    allow_device_bootloader_fallback: bool,
    #[serde(default)]
    branch_hints: Vec<String>,
}

fn read_file(path: &Path) -> Result<DownloadSourcesFile, String> {
    let content = fs::read_to_string(path).map_err(|error| format!("Indirme ayarlari okunamadi: {error}"))?;
    let file: DownloadSourcesFile = serde_yaml_ng::from_str(&content)
        .map_err(|error| format!("Indirme ayarlari YAML hatasi: {error}"))?;
    if file.schema_version != SETTINGS_SCHEMA_VERSION {
        return Err(format!("Desteklenmeyen indirme ayarlari schema surumu: {}", file.schema_version));
    }
    Ok(file)
}

fn write_file(path: &Path, file: &DownloadSourcesFile) -> Result<(), String> {
    let parent = path.parent().ok_or_else(|| String::from("Indirme ayarlari klasoru bulunamadi"))?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let body = serde_yaml_ng::to_string(file).map_err(|error| error.to_string())?;
    let version = env!("CARGO_PKG_VERSION");
    let content = format!(
        "# 📄 Dosya Yolu: /turkuazvm/config/download-sources.yml\n# 📌 Amac: TurkuazVM indirme dizinlerini, Android Source Resolver kaynagini ve surum bazli provider policy bilgisini merkezi olarak tanimlar\n# 📌 Modul - YAML\n# Version: {version}\n# Aciklama: GUI Ayarlar ekranindan yonetilebilen indirme yollarini kayipsiz korur; Linux ve Android resolver/provider policy alanlarini koddan ayirir\n# Bagimli Oldugu Katman: Service | Repo | Tool | View\n\n{body}"
    );
    let temp = PathBuf::from(format!("{}.tmp", path.display()));
    let backup = PathBuf::from(format!("{}.backup", path.display()));
    if temp.exists() { fs::remove_file(&temp).map_err(|error| error.to_string())?; }
    if backup.exists() { fs::remove_file(&backup).map_err(|error| error.to_string())?; }
    fs::write(&temp, content).map_err(|error| error.to_string())?;
    if path.exists() { fs::rename(path, &backup).map_err(|error| error.to_string())?; }
    match fs::rename(&temp, path) {
        Ok(()) => {
            if backup.exists() { let _ = fs::remove_file(&backup); }
            Ok(())
        }
        Err(error) => {
            if backup.exists() && !path.exists() { let _ = fs::rename(&backup, path); }
            let _ = fs::remove_file(&temp);
            Err(error.to_string())
        }
    }
}

fn validate_path(value: &str, field: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() { return Err(format!("{field} bos olamaz")); }
    if value.chars().any(|ch| ch == '\0') { return Err(format!("{field} gecersiz karakter iceriyor")); }
    Ok(value.to_owned())
}

fn validate_base_url(value: &str) -> Result<String, String> {
    let value = value.trim().trim_end_matches('/');
    if !(value.starts_with("https://") || value.starts_with("http://")) || value.chars().any(char::is_whitespace) {
        return Err(String::from("Android CI kaynagi HTTP(S) URL olmali"));
    }
    Ok(value.to_owned())
}
