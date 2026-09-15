// # 📄 Dosya Yolu: /turkuazvm/crates/guest/src/tools/android_sdk_distribution_tool.rs
// # 📌 Amac: Windows icin resmi Android SDK System Image ve Android Emulator paketlerini Google repository katalogundan indirip kurar
// # 📌 Modul - Rust
// # Version: 0.40.13
// # Aciklama: Android 10-17 surumlerini sabit CI target tahmini yapmadan resmi repository XML kataloglarindan cozer ve SDK Emulator runtime bundle'i uretir
// # Bagimli Oldugu Katman: Tool | Repo

use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use turkuazvm_android_image::domain::artifact::{AndroidImageArtifact, AndroidImageArtifactRole};
use turkuazvm_android_image::domain::build_profile::AndroidImageArchitecture;
use turkuazvm_android_image::domain::image::{AndroidImage, AndroidImageCapabilities, AndroidImageRuntimeKind};
use turkuazvm_android_image::ports::android_image_distribution_port::{
    AndroidImageDistributionError, AndroidImageDistributionPort, AndroidImageDistributionProgress,
    AndroidImageDistributionRegistration, AndroidImageDistributionStage,
};

use crate::tools::http_download_tool::{HttpDownloadSettings, HttpDownloadTool};

const INSTALL_STATUS_SCHEMA_VERSION: u16 = 2;
const LEGACY_INSTALL_STATUS_SCHEMA_VERSION: u16 = 1;
const INSTALL_STATUS_FILE: &str = "install-status.yml";
const INSTALL_LOG_FILE: &str = "install.log";
const CANCEL_REQUEST_FILE: &str = "cancel-requested.flag";
const CANCELLED_ERROR_CODE: &str = "ANDROID_IMAGE_INSTALL_CANCELLED";
const BYTES_PER_GIB: u64 = 1024 * 1024 * 1024;
const DOWNLOAD_POLL_INTERVAL_MS: u64 = 500;
const SYSTEM_IMAGE_DIR: &str = "system-image";
const EMULATOR_DIR: &str = "emulator";
const EMULATOR_EXE: &str = "emulator.exe";

#[derive(Debug, Clone)]
pub struct AndroidSdkDistributionSettings {
    pub output_root: PathBuf,
    pub tool_root: PathBuf,
    pub curl_binary: PathBuf,
    pub minimum_free_disk_gib: u64,
    pub download_retry_count: u32,
    pub download_retry_delay_seconds: u64,
    pub download_connect_timeout_seconds: u64,
    pub repository_base_url: String,
    pub package_index_url: String,
    pub emulator_package_path: String,
    pub architecture: String,
    pub variant_priority: Vec<String>,
    pub system_image_indexes: std::collections::BTreeMap<String, String>,
    pub api_levels: std::collections::BTreeMap<String, u32>,
}

#[derive(Debug, Clone)]
pub struct AndroidSdkDistributionTool {
    settings: AndroidSdkDistributionSettings,
    http: HttpDownloadTool,
}

impl AndroidSdkDistributionTool {
    pub fn new(settings: AndroidSdkDistributionSettings) -> Self {
        let http = HttpDownloadTool::new(HttpDownloadSettings {
            curl_binary: settings.curl_binary.clone(),
            connect_timeout_seconds: settings.download_connect_timeout_seconds,
            retry_count: settings.download_retry_count,
            retry_delay_seconds: settings.download_retry_delay_seconds,
        });
        Self { settings, http }
    }

    fn install_inner(&self, image: &AndroidImage) -> Result<AndroidImageDistributionRegistration, String> {
        if !cfg!(windows) {
            return Err(String::from("Android SDK Emulator automatic distribution is enabled only on Windows hosts"));
        }
        if image.architecture != AndroidImageArchitecture::X86_64 || self.settings.architecture != "x86_64" {
            return Err(String::from("Android SDK distribution requires x86_64 architecture"));
        }
        let release = image.requested_release.as_deref().ok_or_else(|| String::from("Android SDK distribution requires requested_release"))?;
        let api_level = self.settings.api_levels.get(release).copied().ok_or_else(|| format!("Android {release} SDK API mapping is not configured"))?;

        fs::create_dir_all(&self.settings.output_root).map_err(|error| error.to_string())?;
        fs::create_dir_all(&self.settings.tool_root).map_err(|error| error.to_string())?;
        let stage_root = self.stage_root(image.id.as_str());
        let final_root = self.final_root(image.id.as_str());
        let backup_root = self.settings.output_root.join(format!(".{}.backup", image.id.as_str()));
        remove_if_exists(&stage_root)?;
        remove_if_exists(&backup_root)?;
        fs::create_dir_all(&stage_root).map_err(|error| error.to_string())?;
        self.initialize_log(image.id.as_str())?;

        self.record_phase(image.id.as_str(), AndroidImageDistributionStage::CheckingDisk, 0, None, "Android SDK System Image kurulumu icin bos disk kontrol ediliyor.")?;
        self.validate_free_disk()?;
        self.ensure_not_cancelled(image.id.as_str())?;

        self.record_phase(image.id.as_str(), AndroidImageDistributionStage::Discovering, 0, None, "Resmi Android SDK repository katalogu okunuyor.")?;
        let selection = self.resolve_system_image(release, api_level)?;
        let emulator_archive = self.resolve_archive(&self.settings.package_index_url, &self.settings.emulator_package_path, true)?;
        self.append_log(image.id.as_str(), &format!("SDK resolver: release={release} api={api_level} variant={} package={} system_url={} emulator_url={}", selection.variant, selection.package_path, selection.archive.url, emulator_archive.url))?;
        self.ensure_not_cancelled(image.id.as_str())?;

        let downloads = stage_root.join("downloads");
        fs::create_dir_all(&downloads).map_err(|error| error.to_string())?;
        let system_archive_path = downloads.join("system-image.zip");
        self.download_archive(
            image.id.as_str(),
            AndroidImageDistributionStage::DownloadingDevice,
            "Android SDK System Image indiriliyor.",
            &selection.archive,
            &system_archive_path,
        )?;

        let emulator_binary = self.settings.tool_root.join(EMULATOR_DIR).join(EMULATOR_EXE);
        if !emulator_binary.is_file() {
            let emulator_archive_path = downloads.join("emulator.zip");
            self.download_archive(
                image.id.as_str(),
                AndroidImageDistributionStage::DownloadingHost,
                "Android Emulator Windows paketi indiriliyor.",
                &emulator_archive,
                &emulator_archive_path,
            )?;
            self.record_phase(image.id.as_str(), AndroidImageDistributionStage::Extracting, 0, None, "Android Emulator paketi aciliyor.")?;
            self.install_emulator_package(&emulator_archive_path, &stage_root.join("emulator-extract"))?;
        } else {
            self.append_log(image.id.as_str(), &format!("Android Emulator zaten kurulu: {}", emulator_binary.display()))?;
        }
        self.ensure_not_cancelled(image.id.as_str())?;

        self.record_phase(image.id.as_str(), AndroidImageDistributionStage::Validating, file_size(&system_archive_path), selection.archive.size, "Indirilen Android SDK arsivi boyut ve path guvenligi acisindan dogrulaniyor.")?;
        validate_expected_size(&system_archive_path, selection.archive.size)?;
        validate_zip_listing(&system_archive_path)?;

        let system_extract = stage_root.join("system-extract");
        remove_if_exists(&system_extract)?;
        fs::create_dir_all(&system_extract).map_err(|error| error.to_string())?;
        self.record_phase(image.id.as_str(), AndroidImageDistributionStage::Extracting, 0, None, "Android SDK System Image paketi aciliyor.")?;
        extract_zip(&system_archive_path, &system_extract)?;
        self.ensure_not_cancelled(image.id.as_str())?;

        let source_system_dir = find_system_image_root(&system_extract)?;
        let bundle_root = stage_root.join("bundle");
        remove_if_exists(&bundle_root)?;
        fs::create_dir_all(&bundle_root).map_err(|error| error.to_string())?;
        self.record_phase(image.id.as_str(), AndroidImageDistributionStage::Assembling, 0, None, "Android Emulator runtime bundle hazirlaniyor.")?;
        copy_dir_recursive(&source_system_dir, &bundle_root.join(SYSTEM_IMAGE_DIR))?;
        write_distribution_metadata(&bundle_root, image.id.as_str(), release, api_level, &selection)?;
        fs::copy(self.stage_log_path(image.id.as_str()), bundle_root.join(INSTALL_LOG_FILE)).map_err(|error| error.to_string())?;
        let artifacts = scan_sdk_artifacts(&bundle_root)?;
        turkuazvm_android_image::domain::artifact::validate_sdk_emulator_candidate(&artifacts).map_err(|error| format!("SDK emulator bundle validation failed: {error:?}"))?;

        self.record_phase(image.id.as_str(), AndroidImageDistributionStage::Finalizing, 0, None, "Android SDK image bundle atomik olarak etkinlestiriliyor.")?;
        replace_bundle_atomically(&bundle_root, &final_root, &backup_root)?;
        remove_if_exists(&stage_root)?;
        remove_if_exists(&backup_root)?;
        let final_log = final_root.join(INSTALL_LOG_FILE).to_string_lossy().into_owned();
        write_status_file(&final_root, image.id.as_str(), AndroidImageDistributionStage::Completed, 0, None, "Android SDK System Image hazir.", &final_log)?;

        Ok(AndroidImageDistributionRegistration {
            source_revision: Some(format!("android-sdk:{}:{}", api_level, selection.variant)),
            android_release: Some(release.to_owned()),
            sdk_level: Some(api_level),
            runtime_kind: AndroidImageRuntimeKind::SdkEmulator,
            artifacts,
            capabilities: AndroidImageCapabilities::stock_sdk_emulator_x86_64(),
        })
    }

    fn resolve_system_image(&self, release: &str, api_level: u32) -> Result<SystemImageSelection, String> {
        for variant in &self.settings.variant_priority {
            let Some(index_url) = self.settings.system_image_indexes.get(variant) else { continue; };
            let package_path = format!("system-images;android-{api_level};{variant};{}", self.settings.architecture);
            match self.resolve_archive(index_url, &package_path, false) {
                Ok(archive) => return Ok(SystemImageSelection { variant: variant.clone(), package_path, archive }),
                Err(error) => {
                    let _ = release;
                    if !error.contains("package not found") { return Err(error); }
                }
            }
        }
        Err(format!("Android {release} API {api_level} icin x86_64 SDK System Image resmi katalogda bulunamadi"))
    }

    fn resolve_archive(&self, index_url: &str, package_path: &str, windows_host_required: bool) -> Result<RemoteArchive, String> {
        let xml = self.http.fetch_text(index_url)?;
        let package = remote_package_block(&xml, package_path).ok_or_else(|| format!("package not found: {package_path}"))?;
        let archives = archive_blocks(package);
        let archive = archives.into_iter().find_map(|block| parse_archive(block, index_url, &self.settings.repository_base_url, windows_host_required));
        archive.ok_or_else(|| format!("compatible archive not found: {package_path}"))
    }

    fn download_archive(&self, image_id: &str, stage: AndroidImageDistributionStage, detail: &str, archive: &RemoteArchive, destination: &Path) -> Result<(), String> {
        if let Some(parent) = destination.parent() { fs::create_dir_all(parent).map_err(|error| error.to_string())?; }
        let mut child = self.http.spawn_file_download(&archive.url, destination)?;
        loop {
            if self.cancel_request_path(image_id).is_file() {
                let _ = child.kill();
                let _ = child.wait();
                return Err(String::from(CANCELLED_ERROR_CODE));
            }
            let downloaded = file_size(destination);
            self.write_progress(image_id, stage, downloaded, archive.size, detail)?;
            match child.try_wait().map_err(|error| error.to_string())? {
                Some(status) if status.success() => break,
                Some(status) => return Err(format!("Android SDK download failed: url={} status={status}", archive.url)),
                None => thread::sleep(Duration::from_millis(DOWNLOAD_POLL_INTERVAL_MS)),
            }
        }
        validate_expected_size(destination, archive.size)
    }

    fn install_emulator_package(&self, archive: &Path, extract_root: &Path) -> Result<(), String> {
        validate_zip_listing(archive)?;
        remove_if_exists(extract_root)?;
        fs::create_dir_all(extract_root).map_err(|error| error.to_string())?;
        extract_zip(archive, extract_root)?;
        let source = find_dir_containing(extract_root, EMULATOR_EXE).ok_or_else(|| String::from("Android Emulator package emulator.exe icermiyor"))?;
        let destination = self.settings.tool_root.join(EMULATOR_DIR);
        let backup = self.settings.tool_root.join(".emulator.backup");
        remove_if_exists(&backup)?;
        if destination.exists() { fs::rename(&destination, &backup).map_err(|error| error.to_string())?; }
        if let Err(error) = copy_dir_recursive(&source, &destination) {
            let _ = remove_if_exists(&destination);
            if backup.exists() { let _ = fs::rename(&backup, &destination); }
            return Err(error);
        }
        remove_if_exists(&backup)?;
        if !destination.join(EMULATOR_EXE).is_file() {
            return Err(String::from("Android Emulator install validation failed: emulator.exe missing"));
        }
        Ok(())
    }

    fn validate_free_disk(&self) -> Result<(), String> {
        let free = fs2::available_space(&self.settings.output_root).map_err(|error| format!("free disk probe failed: {error}"))?;
        let required = self.settings.minimum_free_disk_gib.saturating_mul(BYTES_PER_GIB);
        if free < required { return Err(format!("Android SDK image icin yetersiz disk: required_gib={} free_gib={}", self.settings.minimum_free_disk_gib, free / BYTES_PER_GIB)); }
        Ok(())
    }

    fn stage_root(&self, image_id: &str) -> PathBuf { self.settings.output_root.join(format!(".{image_id}.installing")) }
    fn final_root(&self, image_id: &str) -> PathBuf { self.settings.output_root.join(image_id) }
    fn stage_log_path(&self, image_id: &str) -> PathBuf { self.stage_root(image_id).join(INSTALL_LOG_FILE) }
    fn cancel_request_path(&self, image_id: &str) -> PathBuf { self.stage_root(image_id).join(CANCEL_REQUEST_FILE) }

    fn ensure_not_cancelled(&self, image_id: &str) -> Result<(), String> {
        if self.cancel_request_path(image_id).is_file() { return Err(String::from(CANCELLED_ERROR_CODE)); }
        Ok(())
    }

    fn initialize_log(&self, image_id: &str) -> Result<(), String> {
        let path = self.stage_log_path(image_id);
        if path.is_file() { return Ok(()); }
        if let Some(parent) = path.parent() { fs::create_dir_all(parent).map_err(|error| error.to_string())?; }
        let version = env!("CARGO_PKG_VERSION");
        fs::write(path, format!("# 📄 Dosya Yolu: /turkuazvm/data/android-image-builds/.{image_id}.installing/{INSTALL_LOG_FILE}\n# 📌 Amac: Android SDK System Image otomatik kurulum kaydini saklar\n# 📌 Modul - Log\n# Version: {version}\n# Aciklama: Resmi repository discovery, download, extract ve bundle asamalarini kaydeder\n# Bagimli Oldugu Katman: Tool | View\n\n")).map_err(|error| error.to_string())
    }

    fn append_log(&self, image_id: &str, message: &str) -> Result<(), String> {
        self.initialize_log(image_id)?;
        let mut file = OpenOptions::new().append(true).open(self.stage_log_path(image_id)).map_err(|error| error.to_string())?;
        writeln!(file, "[{}] {message}", epoch_seconds()).map_err(|error| error.to_string())
    }

    fn record_phase(&self, image_id: &str, stage: AndroidImageDistributionStage, downloaded_bytes: u64, total_bytes: Option<u64>, detail: &str) -> Result<(), String> {
        self.append_log(image_id, detail)?;
        self.write_progress(image_id, stage, downloaded_bytes, total_bytes, detail)
    }

    fn write_progress(&self, image_id: &str, stage: AndroidImageDistributionStage, downloaded_bytes: u64, total_bytes: Option<u64>, detail: &str) -> Result<(), String> {
        let root = self.stage_root(image_id);
        let log_path = self.stage_log_path(image_id).to_string_lossy().into_owned();
        write_status_file(&root, image_id, stage, downloaded_bytes, total_bytes, detail, &log_path)
    }

    fn read_progress(&self, image: &AndroidImage) -> Result<Option<AndroidImageDistributionProgress>, String> {
        let stage_status = self.stage_root(image.id.as_str()).join(INSTALL_STATUS_FILE);
        let final_status = self.final_root(image.id.as_str()).join(INSTALL_STATUS_FILE);
        let path = if stage_status.is_file() { stage_status } else if final_status.is_file() { final_status } else { return Ok(None); };
        let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
        let dto: InstallStatusDto = serde_yaml_ng::from_str(&content).map_err(|error| error.to_string())?;
        if !matches!(dto.schema_version, LEGACY_INSTALL_STATUS_SCHEMA_VERSION | INSTALL_STATUS_SCHEMA_VERSION) { return Err(String::from("unsupported Android install status schema")); }
        let updated = dto.updated_at_epoch_seconds.unwrap_or_else(epoch_seconds);
        let started = dto.stage_started_at_epoch_seconds.unwrap_or(updated);
        let elapsed = updated.saturating_sub(started);
        let start_bytes = dto.stage_start_downloaded_bytes.unwrap_or(0);
        let transferred = dto.downloaded_bytes.saturating_sub(start_bytes);
        let rate = (elapsed > 0 && transferred > 0).then(|| transferred / elapsed).filter(|value| *value > 0);
        let eta = match (dto.total_bytes, rate) { (Some(total), Some(rate)) if total > dto.downloaded_bytes => Some((total - dto.downloaded_bytes) / rate), _ => None };
        Ok(Some(AndroidImageDistributionProgress { stage: parse_stage(&dto.stage)?, downloaded_bytes: dto.downloaded_bytes, total_bytes: dto.total_bytes, detail: dto.detail, log_path: dto.log_path, elapsed_seconds: elapsed, bytes_per_second: rate, eta_seconds: eta }))
    }
}

impl AndroidImageDistributionPort for AndroidSdkDistributionTool {
    fn prepare_install(&self, image: &AndroidImage) -> Result<(), AndroidImageDistributionError> {
        remove_if_exists(&self.cancel_request_path(image.id.as_str())).map_err(AndroidImageDistributionError::Install)
    }

    fn install(&self, image: &AndroidImage) -> Result<AndroidImageDistributionRegistration, AndroidImageDistributionError> {
        match self.install_inner(image) {
            Ok(value) => Ok(value),
            Err(error) if error == CANCELLED_ERROR_CODE => Err(AndroidImageDistributionError::Cancelled),
            Err(error) => {
                let _ = self.record_phase(image.id.as_str(), AndroidImageDistributionStage::Failed, 0, None, &error);
                Err(AndroidImageDistributionError::Install(error))
            }
        }
    }

    fn progress(&self, image: &AndroidImage) -> Result<Option<AndroidImageDistributionProgress>, AndroidImageDistributionError> {
        self.read_progress(image).map_err(AndroidImageDistributionError::Install)
    }

    fn cancel(&self, image: &AndroidImage) -> Result<(), AndroidImageDistributionError> {
        let stage_root = self.stage_root(image.id.as_str());
        fs::create_dir_all(&stage_root).map_err(|error| AndroidImageDistributionError::Install(error.to_string()))?;
        fs::write(self.cancel_request_path(image.id.as_str()), "cancel\n").map_err(|error| AndroidImageDistributionError::Install(error.to_string()))?;
        self.record_phase(image.id.as_str(), AndroidImageDistributionStage::Cancelling, 0, None, "Android SDK image kurulumu iptal ediliyor.").map_err(AndroidImageDistributionError::Install)
    }

    fn cleanup(&self, image: &AndroidImage) -> Result<(), AndroidImageDistributionError> {
        remove_if_exists(&self.stage_root(image.id.as_str())).map_err(AndroidImageDistributionError::Install)?;
        remove_if_exists(&self.settings.output_root.join(format!(".{}.backup", image.id.as_str()))).map_err(AndroidImageDistributionError::Install)
    }
}

#[derive(Debug, Clone)]
struct RemoteArchive { url: String, size: Option<u64> }

#[derive(Debug, Clone)]
struct SystemImageSelection { variant: String, package_path: String, archive: RemoteArchive }

#[derive(Debug, Serialize, Deserialize)]
struct InstallStatusDto {
    schema_version: u16,
    stage: String,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
    detail: String,
    log_path: String,
    #[serde(default)] install_started_at_epoch_seconds: Option<u64>,
    #[serde(default)] stage_started_at_epoch_seconds: Option<u64>,
    #[serde(default)] stage_start_downloaded_bytes: Option<u64>,
    #[serde(default)] updated_at_epoch_seconds: Option<u64>,
}

fn remote_package_block<'a>(xml: &'a str, package_path: &str) -> Option<&'a str> {
    let marker = format!("<remotePackage path=\"{package_path}\"");
    let start = xml.find(&marker)?;
    let end_rel = xml[start..].find("</remotePackage>")?;
    Some(&xml[start..start + end_rel + "</remotePackage>".len()])
}

fn archive_blocks(package: &str) -> Vec<&str> {
    let mut blocks = Vec::new();
    let mut offset = 0_usize;
    while let Some(start_rel) = package[offset..].find("<archive>") {
        let start = offset + start_rel;
        let Some(end_rel) = package[start..].find("</archive>") else { break; };
        let end = start + end_rel + "</archive>".len();
        blocks.push(&package[start..end]);
        offset = end;
    }
    blocks
}

fn parse_archive(block: &str, index_url: &str, repository_base_url: &str, windows_host_required: bool) -> Option<RemoteArchive> {
    if windows_host_required {
        let host_os = xml_text(block, "host-os")?;
        if host_os.trim() != "windows" { return None; }
    } else if let Some(host_os) = xml_text(block, "host-os") {
        if host_os.trim() != "windows" { return None; }
    }
    let url = xml_text(block, "url")?;
    let size = xml_text(block, "size").and_then(|value| value.parse::<u64>().ok());
    let resolved = if url.starts_with("http://") || url.starts_with("https://") {
        url
    } else if index_url.ends_with('/') {
        format!("{index_url}{url}")
    } else if let Some((base, _)) = index_url.rsplit_once('/') {
        format!("{base}/{url}")
    } else {
        format!("{}/{}", repository_base_url.trim_end_matches('/'), url)
    };
    Some(RemoteArchive { url: resolved, size })
}

fn xml_text(text: &str, tag: &str) -> Option<String> {
    let start_marker = format!("<{tag}");
    let start = text.find(&start_marker)?;
    let content_start = text[start..].find('>')? + start + 1;
    let end_marker = format!("</{tag}>");
    let end = text[content_start..].find(&end_marker)? + content_start;
    Some(text[content_start..end].trim().replace("&amp;", "&"))
}

fn validate_expected_size(path: &Path, expected: Option<u64>) -> Result<(), String> {
    let actual = file_size(path);
    if actual == 0 { return Err(format!("downloaded archive is empty: {}", path.display())); }
    if let Some(expected) = expected {
        if actual != expected { return Err(format!("download size mismatch: expected={expected} actual={actual} path={}", path.display())); }
    }
    Ok(())
}

fn validate_zip_listing(path: &Path) -> Result<(), String> {
    let output = Command::new("tar").args(["-tf"]).arg(path).output().map_err(|error| format!("tar archive listing unavailable: {error}"))?;
    if !output.status.success() { return Err(format!("zip archive listing failed: {}", String::from_utf8_lossy(&output.stderr).trim())); }
    let mut count = 0_usize;
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let normalized = line.trim().replace('\\', "/");
        if normalized.is_empty() { continue; }
        count += 1;
        let candidate = Path::new(&normalized);
        if normalized.starts_with('/') || normalized.get(1..2) == Some(":") || candidate.components().any(|component| matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_))) {
            return Err(format!("unsafe archive path: {normalized}"));
        }
    }
    if count == 0 { return Err(String::from("archive contains no entries")); }
    Ok(())
}

fn extract_zip(archive: &Path, destination: &Path) -> Result<(), String> {
    let output = Command::new("tar").args(["-xf"]).arg(archive).arg("-C").arg(destination).output().map_err(|error| format!("tar extraction unavailable: {error}"))?;
    if !output.status.success() { return Err(format!("zip extraction failed: {}", String::from_utf8_lossy(&output.stderr).trim())); }
    Ok(())
}

fn find_system_image_root(root: &Path) -> Result<PathBuf, String> {
    let system = find_file_named(root, "system.img").ok_or_else(|| String::from("SDK System Image system.img icermiyor"))?;
    let parent = system.parent().ok_or_else(|| String::from("system.img parent directory unavailable"))?.to_path_buf();
    for required in ["ramdisk.img", "userdata.img"] {
        if !parent.join(required).is_file() { return Err(format!("SDK System Image required artifact missing: {required}")); }
    }
    if !parent.join("kernel-ranchu").is_file() && !parent.join("kernel-qemu").is_file() {
        return Err(String::from("SDK System Image kernel-ranchu/kernel-qemu icermiyor"));
    }
    Ok(parent)
}

fn find_dir_containing(root: &Path, name: &str) -> Option<PathBuf> { find_file_named(root, name).and_then(|path| path.parent().map(Path::to_path_buf)) }

fn find_file_named(root: &Path, name: &str) -> Option<PathBuf> {
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(dir).ok()? {
            let entry = entry.ok()?;
            let file_type = entry.file_type().ok()?;
            if file_type.is_symlink() { continue; }
            let path = entry.path();
            if file_type.is_dir() { stack.push(path); continue; }
            if file_type.is_file() && entry.file_name().to_string_lossy().eq_ignore_ascii_case(name) { return Some(path); }
        }
    }
    None
}

fn copy_dir_recursive(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(|error| error.to_string())?;
    for entry in fs::read_dir(source).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let kind = entry.file_type().map_err(|error| error.to_string())?;
        if kind.is_symlink() { return Err(format!("symlink is not allowed in Android SDK package: {}", entry.path().display())); }
        let target = destination.join(entry.file_name());
        if kind.is_dir() { copy_dir_recursive(&entry.path(), &target)?; }
        else if kind.is_file() { fs::copy(entry.path(), target).map_err(|error| error.to_string())?; }
    }
    Ok(())
}

fn scan_sdk_artifacts(root: &Path) -> Result<Vec<AndroidImageArtifact>, String> {
    let mut artifacts = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let file_type = entry.file_type().map_err(|error| error.to_string())?;
            if file_type.is_symlink() { return Err(String::from("SDK bundle symlink is not allowed")); }
            let path = entry.path();
            if file_type.is_dir() { stack.push(path); continue; }
            if !file_type.is_file() { continue; }
            let name = entry.file_name().to_string_lossy().into_owned();
            let role = match name.as_str() {
                "kernel-ranchu" | "kernel-qemu" => Some(AndroidImageArtifactRole::Kernel),
                "ramdisk.img" => Some(AndroidImageArtifactRole::Ramdisk),
                "system.img" => Some(AndroidImageArtifactRole::System),
                "userdata.img" => Some(AndroidImageArtifactRole::Userdata),
                value if value.ends_with(".img") => Some(AndroidImageArtifactRole::Other),
                _ => None,
            };
            let Some(role) = role else { continue; };
            let relative = path.strip_prefix(root).map_err(|error| error.to_string())?.to_string_lossy().replace('\\', "/");
            let size = fs::metadata(&path).map_err(|error| error.to_string())?.len();
            if size == 0 { continue; }
            artifacts.push(AndroidImageArtifact::create(role, relative, size, sha256_file(&path)?).map_err(|error| format!("{error:?}"))?);
        }
    }
    artifacts.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(artifacts)
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|error| error.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 { break; }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn write_distribution_metadata(root: &Path, image_id: &str, release: &str, api_level: u32, selection: &SystemImageSelection) -> Result<(), String> {
    let version = env!("CARGO_PKG_VERSION");
    let content = format!("# 📄 Dosya Yolu: /turkuazvm/data/android-image-builds/{image_id}/distribution.yml\n# 📌 Amac: Android SDK System Image provenance bilgisini saklar\n# 📌 Modul - YAML\n# Version: {version}\n# Aciklama: Android release, API, SDK variant ve resmi archive URL bilgisini kaydeder\n# Bagimli Oldugu Katman: Repo | Tool\nprovider: android_sdk\nandroid_release: \"{release}\"\napi_level: {api_level}\nvariant: \"{}\"\npackage_path: \"{}\"\narchive_url: \"{}\"\n", selection.variant, selection.package_path, selection.archive.url);
    fs::write(root.join("distribution.yml"), content).map_err(|error| error.to_string())
}

fn replace_bundle_atomically(bundle: &Path, final_root: &Path, backup_root: &Path) -> Result<(), String> {
    if final_root.exists() { fs::rename(final_root, backup_root).map_err(|error| error.to_string())?; }
    match fs::rename(bundle, final_root) {
        Ok(()) => Ok(()),
        Err(error) => {
            if backup_root.exists() && !final_root.exists() { let _ = fs::rename(backup_root, final_root); }
            Err(format!("bundle activation failed: {error}"))
        }
    }
}

fn remove_if_exists(path: &Path) -> Result<(), String> {
    if path.is_dir() { fs::remove_dir_all(path).map_err(|error| error.to_string())?; }
    else if path.is_file() { fs::remove_file(path).map_err(|error| error.to_string())?; }
    Ok(())
}

fn file_size(path: &Path) -> u64 { fs::metadata(path).map(|metadata| metadata.len()).unwrap_or_default() }
fn epoch_seconds() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_secs()).unwrap_or_default() }

fn write_status_file(root: &Path, image_id: &str, stage: AndroidImageDistributionStage, downloaded_bytes: u64, total_bytes: Option<u64>, detail: &str, log_path: &str) -> Result<(), String> {
    fs::create_dir_all(root).map_err(|error| error.to_string())?;
    let path = root.join(INSTALL_STATUS_FILE);
    let now = epoch_seconds();
    let stage_value = stage_code(stage).to_owned();
    let previous = fs::read_to_string(&path).ok().and_then(|content| serde_yaml_ng::from_str::<InstallStatusDto>(&content).ok());
    let install_started = previous.as_ref().and_then(|value| value.install_started_at_epoch_seconds).unwrap_or(now);
    let same_stage = previous.as_ref().is_some_and(|value| value.stage == stage_value);
    let stage_started = if same_stage { previous.as_ref().and_then(|value| value.stage_started_at_epoch_seconds).unwrap_or(now) } else { now };
    let stage_start_bytes = if same_stage { previous.as_ref().and_then(|value| value.stage_start_downloaded_bytes).unwrap_or(downloaded_bytes) } else { downloaded_bytes };
    let dto = InstallStatusDto { schema_version: INSTALL_STATUS_SCHEMA_VERSION, stage: stage_value, downloaded_bytes, total_bytes, detail: detail.to_owned(), log_path: log_path.to_owned(), install_started_at_epoch_seconds: Some(install_started), stage_started_at_epoch_seconds: Some(stage_started), stage_start_downloaded_bytes: Some(stage_start_bytes), updated_at_epoch_seconds: Some(now) };
    let body = serde_yaml_ng::to_string(&dto).map_err(|error| error.to_string())?;
    let version = env!("CARGO_PKG_VERSION");
    let header = format!("# 📄 Dosya Yolu: /turkuazvm/data/android-image-builds/{image_id}/{INSTALL_STATUS_FILE}\n# 📌 Amac: Android SDK image kurulum ilerlemesini saklar\n# 📌 Modul - YAML\n# Version: {version}\n# Aciklama: Desktop polling icin stage, byte, sure ve log yolunu kaydeder\n# Bagimli Oldugu Katman: Tool | View\n");
    let temp = root.join(format!(".{INSTALL_STATUS_FILE}.tmp"));
    fs::write(&temp, format!("{header}{body}")).map_err(|error| error.to_string())?;
    if path.is_file() { fs::remove_file(&path).map_err(|error| error.to_string())?; }
    fs::rename(temp, path).map_err(|error| error.to_string())
}

const fn stage_code(stage: AndroidImageDistributionStage) -> &'static str {
    match stage {
        AndroidImageDistributionStage::Discovering => "discovering",
        AndroidImageDistributionStage::CheckingDisk => "checking_disk",
        AndroidImageDistributionStage::DownloadingDevice => "downloading_device",
        AndroidImageDistributionStage::DownloadingHost => "downloading_host",
        AndroidImageDistributionStage::Validating => "validating",
        AndroidImageDistributionStage::Extracting => "extracting",
        AndroidImageDistributionStage::Assembling => "assembling",
        AndroidImageDistributionStage::Finalizing => "finalizing",
        AndroidImageDistributionStage::Cancelling => "cancelling",
        AndroidImageDistributionStage::Completed => "completed",
        AndroidImageDistributionStage::Failed => "failed",
    }
}

fn parse_stage(value: &str) -> Result<AndroidImageDistributionStage, String> {
    match value {
        "discovering" => Ok(AndroidImageDistributionStage::Discovering),
        "checking_disk" => Ok(AndroidImageDistributionStage::CheckingDisk),
        "downloading_device" => Ok(AndroidImageDistributionStage::DownloadingDevice),
        "downloading_host" => Ok(AndroidImageDistributionStage::DownloadingHost),
        "validating" => Ok(AndroidImageDistributionStage::Validating),
        "extracting" => Ok(AndroidImageDistributionStage::Extracting),
        "assembling" => Ok(AndroidImageDistributionStage::Assembling),
        "finalizing" => Ok(AndroidImageDistributionStage::Finalizing),
        "cancelling" => Ok(AndroidImageDistributionStage::Cancelling),
        "completed" => Ok(AndroidImageDistributionStage::Completed),
        "failed" => Ok(AndroidImageDistributionStage::Failed),
        _ => Err(String::from("unsupported Android install stage")),
    }
}

#[cfg(test)]
mod tests {
    use super::{archive_blocks, parse_archive, remote_package_block, xml_text};

    #[test]
    fn parses_repository_archive_without_hardcoded_filename() {
        let xml = r#"<sdk-repository><remotePackage path="system-images;android-30;default;x86_64"><archives><archive><complete><size>123</size><url>x86.zip</url></complete></archive></archives></remotePackage></sdk-repository>"#;
        let package = remote_package_block(xml, "system-images;android-30;default;x86_64").expect("package");
        let block = archive_blocks(package).into_iter().next().expect("archive");
        let archive = parse_archive(block, "https://dl.google.com/android/repository/sys-img/android/sys-img2-1.xml", "https://dl.google.com/android/repository", false).expect("parsed");
        assert_eq!(archive.url, "https://dl.google.com/android/repository/sys-img/android/x86.zip");
        assert_eq!(archive.size, Some(123));
    }

    #[test]
    fn xml_text_handles_attributes_and_entities() {
        assert_eq!(xml_text("<url type=\"x\">a&amp;b</url>", "url").as_deref(), Some("a&b"));
    }
}
