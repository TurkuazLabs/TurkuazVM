// # 📄 Dosya Yolu: /turkuazvm/crates/guest/src/tools/android_ci_distribution_tool.rs
// # 📌 Amac: Resmi Android CI Cuttlefish x86_64 dagitimini indirir ve TurkuazVM Android bundle'i olarak kurar
// # 📌 Modul - Rust
// # Version: 0.40.13
// # Aciklama: Source Resolver Service tarafindan kesfedilen immutable Android CI buildini ortak HttpDownloadTool ile indirir; artifact cache, resume, validation ve legacy bootloader fallback uygular
// # Bagimli Oldugu Katman: Tool | Repo

use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use turkuazvm_artifact_cache::domain::artifact_cache::{ArtifactCacheRequest, ArtifactSourceValidators};
use turkuazvm_artifact_cache::ports::artifact_cache_client_port::ArtifactCacheClientPort;
use turkuazvm_android_image::domain::artifact::{AndroidImageArtifact, AndroidImageArtifactRole};
use turkuazvm_android_image::domain::build_profile::AndroidImageArchitecture;
use turkuazvm_android_image::domain::image::{AndroidImage, AndroidImageCapabilities, AndroidImageRuntimeKind};
use turkuazvm_android_image::ports::android_distribution_source_resolver_port::AndroidDistributionSourceResolverPort;
use turkuazvm_android_image::ports::android_image_distribution_port::{
    AndroidImageDistributionError, AndroidImageDistributionPort, AndroidImageDistributionProgress,
    AndroidImageDistributionRegistration, AndroidImageDistributionStage,
};

use crate::tools::android_composite_disk_tool::AndroidCompositeDiskTool;
use crate::tools::http_download_tool::{HttpDownloadSettings, HttpDownloadTool, HttpRequestContext};

const INSTALL_STATUS_SCHEMA_VERSION: u16 = 2;
const LEGACY_INSTALL_STATUS_SCHEMA_VERSION: u16 = 1;
const BYTES_PER_GIB: u64 = 1024 * 1024 * 1024;
const DOWNLOAD_POLL_INTERVAL_MS: u64 = 500;
const INSTALL_STATUS_FILE: &str = "install-status.yml";
const INSTALL_LOG_FILE: &str = "install.log";
const BUILD_ID_FILE: &str = "build-id.txt";
const CANCEL_REQUEST_FILE: &str = "cancel-requested.flag";
const CANCELLED_ERROR_CODE: &str = "ANDROID_IMAGE_INSTALL_CANCELLED";
const ANDROID_CI_BROWSER_USER_AGENT: &str = concat!(
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) TurkuazVM/",
    env!("CARGO_PKG_VERSION"),
    " Safari/537.36"
);
const ANDROID_CI_ACCEPT_ARTIFACT: &str = "Accept: application/octet-stream,*/*";

#[derive(Debug, Clone)]
pub struct AndroidCiDistributionSettings {
    pub output_root: PathBuf,
    pub curl_binary: PathBuf,
    pub minimum_free_disk_gib: u64,
    pub download_retry_count: u32,
    pub download_retry_delay_seconds: u64,
    pub download_connect_timeout_seconds: u64,
}

#[derive(Clone)]
pub struct AndroidCiDistributionTool {
    settings: AndroidCiDistributionSettings,
    source_resolver: SharedAndroidDistributionSourceResolver,
    artifact_cache: Option<SharedArtifactCacheClient>,
    http: HttpDownloadTool,
}

pub type SharedArtifactCacheClient = Arc<Mutex<Box<dyn ArtifactCacheClientPort>>>;
pub type SharedAndroidDistributionSourceResolver = Arc<Mutex<Box<dyn AndroidDistributionSourceResolverPort>>>;

impl std::fmt::Debug for AndroidCiDistributionTool {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AndroidCiDistributionTool")
            .field("settings", &self.settings)
            .field("source_resolver", &"configured")
            .field("artifact_cache", &self.artifact_cache.is_some())
            .field("http", &"configured")
            .finish()
    }
}

impl AndroidCiDistributionTool {
    pub fn new(
        settings: AndroidCiDistributionSettings,
        source_resolver: SharedAndroidDistributionSourceResolver,
    ) -> Self {
        let http = android_http_tool(&settings);
        Self { settings, source_resolver, artifact_cache: None, http }
    }

    pub fn with_artifact_cache(
        settings: AndroidCiDistributionSettings,
        source_resolver: SharedAndroidDistributionSourceResolver,
        artifact_cache: SharedArtifactCacheClient,
    ) -> Self {
        let http = android_http_tool(&settings);
        Self { settings, source_resolver, artifact_cache: Some(artifact_cache), http }
    }

    fn install_inner(&self, image: &AndroidImage) -> Result<AndroidImageDistributionRegistration, String> {
        if image.architecture != AndroidImageArchitecture::X86_64 {
            return Err(String::from("only x86_64 Android CI distribution is supported"));
        }

        fs::create_dir_all(&self.settings.output_root).map_err(|error| error.to_string())?;
        let stage_root = self.stage_root(image.id.as_str());
        let final_root = self.final_root(image.id.as_str());
        let backup_root = self.settings.output_root.join(format!(".{}.backup", image.id.as_str()));
        fs::create_dir_all(&stage_root).map_err(|error| error.to_string())?;
        remove_if_exists(&backup_root)?;
        self.initialize_log(image.id.as_str())?;

        self.ensure_not_cancelled(image.id.as_str())?;
        self.record_phase(
            image.id.as_str(),
            AndroidImageDistributionStage::CheckingDisk,
            0,
            None,
            "Android image kurulumu icin bos disk kontrol ediliyor.",
        )?;
        self.validate_free_disk()?;
        self.ensure_not_cancelled(image.id.as_str())?;

        self.record_phase(
            image.id.as_str(),
            AndroidImageDistributionStage::Discovering,
            0,
            None,
            "Android resmi source resolver provider/cache politikasi ile uyumlu build kesfediliyor.",
        )?;
        let discovery = self.resolve_latest_build(image)?;
        self.ensure_not_cancelled(image.id.as_str())?;
        self.prepare_stage_for_build(image.id.as_str(), &discovery.build_id)?;

        let downloads = stage_root.join("downloads");
        let device_root = stage_root.join("device");
        let bundle_root = stage_root.join("bundle");
        fs::create_dir_all(&downloads).map_err(|error| error.to_string())?;
        fs::create_dir_all(&device_root).map_err(|error| error.to_string())?;
        fs::create_dir_all(&bundle_root).map_err(|error| error.to_string())?;

        let image_archive = downloads.join("device-images.zip");
        let host_archive = downloads.join(&discovery.host_artifact_name);
        let (image_url, image_archive_name) = self.download_device_archive(image.id.as_str(), &discovery, &image_archive)?;
        let host_url = format!("{}/raw/{}", discovery.artifact_base_url, discovery.host_artifact_name);
        let host_downloaded = if discovery.host_package_available {
            match self.download_cached(
                image.id.as_str(),
                AndroidImageDistributionStage::DownloadingHost,
                "Cuttlefish host paketi indiriliyor.",
                &host_url,
                &host_archive,
                &discovery.branch,
                &discovery.target,
                &discovery.build_id,
                &discovery.host_artifact_name,
            ) {
                Ok(()) => true,
                Err(error) if discovery.allow_device_bootloader_fallback && error != CANCELLED_ERROR_CODE => {
                    self.append_log(
                        image.id.as_str(),
                        &format!("Same-build host paketi indirilemedi; legacy device bootloader fallback denenecek: {error}"),
                    )?;
                    false
                }
                Err(error) => return Err(error),
            }
        } else {
            self.append_log(
                image.id.as_str(),
                "Same-build host paketi CI artefaktlarinda bulunamadi; legacy device bootloader fallback kullanilacak.",
            )?;
            false
        };

        self.ensure_not_cancelled(image.id.as_str())?;
        self.record_phase(
            image.id.as_str(),
            AndroidImageDistributionStage::Validating,
            0,
            None,
            "Indirilen arsivler guvenlik ve butunluk acisindan dogrulaniyor.",
        )?;
        validate_archive_entries(&image_archive, ArchiveKind::Zip)?;
        if host_downloaded {
            validate_archive_entries(&host_archive, ArchiveKind::TarGz)?;
        }
        self.ensure_not_cancelled(image.id.as_str())?;

        self.record_phase(
            image.id.as_str(),
            AndroidImageDistributionStage::Extracting,
            0,
            None,
            "Android device paketi aciliyor ve host paketinden x86_64 QEMU bootloader seciliyor.",
        )?;
        remove_if_exists(&device_root)?;
        fs::create_dir_all(&device_root).map_err(|error| error.to_string())?;
        extract_zip(&image_archive, &device_root)?;
        self.ensure_not_cancelled(image.id.as_str())?;

        remove_if_exists(&bundle_root)?;
        fs::create_dir_all(&bundle_root).map_err(|error| error.to_string())?;
        copy_named_artifact(&device_root, &bundle_root, "boot.img", true)?;
        copy_named_artifact(&device_root, &bundle_root, "init_boot.img", false)?;
        copy_named_artifact(&device_root, &bundle_root, "vendor_boot.img", false)?;
        copy_named_artifact(&device_root, &bundle_root, "super.img", true)?;
        copy_named_artifact(&device_root, &bundle_root, "userdata.img", true)?;
        copy_named_artifact(&device_root, &bundle_root, "vbmeta.img", false)?;
        copy_named_artifact(&device_root, &bundle_root, "vbmeta_system.img", false)?;
        copy_named_artifact(&device_root, &bundle_root, "metadata.img", false)?;
        copy_named_artifact(&device_root, &bundle_root, "misc.img", false)?;
        copy_named_artifact(&device_root, &bundle_root, "kernel", false)?;

        let bootloader_source = if host_downloaded {
            extract_x86_64_bootloader(&host_archive, &bundle_root.join("bootloader.qemu"))?;
            "same_build_host_package"
        } else {
            copy_x86_64_bootloader_from_device(&device_root, &bundle_root.join("bootloader.qemu"))?;
            "device_archive"
        };
        self.ensure_not_cancelled(image.id.as_str())?;

        self.record_phase(
            image.id.as_str(),
            AndroidImageDistributionStage::Assembling,
            0,
            None,
            "TurkuazVM Android composite disk uretiliyor.",
        )?;
        AndroidCompositeDiskTool::assemble(&bundle_root, &bundle_root.join("composite.img"))?;
        self.ensure_not_cancelled(image.id.as_str())?;

        let image_sha256 = sha256_file(&image_archive)?;
        let host_sha256 = if host_downloaded { Some(sha256_file(&host_archive)?) } else { None };
        write_distribution_metadata(
            &bundle_root,
            image.id.as_str(),
            &discovery.base_url,
            &discovery.branch,
            &discovery.target,
            &discovery.build_id,
            &image_archive_name,
            &image_url,
            &image_sha256,
            host_downloaded.then_some(host_url.as_str()),
            host_sha256.as_deref(),
            bootloader_source,
        )?;

        self.record_phase(
            image.id.as_str(),
            AndroidImageDistributionStage::Finalizing,
            0,
            None,
            "Bundle atomik olarak registry alanina aliniyor.",
        )?;
        let stage_log = self.stage_log_path(image.id.as_str());
        if stage_log.is_file() {
            fs::copy(&stage_log, bundle_root.join(INSTALL_LOG_FILE)).map_err(|error| error.to_string())?;
        }
        let final_log_path = final_root.join(INSTALL_LOG_FILE).to_string_lossy().into_owned();
        write_status_file(
            &bundle_root,
            image.id.as_str(),
            AndroidImageDistributionStage::Completed,
            0,
            None,
            "Android CI image kurulumu tamamlandi.",
            &final_log_path,
        )?;
        self.ensure_not_cancelled(image.id.as_str())?;

        replace_bundle_atomically(&bundle_root, &final_root, &backup_root)?;
        remove_if_exists(&stage_root)?;
        remove_if_exists(&backup_root)?;
        let artifacts = scan_artifacts(&final_root)?;

        Ok(AndroidImageDistributionRegistration {
            source_revision: Some(format!("android-ci:{}", discovery.build_id)),
            android_release: discovery.android_release,
            sdk_level: discovery.sdk_level,
            runtime_kind: AndroidImageRuntimeKind::QemuComposite,
            artifacts,
            capabilities: AndroidImageCapabilities::stock_cuttlefish_x86_64(),
        })
    }

    fn resolve_latest_build(&self, image: &AndroidImage) -> Result<BuildDiscovery, String> {
        let source = self.source_resolver
            .lock()
            .map_err(|_| String::from("Android source resolver lock poisoned"))?
            .resolve(image)
            .map_err(|error| format!("Android source resolver failed: {error:?}"))?;
        self.append_log(
            image.id.as_str(),
            &format!(
                "Android source resolver secildi: provider={} release={} branch={} target={} build={} cache_key={}",
                source.provider,
                source.android_release.as_deref().unwrap_or("latest"),
                source.branch,
                source.target,
                source.build_id,
                source.cache_key
            ),
        )?;
        Ok(BuildDiscovery {
            base_url: source.base_url,
            branch: source.branch,
            target: source.target,
            build_id: source.build_id,
            artifact_base_url: source.artifact_base_url,
            device_artifact_name: source.device_artifact_name,
            host_artifact_name: source.host_artifact_name,
            android_release: source.android_release,
            sdk_level: source.sdk_level,
            host_package_available: source.host_package_available,
            allow_device_bootloader_fallback: source.allow_device_bootloader_fallback,
        })
    }

    fn prepare_stage_for_build(&self, image_id: &str, build_id: &str) -> Result<(), String> {
        let stage_root = self.stage_root(image_id);
        let marker_path = stage_root.join(BUILD_ID_FILE);
        let current = fs::read_to_string(&marker_path).ok().map(|value| value.trim().to_owned());
        if current.as_deref() != Some(build_id) {
            for name in ["downloads", "device", "host", "bundle"] {
                remove_if_exists(&stage_root.join(name))?;
            }
            fs::write(&marker_path, format!("{build_id}\n")).map_err(|error| error.to_string())?;
            self.append_log(image_id, &format!("Yeni Android CI build secildi: {build_id}"))?;
        } else {
            for name in ["device", "host", "bundle"] {
                remove_if_exists(&stage_root.join(name))?;
            }
            self.append_log(image_id, &format!("Ayni build icin yarim indirmeler devam ettirilecek: {build_id}"))?;
        }
        Ok(())
    }

    fn download_device_archive(
        &self,
        image_id: &str,
        discovery: &BuildDiscovery,
        destination: &Path,
    ) -> Result<(String, String), String> {
        let name = discovery.device_artifact_name.clone();
        let url = format!("{}/raw/{name}", discovery.artifact_base_url);
        self.download_cached(
            image_id,
            AndroidImageDistributionStage::DownloadingDevice,
            "Android Cuttlefish device image indiriliyor.",
            &url,
            destination,
            &discovery.branch,
            &discovery.target,
            &discovery.build_id,
            &name,
        )?;
        Ok((url, name))
    }

    fn download_cached(
        &self,
        image_id: &str,
        stage: AndroidImageDistributionStage,
        detail: &str,
        url: &str,
        destination: &Path,
        branch: &str,
        target: &str,
        build_id: &str,
        artifact_name: &str,
    ) -> Result<(), String> {
        let request = ArtifactCacheRequest {
            source_key: format!(
                "android-ci:{}:{}:{}:{}",
                branch, target, build_id, artifact_name
            ),
            source_url: url.to_owned(),
            immutable: true,
            pinned: false,
            validators: ArtifactSourceValidators::default(),
        };

        let Some(cache) = &self.artifact_cache else {
            return self.download(image_id, stage, detail, url, destination);
        };

        {
            let mut client = cache.lock().map_err(|_| String::from("Artifact Cache lock poisoned"))?;
            client.acquire_source_lock(&request.source_key)?;
        }
        let operation = (|| {
            let restored = {
                let mut client = cache.lock().map_err(|_| String::from("Artifact Cache lock poisoned"))?;
                client.restore(&request, destination)
            };
            match restored {
                Ok(Some(record)) => {
                    self.record_phase(image_id, stage, record.size_bytes, Some(record.size_bytes), detail)?;
                    let _ = self.append_log(
                        image_id,
                        &format!("Artifact Cache HIT: {} sha256={}", artifact_name, record.sha256),
                    );
                    return Ok(());
                }
                Ok(None) => {
                    let _ = self.append_log(image_id, &format!("Artifact Cache MISS: {artifact_name}"));
                }
                Err(error) => {
                    let _ = self.append_log(
                        image_id,
                        &format!("Artifact Cache restore kullanilamadi; ag indirmesine geciliyor: {error}"),
                    );
                }
            }

            self.download(image_id, stage, detail, url, destination)?;
            let stored = {
                let mut client = cache.lock().map_err(|_| String::from("Artifact Cache lock poisoned"))?;
                client.store(&request, destination)
            };
            match stored {
                Ok(record) => {
                    let _ = self.append_log(
                        image_id,
                        &format!("Artifact Cache STORE: {} sha256={}", artifact_name, record.sha256),
                    );
                }
                Err(error) => {
                    let _ = self.append_log(
                        image_id,
                        &format!("Artifact Cache store basarisiz; kurulum devam ediyor: {error}"),
                    );
                }
            }
            Ok(())
        })();
        let release = cache
            .lock()
            .map_err(|_| String::from("Artifact Cache lock poisoned"))?
            .release_source_lock(&request.source_key);
        match (operation, release) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(error), Ok(())) => Err(error),
            (Ok(()), Err(error)) => Err(format!("Artifact Cache source lock release failed: {error}")),
            (Err(error), Err(release_error)) => Err(format!("{error}; source lock release failed: {release_error}")),
        }
    }

    fn download(
        &self,
        image_id: &str,
        stage: AndroidImageDistributionStage,
        detail: &str,
        url: &str,
        destination: &Path,
    ) -> Result<(), String> {
        let total_bytes = self.content_length(url);
        let existing_bytes = file_size(destination);
        self.record_phase(image_id, stage, existing_bytes, total_bytes, detail)?;

        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }

        if existing_bytes > 0 {
            match self.download_attempt(image_id, stage, detail, url, destination, total_bytes, true) {
                Ok(()) => return Ok(()),
                Err(resume_error) if resume_error == CANCELLED_ERROR_CODE => return Err(resume_error),
                Err(resume_error) => {
                    self.append_log(
                        image_id,
                        &format!(
                            "Resume basarisiz; partial dosya silinip sifirdan tekrar denenecek: {resume_error}"
                        ),
                    )?;
                    remove_if_exists(destination)?;
                    self.write_progress(image_id, stage, 0, total_bytes, detail)?;
                }
            }
        }

        self.download_attempt(image_id, stage, detail, url, destination, total_bytes, false)
    }

    fn download_attempt(
        &self,
        image_id: &str,
        stage: AndroidImageDistributionStage,
        detail: &str,
        url: &str,
        destination: &Path,
        total_bytes: Option<u64>,
        resume: bool,
    ) -> Result<(), String> {
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.stage_log_path(image_id))
            .map_err(|error| error.to_string())?;
        let stderr_file = log_file.try_clone().map_err(|error| error.to_string())?;

        let mut command = self.http.build_file_download_command(
            url,
            destination,
            resume,
            &android_ci_request_context(url),
        );
        command.stdout(Stdio::null()).stderr(Stdio::from(stderr_file));

        let mut child = command.spawn().map_err(|error| format!("curl launch failed: {error}"))?;
        loop {
            if self.cancel_request_path(image_id).is_file() {
                let _ = child.kill();
                let _ = child.wait();
                let _ = self.append_log(image_id, "Kullanici iptal istegi nedeniyle aktif indirme durduruldu.");
                return Err(String::from(CANCELLED_ERROR_CODE));
            }
            match child.try_wait().map_err(|error| error.to_string())? {
                Some(status) => {
                    let downloaded_bytes = file_size(destination);
                    self.write_progress(image_id, stage, downloaded_bytes, total_bytes, detail)?;
                    if !status.success() {
                        return Err(format!(
                            "download {url} failed with {status}; log: {}",
                            self.stage_log_path(image_id).display()
                        ));
                    }
                    if downloaded_bytes == 0 {
                        return Err(format!("download returned an empty file: {url}"));
                    }
                    return Ok(());
                }
                None => {
                    let downloaded_bytes = file_size(destination);
                    self.write_progress(image_id, stage, downloaded_bytes, total_bytes, detail)?;
                    thread::sleep(Duration::from_millis(DOWNLOAD_POLL_INTERVAL_MS));
                }
            }
        }
    }

    fn content_length(&self, url: &str) -> Option<u64> {
        let headers = self
            .http
            .fetch_headers_with_context(url, &android_ci_request_context(url))
            .ok()?;
        parse_content_length(&headers)
    }

    fn cancel_request_path(&self, image_id: &str) -> PathBuf {
        self.stage_root(image_id).join(CANCEL_REQUEST_FILE)
    }

    fn ensure_not_cancelled(&self, image_id: &str) -> Result<(), String> {
        if self.cancel_request_path(image_id).is_file() {
            return Err(String::from(CANCELLED_ERROR_CODE));
        }
        Ok(())
    }

    fn validate_free_disk(&self) -> Result<(), String> {
        let required_bytes = self
            .settings
            .minimum_free_disk_gib
            .checked_mul(BYTES_PER_GIB)
            .ok_or_else(|| String::from("android image minimum free disk value overflow"))?;
        let available_bytes = available_free_bytes(&self.settings.output_root)?;
        if available_bytes < required_bytes {
            return Err(format!(
                "insufficient free disk for Android image install: required={} GiB available={} GiB",
                self.settings.minimum_free_disk_gib,
                available_bytes / BYTES_PER_GIB
            ));
        }
        Ok(())
    }

    fn stage_root(&self, image_id: &str) -> PathBuf {
        self.settings.output_root.join(format!(".{image_id}.installing"))
    }

    fn final_root(&self, image_id: &str) -> PathBuf { self.settings.output_root.join(image_id) }

    fn stage_log_path(&self, image_id: &str) -> PathBuf { self.stage_root(image_id).join(INSTALL_LOG_FILE) }

    fn initialize_log(&self, image_id: &str) -> Result<(), String> {
        let path = self.stage_log_path(image_id);
        if path.is_file() { return Ok(()); }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let version = env!("CARGO_PKG_VERSION");
        let header = format!(
            "# 📄 Dosya Yolu: /turkuazvm/data/android-image-builds/.{image_id}.installing/{INSTALL_LOG_FILE}\n# 📌 Amac: Android CI otomatik kurulum calisma kaydini saklar\n# 📌 Modul - Log\n# Version: {version}\n# Aciklama: Discovery, download, extract, assemble ve finalization adimlarini kaydeder\n# Bagimli Oldugu Katman: Tool | View\n\n"
        );
        fs::write(path, header).map_err(|error| error.to_string())
    }

    fn append_log(&self, image_id: &str, message: &str) -> Result<(), String> {
        self.initialize_log(image_id)?;
        let mut file = OpenOptions::new()
            .append(true)
            .open(self.stage_log_path(image_id))
            .map_err(|error| error.to_string())?;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or_default();
        writeln!(file, "[{timestamp}] {message}").map_err(|error| error.to_string())
    }

    fn record_phase(
        &self,
        image_id: &str,
        stage: AndroidImageDistributionStage,
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
        detail: &str,
    ) -> Result<(), String> {
        self.append_log(image_id, detail)?;
        self.write_progress(image_id, stage, downloaded_bytes, total_bytes, detail)
    }

    fn write_progress(
        &self,
        image_id: &str,
        stage: AndroidImageDistributionStage,
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
        detail: &str,
    ) -> Result<(), String> {
        let stage_root = self.stage_root(image_id);
        fs::create_dir_all(&stage_root).map_err(|error| error.to_string())?;
        let log_path = self.stage_log_path(image_id).to_string_lossy().into_owned();
        write_status_file(
            &stage_root,
            image_id,
            stage,
            downloaded_bytes,
            total_bytes,
            detail,
            &log_path,
        )
    }

    fn read_progress(&self, image: &AndroidImage) -> Result<Option<AndroidImageDistributionProgress>, String> {
        let stage_status = self.stage_root(image.id.as_str()).join(INSTALL_STATUS_FILE);
        let final_status = self.final_root(image.id.as_str()).join(INSTALL_STATUS_FILE);
        let path = if stage_status.is_file() {
            stage_status
        } else if final_status.is_file() {
            final_status
        } else {
            return Ok(None);
        };
        let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
        let dto: InstallStatusDto = serde_yaml_ng::from_str(&content).map_err(|error| error.to_string())?;
        if !matches!(dto.schema_version, LEGACY_INSTALL_STATUS_SCHEMA_VERSION | INSTALL_STATUS_SCHEMA_VERSION) {
            return Err(String::from("unsupported Android install status schema"));
        }
        let now = epoch_seconds();
        let updated_at = dto.updated_at_epoch_seconds.unwrap_or(now);
        let stage_started_at = dto.stage_started_at_epoch_seconds.unwrap_or(updated_at);
        let elapsed_seconds = updated_at.saturating_sub(stage_started_at);
        let stage_start_bytes = dto.stage_start_downloaded_bytes.unwrap_or(0);
        let transferred_bytes = dto.downloaded_bytes.saturating_sub(stage_start_bytes);
        let bytes_per_second = (elapsed_seconds > 0 && transferred_bytes > 0)
            .then(|| transferred_bytes / elapsed_seconds)
            .filter(|value| *value > 0);
        let eta_seconds = match (dto.total_bytes, bytes_per_second) {
            (Some(total), Some(rate)) if total > dto.downloaded_bytes && rate > 0 => {
                Some((total - dto.downloaded_bytes) / rate)
            }
            _ => None,
        };
        let mut progress = AndroidImageDistributionProgress {
            stage: parse_stage(&dto.stage)?,
            downloaded_bytes: dto.downloaded_bytes,
            total_bytes: dto.total_bytes,
            detail: dto.detail,
            log_path: dto.log_path,
            elapsed_seconds,
            bytes_per_second,
            eta_seconds,
        };
        if image.state == turkuazvm_android_image::domain::image::AndroidImageState::Failed
            && progress.stage != AndroidImageDistributionStage::Failed
        {
            progress.stage = AndroidImageDistributionStage::Failed;
            if let Some(error) = &image.last_error {
                progress.detail = error.clone();
            }
        }
        Ok(Some(progress))
    }
}

impl AndroidImageDistributionPort for AndroidCiDistributionTool {
    fn prepare_install(&self, image: &AndroidImage) -> Result<(), AndroidImageDistributionError> {
        remove_if_exists(&self.cancel_request_path(image.id.as_str()))
            .map_err(AndroidImageDistributionError::Install)
    }

    fn install(&self, image: &AndroidImage) -> Result<AndroidImageDistributionRegistration, AndroidImageDistributionError> {
        match self.install_inner(image) {
            Ok(registration) => Ok(registration),
            Err(error) if error == CANCELLED_ERROR_CODE => {
                let detail = "Android image kurulumu kullanici tarafindan iptal edildi.";
                let _ = self.record_phase(
                    image.id.as_str(),
                    AndroidImageDistributionStage::Failed,
                    0,
                    None,
                    detail,
                );
                let _ = remove_if_exists(&self.cancel_request_path(image.id.as_str()));
                Err(AndroidImageDistributionError::Cancelled)
            }
            Err(error) => {
                let _ = self.record_phase(
                    image.id.as_str(),
                    AndroidImageDistributionStage::Failed,
                    0,
                    None,
                    &error,
                );
                Err(AndroidImageDistributionError::Install(error))
            }
        }
    }

    fn progress(
        &self,
        image: &AndroidImage,
    ) -> Result<Option<AndroidImageDistributionProgress>, AndroidImageDistributionError> {
        self.read_progress(image).map_err(AndroidImageDistributionError::Install)
    }

    fn cancel(&self, image: &AndroidImage) -> Result<(), AndroidImageDistributionError> {
        let stage_root = self.stage_root(image.id.as_str());
        fs::create_dir_all(&stage_root).map_err(|error| AndroidImageDistributionError::Install(error.to_string()))?;
        let marker = format!(
            "# 📄 Dosya Yolu: /turkuazvm/data/android-image-builds/.{}.installing/{}\n# 📌 Amac: Android image otomatik kurulum iptal istegini kalici olarak isaretler\n# 📌 Modul - Flag\n# Version: {}\n# Aciklama: Background worker aktif indirme veya sonraki fazda bu marker'i gorup kontrollu iptal eder\n# Bagimli Oldugu Katman: Service | Tool\n",
            image.id.as_str(),
            CANCEL_REQUEST_FILE,
            env!("CARGO_PKG_VERSION")
        );
        fs::write(self.cancel_request_path(image.id.as_str()), marker)
            .map_err(|error| AndroidImageDistributionError::Install(error.to_string()))?;
        let current = self.read_progress(image).ok().flatten();
        let downloaded = current.as_ref().map(|value| value.downloaded_bytes).unwrap_or_default();
        let total = current.and_then(|value| value.total_bytes);
        self.record_phase(
            image.id.as_str(),
            AndroidImageDistributionStage::Cancelling,
            downloaded,
            total,
            "Android image kurulumu iptal ediliyor.",
        )
        .map_err(AndroidImageDistributionError::Install)
    }

    fn cleanup(&self, image: &AndroidImage) -> Result<(), AndroidImageDistributionError> {
        remove_if_exists(&self.stage_root(image.id.as_str()))
            .map_err(AndroidImageDistributionError::Install)?;
        remove_if_exists(&self.settings.output_root.join(format!(".{}.backup", image.id.as_str())))
            .map_err(AndroidImageDistributionError::Install)
    }
}

#[derive(Debug)]
struct BuildDiscovery {
    base_url: String,
    branch: String,
    target: String,
    build_id: String,
    artifact_base_url: String,
    device_artifact_name: String,
    host_artifact_name: String,
    android_release: Option<String>,
    sdk_level: Option<u32>,
    host_package_available: bool,
    allow_device_bootloader_fallback: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct InstallStatusDto {
    schema_version: u16,
    stage: String,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
    detail: String,
    log_path: String,
    #[serde(default)]
    install_started_at_epoch_seconds: Option<u64>,
    #[serde(default)]
    stage_started_at_epoch_seconds: Option<u64>,
    #[serde(default)]
    stage_start_downloaded_bytes: Option<u64>,
    #[serde(default)]
    updated_at_epoch_seconds: Option<u64>,
}

#[derive(Clone, Copy)]
enum ArchiveKind { Zip, TarGz }

fn write_status_file(
    root: &Path,
    image_id: &str,
    stage: AndroidImageDistributionStage,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
    detail: &str,
    log_path: &str,
) -> Result<(), String> {
    fs::create_dir_all(root).map_err(|error| error.to_string())?;
    let path = root.join(INSTALL_STATUS_FILE);
    let now = epoch_seconds();
    let stage_value = stage_code(stage).to_owned();
    let previous = fs::read_to_string(&path)
        .ok()
        .and_then(|content| serde_yaml_ng::from_str::<InstallStatusDto>(&content).ok());
    let install_started_at = previous
        .as_ref()
        .and_then(|value| value.install_started_at_epoch_seconds)
        .unwrap_or(now);
    let same_stage = previous.as_ref().is_some_and(|value| value.stage == stage_value);
    let stage_started_at = if same_stage {
        previous.as_ref().and_then(|value| value.stage_started_at_epoch_seconds).unwrap_or(now)
    } else {
        now
    };
    let stage_start_downloaded_bytes = if same_stage {
        previous.as_ref().and_then(|value| value.stage_start_downloaded_bytes).unwrap_or(downloaded_bytes)
    } else {
        downloaded_bytes
    };
    let dto = InstallStatusDto {
        schema_version: INSTALL_STATUS_SCHEMA_VERSION,
        stage: stage_value,
        downloaded_bytes,
        total_bytes,
        detail: detail.to_owned(),
        log_path: log_path.to_owned(),
        install_started_at_epoch_seconds: Some(install_started_at),
        stage_started_at_epoch_seconds: Some(stage_started_at),
        stage_start_downloaded_bytes: Some(stage_start_downloaded_bytes),
        updated_at_epoch_seconds: Some(now),
    };
    let yaml = serde_yaml_ng::to_string(&dto).map_err(|error| error.to_string())?;
    let status_root = root.file_name().and_then(|value| value.to_str()).unwrap_or(image_id);
    let version = env!("CARGO_PKG_VERSION");
    let header = format!(
        "# 📄 Dosya Yolu: /turkuazvm/data/android-image-builds/{status_root}/{INSTALL_STATUS_FILE}\n# 📌 Amac: Android image otomatik kurulum ilerleme durumunu saklar\n# 📌 Modul - YAML\n# Version: {version}\n# Aciklama: Kurulum asamasi, indirilen byte ve log yolunu Desktop polling icin kaydeder\n# Bagimli Oldugu Katman: Tool | View\n"
    );
    let temp = root.join(format!(".{INSTALL_STATUS_FILE}.tmp"));
    fs::write(&temp, format!("{header}{yaml}")).map_err(|error| error.to_string())?;
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

fn epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn android_http_tool(settings: &AndroidCiDistributionSettings) -> HttpDownloadTool {
    HttpDownloadTool::new(HttpDownloadSettings {
        curl_binary: settings.curl_binary.clone(),
        connect_timeout_seconds: settings.download_connect_timeout_seconds,
        retry_count: settings.download_retry_count,
        retry_delay_seconds: settings.download_retry_delay_seconds,
    })
}

fn android_ci_request_context(url: &str) -> HttpRequestContext {
    let base_url = android_ci_base_from_url(url);
    HttpRequestContext {
        user_agent: Some(ANDROID_CI_BROWSER_USER_AGENT.to_owned()),
        referer: Some(format!("{}/", base_url.trim_end_matches('/'))),
        headers: vec![ANDROID_CI_ACCEPT_ARTIFACT.to_owned()],
    }
}

fn android_ci_base_from_url(url: &str) -> &str {
    url.split("/builds/").next().unwrap_or(url)
}

fn ensure_success(operation: &str, output: &Output) -> Result<(), String> {
    if output.status.success() { return Ok(()); }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    Err(format!("{operation} failed with {}: {stderr}", output.status))
}

fn parse_content_length(headers: &str) -> Option<u64> {
    headers
        .lines()
        .filter_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.trim().eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<u64>().ok())
                .flatten()
        })
        .last()
}

fn validate_archive_entries(path: &Path, kind: ArchiveKind) -> Result<(), String> {
    let output = match kind {
        ArchiveKind::TarGz => Command::new("tar").args(["-tzf"]).arg(path).output(),
        ArchiveKind::Zip => {
            let first = Command::new("tar").args(["-tf"]).arg(path).output().map_err(|error| error.to_string())?;
            if first.status.success() { return validate_archive_listing(&String::from_utf8_lossy(&first.stdout)); }
            Command::new("unzip").args(["-Z1"]).arg(path).output()
        }
    }.map_err(|error| error.to_string())?;
    ensure_success("archive list", &output)?;
    validate_archive_listing(&String::from_utf8_lossy(&output.stdout))
}

fn validate_archive_listing(listing: &str) -> Result<(), String> {
    let mut count = 0_usize;
    for line in listing.lines() {
        let normalized = line.trim().replace('\\', "/");
        if normalized.is_empty() { continue; }
        count += 1;
        if normalized.starts_with('/') || normalized.starts_with("//") || normalized.get(1..2) == Some(":") {
            return Err(format!("archive contains absolute path: {normalized}"));
        }
        let path = Path::new(&normalized);
        if path.components().any(|component| matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_))) {
            return Err(format!("archive contains unsafe path: {normalized}"));
        }
    }
    if count == 0 { return Err(String::from("archive contains no entries")); }
    Ok(())
}

fn extract_zip(archive: &Path, destination: &Path) -> Result<(), String> {
    let tar_result = Command::new("tar").args(["-xf"]).arg(archive).arg("-C").arg(destination).output();
    if let Ok(output) = tar_result {
        if output.status.success() { return Ok(()); }
    }
    let output = Command::new("unzip").arg("-q").arg(archive).arg("-d").arg(destination).output()
        .map_err(|error| format!("zip extractor unavailable: {error}"))?;
    ensure_success("zip extraction", &output)
}

fn extract_x86_64_bootloader(archive: &Path, destination: &Path) -> Result<(), String> {
    let listing_output = Command::new("tar")
        .args(["-tzf"])
        .arg(archive)
        .output()
        .map_err(|error| format!("tar unavailable: {error}"))?;
    ensure_success("host package listing", &listing_output)?;
    let listing = String::from_utf8_lossy(&listing_output.stdout);
    let member = select_x86_64_bootloader_member(&listing)
        .ok_or_else(|| String::from("cvd host package does not contain x86_64 QEMU bootloader"))?;

    let output = Command::new("tar")
        .args(["-xOzf"])
        .arg(archive)
        .arg("--")
        .arg(&member)
        .output()
        .map_err(|error| format!("tar bootloader stream unavailable: {error}"))?;
    ensure_success("host x86_64 bootloader stream", &output)?;
    if output.stdout.is_empty() {
        return Err(format!("cvd host package x86_64 bootloader is empty: {member}"));
    }
    fs::write(destination, &output.stdout).map_err(|error| error.to_string())
}

fn copy_x86_64_bootloader_from_device(root: &Path, destination: &Path) -> Result<(), String> {
    let mut stack = vec![root.to_path_buf()];
    let mut candidates = Vec::new();
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let file_type = entry.file_type().map_err(|error| error.to_string())?;
            if file_type.is_symlink() { continue; }
            let path = entry.path();
            if file_type.is_dir() {
                stack.push(path);
                continue;
            }
            if !file_type.is_file() { continue; }
            let normalized = path.to_string_lossy().replace('\\', "/");
            let lower = normalized.to_ascii_lowercase();
            let name = lower.rsplit('/').next().unwrap_or_default();
            let recognized = matches!(name, "bootloader.qemu" | "u-boot.rom" | "bootloader_qemu_x86_64");
            if !recognized || lower.contains("aarch64") || lower.contains("riscv") || lower.contains("qemu_arm") {
                continue;
            }
            let rank = bootloader_member_rank(&lower, name);
            if rank < 10 { candidates.push((rank, path)); }
        }
    }
    candidates.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    let source = candidates.into_iter().next().map(|(_, path)| path)
        .ok_or_else(|| String::from("legacy Android device archive does not contain an x86_64 QEMU bootloader"))?;
    fs::copy(source, destination).map_err(|error| error.to_string())?;
    Ok(())
}

fn select_x86_64_bootloader_member(listing: &str) -> Option<String> {
    let mut candidates = listing
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.ends_with('/'))
        .filter_map(|line| {
            let normalized = line.replace('\\', "/");
            let lower = normalized.to_ascii_lowercase();
            let name = lower.rsplit('/').next().unwrap_or_default();
            let recognized = matches!(name, "bootloader.qemu" | "u-boot.rom" | "bootloader_qemu_x86_64");
            if !recognized
                || lower.contains("aarch64")
                || lower.contains("riscv")
                || lower.contains("qemu_arm")
            {
                return None;
            }
            Some((bootloader_member_rank(&lower, name), line.to_owned()))
        })
        .filter(|(rank, _)| *rank < 10)
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    candidates.into_iter().next().map(|(_, member)| member)
}

fn bootloader_member_rank(normalized: &str, name: &str) -> u8 {
    if normalized.contains("bootloader_qemu_x86_64") { return 0; }
    if normalized.contains("qemu_x86_64") || normalized.contains("qemu/x86_64") { return 1; }
    if name == "bootloader_qemu_x86_64" { return 2; }
    if name == "bootloader.qemu" && normalized.contains("x86_64") { return 3; }
    if name == "u-boot.rom" && normalized.contains("x86_64") && normalized.contains("qemu") { return 4; }
    if name == "bootloader.qemu" { return 5; }
    10
}

fn copy_named_artifact(source_root: &Path, bundle_root: &Path, name: &str, required: bool) -> Result<(), String> {
    match find_file_named(source_root, name) {
        Some(source) => {
            fs::copy(source, bundle_root.join(name)).map_err(|error| error.to_string())?;
            Ok(())
        }
        None if required => Err(format!("required Android CI artifact is missing: {name}")),
        None => Ok(()),
    }
}

fn find_file_named(root: &Path, name: &str) -> Option<PathBuf> {
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(dir).ok()? {
            let entry = entry.ok()?;
            let file_type = entry.file_type().ok()?;
            if file_type.is_symlink() { continue; }
            let path = entry.path();
            if file_type.is_dir() { stack.push(path); continue; }
            if file_type.is_file() && entry.file_name().to_string_lossy() == name { return Some(path); }
        }
    }
    None
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

fn write_distribution_metadata(
    root: &Path,
    image_id: &str,
    base_url: &str,
    branch: &str,
    target: &str,
    build_id: &str,
    image_name: &str,
    image_url: &str,
    image_sha256: &str,
    host_url: Option<&str>,
    host_sha256: Option<&str>,
    bootloader_source: &str,
) -> Result<(), String> {
    let version = env!("CARGO_PKG_VERSION");
    let host_url = host_url.unwrap_or_default();
    let host_sha256 = host_sha256.unwrap_or_default();
    let content = format!(
        "# 📄 Dosya Yolu: /turkuazvm/data/android-image-builds/{image_id}/distribution.yml\n# 📌 Amac: Otomatik Android CI kurulum provenance bilgisini saklar\n# 📌 Modul - YAML\n# Version: {version}\n# Aciklama: Device/host paket URL-SHA256 ve bootloader kaynagini kaydeder\n# Bagimli Oldugu Katman: Repo | Tool\nbase_url: \"{base_url}\"\nbranch: \"{branch}\"\ntarget: \"{target}\"\nbuild_id: \"{build_id}\"\nimage_archive: \"{image_name}\"\nimage_url: \"{image_url}\"\nimage_sha256: \"{image_sha256}\"\nhost_url: \"{host_url}\"\nhost_sha256: \"{host_sha256}\"\nbootloader_source: \"{bootloader_source}\"\n"
    );
    fs::write(root.join("distribution.yml"), content).map_err(|error| error.to_string())
}

fn scan_artifacts(root: &Path) -> Result<Vec<AndroidImageArtifact>, String> {
    let mut artifacts = Vec::new();
    for entry in fs::read_dir(root).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let file_type = entry.file_type().map_err(|error| error.to_string())?;
        if file_type.is_symlink() { return Err(String::from("bundle artifact symlinks are not allowed")); }
        if !file_type.is_file() { continue; }
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(role) = artifact_role(&name) else { continue; };
        let path = entry.path();
        let size = fs::metadata(&path).map_err(|error| error.to_string())?.len();
        if size == 0 { continue; }
        let sha256 = sha256_file(&path)?;
        artifacts.push(AndroidImageArtifact::create(role, name, size, sha256).map_err(|error| format!("{error:?}"))?);
    }
    artifacts.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(artifacts)
}

fn artifact_role(name: &str) -> Option<AndroidImageArtifactRole> {
    match name {
        "boot.img" => Some(AndroidImageArtifactRole::Boot),
        "init_boot.img" => Some(AndroidImageArtifactRole::InitBoot),
        "vendor_boot.img" => Some(AndroidImageArtifactRole::VendorBoot),
        "super.img" => Some(AndroidImageArtifactRole::Super),
        "userdata.img" => Some(AndroidImageArtifactRole::Userdata),
        "vbmeta.img" => Some(AndroidImageArtifactRole::Vbmeta),
        "vbmeta_system.img" => Some(AndroidImageArtifactRole::VbmetaSystem),
        "metadata.img" => Some(AndroidImageArtifactRole::Metadata),
        "misc.img" => Some(AndroidImageArtifactRole::Misc),
        "bootloader.qemu" => Some(AndroidImageArtifactRole::Bootloader),
        "composite.img" => Some(AndroidImageArtifactRole::CompositeDisk),
        "kernel" | "Image" => Some(AndroidImageArtifactRole::Kernel),
        value if value.ends_with(".img") => Some(AndroidImageArtifactRole::Other),
        _ => None,
    }
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

fn file_size(path: &Path) -> u64 {
    fs::metadata(path).map(|metadata| metadata.len()).unwrap_or_default()
}

fn available_free_bytes(path: &Path) -> Result<u64, String> {
    fs::create_dir_all(path).map_err(|error| format!("free disk path prepare failed: {error}"))?;
    fs2::available_space(path).map_err(|error| format!("free disk probe failed: {error}"))
}

#[cfg(test)]
mod tests {
    use super::{
        parse_content_length, parse_stage, select_x86_64_bootloader_member, validate_archive_listing,
        ANDROID_CI_BROWSER_USER_AGENT,
    };
    use turkuazvm_android_image::ports::android_image_distribution_port::AndroidImageDistributionStage;

    #[test]
    fn host_package_bootloader_selector_prefers_x86_64_qemu_member() {
        let listing = "./bin/launch_cvd\n./etc/bootloader_qemu_aarch64/bootloader.qemu\n./etc/bootloader_qemu_x86_64/bootloader.qemu\n./etc/bootloader_crosvm_x86_64/bootloader.qemu\n";
        assert_eq!(
            select_x86_64_bootloader_member(listing).as_deref(),
            Some("./etc/bootloader_qemu_x86_64/bootloader.qemu")
        );
    }

    #[test]
    fn host_package_bootloader_selector_rejects_non_x86_qemu_candidates() {
        let listing = "./etc/bootloader_qemu_aarch64/bootloader.qemu\n./etc/bootloader_qemu_riscv64/bootloader.qemu\n";
        assert!(select_x86_64_bootloader_member(listing).is_none());
    }

    #[test]
    fn android_ci_request_identity_is_browser_compatible() {
        assert!(ANDROID_CI_BROWSER_USER_AGENT.starts_with("Mozilla/5.0"));
    }

    #[test]
    fn content_length_uses_last_redirect_header_value() {
        let headers = "HTTP/1.1 302 Found\r\nContent-Length: 12\r\n\r\nHTTP/2 200\r\ncontent-length: 4096\r\n";
        assert_eq!(parse_content_length(headers), Some(4096));
    }

    #[test]
    fn cancelling_stage_round_trip_is_supported() {
        assert_eq!(
            parse_stage("cancelling").expect("cancelling stage parse failed"),
            AndroidImageDistributionStage::Cancelling
        );
    }

    #[test]
    fn archive_listing_rejects_parent_traversal() {
        assert!(validate_archive_listing("system.img\n../escape.img\n").is_err());
    }
}
