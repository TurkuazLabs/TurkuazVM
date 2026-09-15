// # 📄 Dosya Yolu: /turkuazvm/apps/engine/src/services/installer_media_download_application_service.rs
// # 📌 Amac: Installer media kaynagini runtime resolver ile bulur, resmi mirrorlardan indirir ve SHA-256 ile dogrular
// # 📌 Modul - Rust
// # Version: 0.40.1
// # Aciklama: Resolver zincirini tek HTTP Tool ile indirir; ayni mirror resume eder, mirror failover oncesinde partial dosyayi temizler
// # Bagimli Oldugu Katman: Service | Repo | Tool

use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use sha2::{Digest, Sha256};
use turkuazvm_guest::tools::http_download_tool::{HttpDownloadSettings, HttpDownloadTool};
use turkuazvm_guest::tools::linux_installer_media_provider_tool::LinuxInstallerMediaProviderTool;
use turkuazvm_guest_catalog::domain::installer_media_source::{
    InstallerMediaResolverPolicy, InstallerMediaSourceRequest, ResolvedInstallerMediaSource,
};
use turkuazvm_guest_catalog::ports::installer_media_source_resolver_port::InstallerMediaSourceResolverPort;
use turkuazvm_guest_catalog::services::installer_media_source_resolver_service::InstallerMediaSourceResolverService;
use turkuazvm_repositories::repositories::yaml_installer_media_source_cache_repository::YamlInstallerMediaSourceCacheRepository;

use crate::services::guest_catalog_application_service::GuestCatalogApplicationService;

const DOWNLOAD_POLL_INTERVAL_MS: u64 = 120;

type InstallerMediaResolver = InstallerMediaSourceResolverService<
    LinuxInstallerMediaProviderTool,
    YamlInstallerMediaSourceCacheRepository,
>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerMediaDownloadState {
    NotStarted,
    Downloading,
    Cancelling,
    Cancelled,
    Verifying,
    Ready,
    Failed,
}

impl InstallerMediaDownloadState {
    pub const fn code(self) -> &'static str {
        match self {
            Self::NotStarted => "not_started",
            Self::Downloading => "downloading",
            Self::Cancelling => "cancelling",
            Self::Cancelled => "cancelled",
            Self::Verifying => "verifying",
            Self::Ready => "ready",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallerMediaDownloadStatus {
    pub guest_template_id: String,
    pub media_id: String,
    pub state: InstallerMediaDownloadState,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub local_path: Option<PathBuf>,
    pub sha256: Option<String>,
    pub detail: String,
}

#[derive(Debug, Clone)]
struct DownloadJob {
    state: InstallerMediaDownloadState,
    part_path: PathBuf,
    final_path: PathBuf,
    total_bytes: Option<u64>,
    sha256: Option<String>,
    detail: String,
    cancel_requested: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DownloadRunOutcome {
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallerMediaDownloadError {
    Catalog(String),
    Source(String),
    UnsupportedMode,
    MissingFilename,
    Busy,
    Download(String),
    NotReady,
}

pub struct InstallerMediaDownloadApplicationService {
    catalog: GuestCatalogApplicationService,
    download_root: PathBuf,
    resolver: Arc<Mutex<InstallerMediaResolver>>,
    http: Arc<HttpDownloadTool>,
    jobs: Arc<Mutex<HashMap<String, DownloadJob>>>,
}

impl InstallerMediaDownloadApplicationService {
    pub fn new(
        catalog_path: PathBuf,
        download_root: PathBuf,
        source_cache_path: PathBuf,
        resolver_policy: InstallerMediaResolverPolicy,
        http_settings: HttpDownloadSettings,
    ) -> Self {
        let http = Arc::new(HttpDownloadTool::new(http_settings));
        let provider = LinuxInstallerMediaProviderTool::new(Arc::clone(&http));
        let cache = YamlInstallerMediaSourceCacheRepository::new(source_cache_path);
        let resolver = InstallerMediaSourceResolverService::new(resolver_policy, provider, cache);
        Self {
            catalog: GuestCatalogApplicationService::new(catalog_path),
            download_root,
            resolver: Arc::new(Mutex::new(resolver)),
            http,
            jobs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn start(
        &self,
        guest_template_id: &str,
        media_id: &str,
    ) -> Result<InstallerMediaDownloadStatus, InstallerMediaDownloadError> {
        let source = self.resolve_source(guest_template_id, media_id)?;
        let filename = source.filename.clone();
        if filename.trim().is_empty() {
            return Err(InstallerMediaDownloadError::MissingFilename);
        }
        let final_dir = self.download_root.join(guest_template_id).join(media_id);
        let final_path = final_dir.join(&filename);
        let part_path = final_dir.join(format!("{filename}.part"));

        if final_path.is_file() {
            return Ok(InstallerMediaDownloadStatus {
                guest_template_id: guest_template_id.to_owned(),
                media_id: media_id.to_owned(),
                state: InstallerMediaDownloadState::Ready,
                downloaded_bytes: file_size(&final_path),
                total_bytes: source.size_bytes,
                local_path: Some(final_path),
                sha256: None,
                detail: String::from("ISO daha once indirildi ve kullanima hazir."),
            });
        }

        let job_key = download_key(guest_template_id, media_id);
        {
            let mut jobs = self.jobs.lock().map_err(|_| {
                InstallerMediaDownloadError::Download(String::from("installer media job lock poisoned"))
            })?;
            if jobs.get(&job_key).is_some_and(|job| {
                matches!(
                    job.state,
                    InstallerMediaDownloadState::Downloading
                        | InstallerMediaDownloadState::Cancelling
                        | InstallerMediaDownloadState::Verifying
                )
            }) {
                return Err(InstallerMediaDownloadError::Busy);
            }
            jobs.insert(
                job_key.clone(),
                DownloadJob {
                    state: InstallerMediaDownloadState::Downloading,
                    part_path: part_path.clone(),
                    final_path: final_path.clone(),
                    total_bytes: source.size_bytes,
                    sha256: None,
                    detail: format!(
                        "Kaynak cozuldu: {}. Resmi mirror deneniyor.",
                        source.origin.code()
                    ),
                    cancel_requested: false,
                },
            );
        }

        let selected_media_id = media_id.to_owned();
        let jobs = Arc::clone(&self.jobs);
        let http = Arc::clone(&self.http);
        thread::spawn(move || {
            let result = run_download(&http, &source, &part_path, &final_path, &jobs, &job_key);
            if let Err(error) = result {
                if let Ok(mut locked) = jobs.lock() {
                    if let Some(job) = locked.get_mut(&job_key) {
                        if job.state != InstallerMediaDownloadState::Cancelled {
                            job.state = InstallerMediaDownloadState::Failed;
                            job.detail = error;
                            job.cancel_requested = false;
                        }
                    }
                }
            }
        });

        self.status(guest_template_id, &selected_media_id)
    }

    pub fn cancel(
        &self,
        guest_template_id: &str,
        media_id: &str,
    ) -> Result<InstallerMediaDownloadStatus, InstallerMediaDownloadError> {
        let _ = self.source_request(guest_template_id, media_id)?;
        let job_key = download_key(guest_template_id, media_id);
        {
            let mut jobs = self.jobs.lock().map_err(|_| {
                InstallerMediaDownloadError::Download(String::from("installer media job lock poisoned"))
            })?;
            if let Some(job) = jobs.get_mut(&job_key) {
                if matches!(
                    job.state,
                    InstallerMediaDownloadState::Downloading | InstallerMediaDownloadState::Verifying
                ) {
                    job.cancel_requested = true;
                    job.state = InstallerMediaDownloadState::Cancelling;
                    job.detail = String::from("ISO indirme durduruluyor.");
                }
            }
        }
        self.status(guest_template_id, media_id)
    }

    pub fn status(
        &self,
        guest_template_id: &str,
        media_id: &str,
    ) -> Result<InstallerMediaDownloadStatus, InstallerMediaDownloadError> {
        let job_key = download_key(guest_template_id, media_id);
        if let Some(job) = self
            .jobs
            .lock()
            .map_err(|_| InstallerMediaDownloadError::Download(String::from("installer media job lock poisoned")))?
            .get(&job_key)
            .cloned()
        {
            let downloaded_bytes = if job.state == InstallerMediaDownloadState::Ready {
                file_size(&job.final_path)
            } else {
                file_size(&job.part_path)
            };
            return Ok(InstallerMediaDownloadStatus {
                guest_template_id: guest_template_id.to_owned(),
                media_id: media_id.to_owned(),
                state: job.state,
                downloaded_bytes,
                total_bytes: job.total_bytes,
                local_path: (job.state == InstallerMediaDownloadState::Ready).then_some(job.final_path),
                sha256: job.sha256,
                detail: job.detail,
            });
        }

        let request = self.source_request(guest_template_id, media_id)?;
        let media_dir = self.download_root.join(guest_template_id).join(media_id);
        if let Some(final_path) = find_ready_iso(&media_dir, request.catalog_source.filename.as_deref()) {
            return Ok(InstallerMediaDownloadStatus {
                guest_template_id: guest_template_id.to_owned(),
                media_id: media_id.to_owned(),
                state: InstallerMediaDownloadState::Ready,
                downloaded_bytes: file_size(&final_path),
                total_bytes: request.catalog_source.size_bytes,
                local_path: Some(final_path),
                sha256: None,
                detail: String::from("Resolver tarafindan indirilen ISO kullanima hazir."),
            });
        }

        Ok(InstallerMediaDownloadStatus {
            guest_template_id: guest_template_id.to_owned(),
            media_id: media_id.to_owned(),
            state: InstallerMediaDownloadState::NotStarted,
            downloaded_bytes: 0,
            total_bytes: request.catalog_source.size_bytes,
            local_path: None,
            sha256: None,
            detail: String::from("ISO henuz indirilmedi."),
        })
    }

    pub fn ready_path(
        &self,
        guest_template_id: &str,
        media_id: &str,
    ) -> Result<PathBuf, InstallerMediaDownloadError> {
        let status = self.status(guest_template_id, media_id)?;
        if status.state != InstallerMediaDownloadState::Ready {
            return Err(InstallerMediaDownloadError::NotReady);
        }
        status.local_path.ok_or(InstallerMediaDownloadError::NotReady)
    }

    fn source_request(
        &self,
        guest_template_id: &str,
        media_id: &str,
    ) -> Result<InstallerMediaSourceRequest, InstallerMediaDownloadError> {
        let template = self.catalog.get(guest_template_id).map_err(|error| {
            InstallerMediaDownloadError::Catalog(format!("{error:?}"))
        })?;
        let media = template
            .installer_media_source(media_id)
            .cloned()
            .ok_or(InstallerMediaDownloadError::UnsupportedMode)?;
        Ok(InstallerMediaSourceRequest {
            guest_template_id: guest_template_id.to_owned(),
            media_id: media_id.to_owned(),
            provider: template.product_id,
            release_id: template.release_id,
            architecture: media.architecture.clone(),
            media_kind: media.media_kind,
            catalog_source: media,
        })
    }

    fn resolve_source(
        &self,
        guest_template_id: &str,
        media_id: &str,
    ) -> Result<ResolvedInstallerMediaSource, InstallerMediaDownloadError> {
        let request = self.source_request(guest_template_id, media_id)?;
        let mut resolver = self.resolver.lock().map_err(|_| {
            InstallerMediaDownloadError::Source(String::from("installer media resolver lock poisoned"))
        })?;
        resolver
            .resolve(&request)
            .map_err(|error| InstallerMediaDownloadError::Source(format!("{error:?}")))
    }
}

fn download_key(guest_template_id: &str, media_id: &str) -> String {
    format!("{guest_template_id}::{media_id}")
}

fn run_download(
    http: &HttpDownloadTool,
    source: &ResolvedInstallerMediaSource,
    part_path: &Path,
    final_path: &Path,
    jobs: &Arc<Mutex<HashMap<String, DownloadJob>>>,
    job_key: &str,
) -> Result<DownloadRunOutcome, String> {
    let parent = final_path
        .parent()
        .ok_or_else(|| String::from("installer media parent path is unavailable"))?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;

    if handle_cancel_if_requested(jobs, job_key, part_path)? {
        return Ok(DownloadRunOutcome::Cancelled);
    }
    set_job_stage(
        jobs,
        job_key,
        InstallerMediaDownloadState::Downloading,
        "Resmi SHA-256 kaynagi cozuluyor.",
    )?;
    let expected_sha256 = fetch_expected_sha256(http, source, jobs, job_key, part_path)?;

    let mut download_errors = Vec::new();
    let mut verified_sha256 = None;
    for (index, url) in source.download_urls.iter().enumerate() {
        if handle_cancel_if_requested(jobs, job_key, part_path)? {
            return Ok(DownloadRunOutcome::Cancelled);
        }
        if index > 0 && part_path.exists() {
            fs::remove_file(part_path).map_err(|error| {
                format!("Mirror failover oncesi partial ISO temizlenemedi: {error}")
            })?;
        }
        set_job_stage(
            jobs,
            job_key,
            InstallerMediaDownloadState::Downloading,
            &format!(
                "Resmi mirror deneniyor {}/{}: {}",
                index + 1,
                source.download_urls.len(),
                host_label(url)
            ),
        )?;
        let mut child = match http.spawn_file_download(url, part_path) {
            Ok(child) => child,
            Err(error) => {
                download_errors.push(format!("{}: {error}", host_label(url)));
                continue;
            }
        };
        match wait_for_download(&mut child, jobs, job_key, part_path) {
            Ok(true) => {}
            Ok(false) => return Ok(DownloadRunOutcome::Cancelled),
            Err(error) => {
                download_errors.push(format!("{}: {error}", host_label(url)));
                continue;
            }
        }

        if handle_cancel_if_requested(jobs, job_key, part_path)? {
            return Ok(DownloadRunOutcome::Cancelled);
        }
        set_job_stage(
            jobs,
            job_key,
            InstallerMediaDownloadState::Verifying,
            &format!("SHA-256 dogrulaniyor: {}", host_label(url)),
        )?;
        let Some(local_sha256) = sha256_file_cancelable(part_path, jobs, job_key)? else {
            mark_cancelled(jobs, job_key, part_path)?;
            return Ok(DownloadRunOutcome::Cancelled);
        };
        if expected_sha256.eq_ignore_ascii_case(&local_sha256) {
            verified_sha256 = Some(local_sha256);
            break;
        }

        download_errors.push(format!(
            "{}: checksum uyusmazligi beklenen={} bulunan={}",
            host_label(url), expected_sha256, local_sha256
        ));
        let _ = fs::remove_file(part_path);
    }

    let local_sha256 = verified_sha256.ok_or_else(|| {
        format!(
            "Tum resmi ISO mirrorlari basarisiz veya SHA-256 dogrulamasi gecmedi: {}",
            download_errors.join(" | ")
        )
    })?;

    if handle_cancel_if_requested(jobs, job_key, part_path)? {
        return Ok(DownloadRunOutcome::Cancelled);
    }
    remove_other_ready_iso_files(parent, final_path)?;
    if final_path.exists() {
        fs::remove_file(final_path).map_err(|error| error.to_string())?;
    }
    fs::rename(part_path, final_path).map_err(|error| error.to_string())?;
    let mut locked = jobs
        .lock()
        .map_err(|_| String::from("installer media job lock poisoned"))?;
    if let Some(job) = locked.get_mut(job_key) {
        job.state = InstallerMediaDownloadState::Ready;
        job.sha256 = Some(local_sha256);
        job.detail = String::from("ISO indirildi ve resmi SHA-256 ile dogrulandi.");
        job.cancel_requested = false;
    }
    Ok(DownloadRunOutcome::Completed)
}

fn fetch_expected_sha256(
    http: &HttpDownloadTool,
    source: &ResolvedInstallerMediaSource,
    jobs: &Arc<Mutex<HashMap<String, DownloadJob>>>,
    job_key: &str,
    part_path: &Path,
) -> Result<String, String> {
    if source.checksum_urls.is_empty() {
        return Err(String::from("Resmi SHA-256 kaynagi bulunamadi"));
    }
    let mut errors = Vec::new();
    for url in &source.checksum_urls {
        if handle_cancel_if_requested(jobs, job_key, part_path)? {
            return Err(String::from("ISO indirme durduruldu"));
        }
        match http.fetch_text(url) {
            Ok(content) => match expected_sha256(&content, &source.filename) {
                Some(hash) => return Ok(hash),
                None => errors.push(format!(
                    "{}: {} checksum listesinde yok",
                    host_label(url),
                    source.filename
                )),
            },
            Err(error) => errors.push(format!("{}: {error}", host_label(url))),
        }
    }
    Err(format!(
        "Tum resmi SHA-256 kaynaklari basarisiz: {}",
        errors.join(" | ")
    ))
}

fn wait_for_download(
    child: &mut Child,
    jobs: &Arc<Mutex<HashMap<String, DownloadJob>>>,
    job_key: &str,
    part_path: &Path,
) -> Result<bool, String> {
    loop {
        if is_cancel_requested(jobs, job_key)? {
            let _ = child.kill();
            let _ = child.wait();
            mark_cancelled(jobs, job_key, part_path)?;
            return Ok(false);
        }
        match child
            .try_wait()
            .map_err(|error| format!("curl durumu okunamadi: {error}"))?
        {
            Some(status) if status.success() => return Ok(true),
            Some(status) => {
                return Err(format!(
                    "ISO indirme basarisiz: curl exit={}",
                    status.code().unwrap_or(-1)
                ))
            }
            None => thread::sleep(Duration::from_millis(DOWNLOAD_POLL_INTERVAL_MS)),
        }
    }
}

fn handle_cancel_if_requested(
    jobs: &Arc<Mutex<HashMap<String, DownloadJob>>>,
    job_key: &str,
    part_path: &Path,
) -> Result<bool, String> {
    if !is_cancel_requested(jobs, job_key)? {
        return Ok(false);
    }
    mark_cancelled(jobs, job_key, part_path)?;
    Ok(true)
}

fn mark_cancelled(
    jobs: &Arc<Mutex<HashMap<String, DownloadJob>>>,
    job_key: &str,
    part_path: &Path,
) -> Result<(), String> {
    let _ = fs::remove_file(part_path);
    let mut locked = jobs
        .lock()
        .map_err(|_| String::from("installer media job lock poisoned"))?;
    if let Some(job) = locked.get_mut(job_key) {
        job.state = InstallerMediaDownloadState::Cancelled;
        job.detail = String::from("ISO indirme durduruldu. Yarim dosya temizlendi.");
        job.cancel_requested = false;
        job.sha256 = None;
    }
    Ok(())
}

fn is_cancel_requested(
    jobs: &Arc<Mutex<HashMap<String, DownloadJob>>>,
    job_key: &str,
) -> Result<bool, String> {
    let locked = jobs
        .lock()
        .map_err(|_| String::from("installer media job lock poisoned"))?;
    Ok(locked.get(job_key).is_some_and(|job| job.cancel_requested))
}

fn set_job_stage(
    jobs: &Arc<Mutex<HashMap<String, DownloadJob>>>,
    job_key: &str,
    state: InstallerMediaDownloadState,
    detail: &str,
) -> Result<(), String> {
    let mut locked = jobs
        .lock()
        .map_err(|_| String::from("installer media job lock poisoned"))?;
    if let Some(job) = locked.get_mut(job_key) {
        job.state = state;
        job.detail = detail.to_owned();
    }
    Ok(())
}

fn find_ready_iso(directory: &Path, catalog_filename: Option<&str>) -> Option<PathBuf> {
    if let Some(filename) = catalog_filename {
        let catalog_path = directory.join(filename);
        if catalog_path.is_file() {
            return Some(catalog_path);
        }
    }

    let mut candidates = fs::read_dir(directory)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case("iso"))
        })
        .collect::<Vec<_>>();
    candidates.sort();
    (candidates.len() == 1).then(|| candidates.remove(0))
}

fn remove_other_ready_iso_files(directory: &Path, keep_path: &Path) -> Result<(), String> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.to_string()),
    };
    for entry in entries {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path == keep_path || !path.is_file() {
            continue;
        }
        let is_iso = path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.eq_ignore_ascii_case("iso"));
        if is_iso {
            fs::remove_file(path).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn file_size(path: &Path) -> u64 {
    fs::metadata(path).map(|metadata| metadata.len()).unwrap_or(0)
}

fn sha256_file_cancelable(
    path: &Path,
    jobs: &Arc<Mutex<HashMap<String, DownloadJob>>>,
    job_key: &str,
) -> Result<Option<String>, String> {
    let mut file = fs::File::open(path).map_err(|error| error.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        if is_cancel_requested(jobs, job_key)? {
            return Ok(None);
        }
        let read = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(Some(format!("{:x}", hasher.finalize())))
}

fn expected_sha256(content: &str, filename: &str) -> Option<String> {
    content.lines().find_map(|line| {
        let line = line.trim();
        let mut parts = line.split_whitespace();
        if let (Some(hash), Some(name)) = (parts.next(), parts.next()) {
            let name = name.trim_start_matches('*');
            if name == filename && valid_sha256(hash) {
                return Some(hash.to_ascii_lowercase());
            }
        }

        let prefix = format!("SHA256 ({filename}) = ");
        line.strip_prefix(&prefix)
            .map(str::trim)
            .filter(|hash| valid_sha256(hash))
            .map(|hash| hash.to_ascii_lowercase())
    })
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn host_label(url: &str) -> &str {
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    without_scheme.split('/').next().unwrap_or(without_scheme)
}
