// # 📄 Dosya Yolu: /turkuazvm/apps/engine/src/config/engine_config.rs
// # 📌 Amac: Merkezi turkuazvm.yml dosyasini Engine runtime ve remote access ayarlarina donusturur
// # 📌 Modul - Rust
// # Version: 0.41.6
// # Aciklama: Schema 22; metadata data_root ile buyuk disk image_root kokunu ayirir, eksik image_root icin legacy data_root fallbackini korur
// # Bagimli Oldugu Katman: Service | Tool

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::net::{IpAddr, Ipv4Addr};
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Deserialize;
use turkuazvm_android::domain::runtime_profile::AndroidDisplayProfile;
use turkuazvm_core::domain::disk::{DiskBus, DiskFormat};
use turkuazvm_core::domain::network::{NetworkDeviceModel, NetworkMode};
use turkuazvm_core::domain::storage_host::PortableImagePolicy;
use turkuazvm_gpu::domain::profile::{GpuBackend, GpuBackendPreference, GamingGpuPolicy};
use turkuazvm_guest_catalog::domain::installer_media_source::{
    InstallerMediaChecksumStrategy, InstallerMediaDiscoveryMode, InstallerMediaProviderPolicy,
    InstallerMediaProviderRule, InstallerMediaResolverPolicy,
};
use turkuazvm_platform::tools::native_network_tool::NativeNetworkSettings;
use turkuazvm_qemu::domain::qemu_runtime::{
    QemuDisplayMode, QemuGpuRuntimeSettings, QemuRuntimeSettings,
};
use turkuazvm_transport::tools::tls_stream_tool::TlsServerSettings;

const CONFIG_ENV: &str = "TURKUAZVM_CONFIG";
const ENGINE_TOKEN_ENV: &str = "TURKUAZVM_ENGINE_TOKEN";
const DEFAULT_CONFIG_PATH: &str = "config/turkuazvm.yml";
const MIN_REMOTE_TOKEN_LENGTH: usize = 32;
const DEFAULT_GPU_MODE: &str = "auto";
const DEFAULT_GPU_HOSTMEM_MIB: u64 = 1024;
const EXPECTED_CONFIG_SCHEMA_VERSION: u16 = 22;
const BYTES_PER_GIB: u64 = 1024 * 1024 * 1024;
const LEGACY_MACHINE_DIRECTORY: &str = "machines";

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub host_id: String,
    pub host_label: String,
    pub api_bind_ip: IpAddr,
    pub api_port: u16,
    pub api_timeout: Duration,
    pub api_auth_token: Option<String>,
    pub api_tls: Option<TlsServerSettings>,
    pub data_root: PathBuf,
    pub image_root: PathBuf,
    pub storage_policy: StorageEnginePolicy,
    pub download_http: DownloadHttpEngineConfig,
    pub artifact_cache: ArtifactCacheEngineConfig,
    pub qemu_runtime: QemuRuntimeSettings,
    pub runtime_maintenance_interval: Duration,
    pub runtime_recovery: RuntimeRecoveryEngineConfig,
    pub network: NativeNetworkSettings,
    pub network_policy: NetworkEnginePolicy,
    pub guest: GuestEngineConfig,
    pub gpu_policy: GamingGpuPolicy,
    pub android: AndroidEngineConfig,
    pub android_image: AndroidImageEngineConfig,
    pub game_catalog_path: PathBuf,
    pub guest_catalog_path: PathBuf,
}



#[derive(Debug, Clone)]
pub struct DownloadHttpEngineConfig {
    pub curl_binary: PathBuf,
    pub connect_timeout_seconds: u64,
    pub retry_count: u32,
    pub retry_delay_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct StorageEnginePolicy {
    pub default_disk_format: DiskFormat,
    pub default_disk_bus: DiskBus,
    pub default_disk_size_gib: u64,
    pub portable_image: PortableImagePolicy,
}

#[derive(Debug, Clone)]
pub struct NetworkEnginePolicy {
    pub default_network_id: String,
    pub private_network_id: String,
    pub default_device_model: NetworkDeviceModel,
    pub default_profile: NetworkMode,
    pub managed_nat: ManagedNetworkPoolPolicy,
    pub private_network: ManagedNetworkPoolPolicy,
}

#[derive(Debug, Clone)]
pub struct ManagedNetworkPoolPolicy {
    pub subnet: Ipv4Addr,
    pub prefix_length: u8,
    pub gateway: Ipv4Addr,
    pub pool_start: u8,
    pub pool_end: u8,
    pub dns_servers: Vec<Ipv4Addr>,
}


#[derive(Debug, Clone)]
pub struct GuestEngineConfig {
    pub installer_media_relative_path: String,
    pub installer_media_download_root: PathBuf,
    pub installer_media_source_cache_path: PathBuf,
    pub installer_media_resolver_policy: InstallerMediaResolverPolicy,
    pub uefi_enabled: bool,
    pub uefi_code_source_path: PathBuf,
    pub uefi_vars_template_source_path: PathBuf,
    pub uefi_code_relative_path: String,
    pub uefi_vars_relative_path: String,
}

#[derive(Debug, Clone, Copy)]
pub struct RuntimeRecoveryEngineConfig {
    pub auto_restart: bool,
    pub retry_delay: Duration,
    pub max_attempts: u32,
    pub journal_retention: usize,
}

#[derive(Debug, Clone)]
pub struct ArtifactCacheEngineConfig {
    pub enabled: bool,
    pub root: PathBuf,
    pub quota_bytes: u64,
    pub verify_on_hit: bool,
    pub mutable: ArtifactCacheMutableEngineConfig,
    pub downloader: ArtifactDownloaderEngineConfig,
}

#[derive(Debug, Clone, Copy)]
pub struct ArtifactCacheMutableEngineConfig {
    pub allow_stale_on_transient_error: bool,
    pub revalidation_timeout: Duration,
    pub require_https: bool,
    pub allow_private_networks: bool,
    pub max_redirects: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct ArtifactDownloaderEngineConfig {
    pub retry_count: u32,
    pub retry_delay_seconds: u64,
    pub connect_timeout_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct AndroidEngineConfig {
    pub adb_binary: Option<PathBuf>,
    pub adb_host_ip: Ipv4Addr,
    pub adb_host_port_min: u16,
    pub adb_host_port_max: u16,
    pub adb_guest_port: u16,
    pub command_timeout: Duration,
    pub ready_timeout: Duration,
    pub ready_poll_interval: Duration,
    pub package_root: PathBuf,
    pub default_width: u32,
    pub default_height: u32,
    pub default_density_dpi: u32,
    pub default_target_fps: u16,
    pub guest_agent: AndroidGuestAgentEngineConfig,
}


#[derive(Debug, Clone)]
pub struct AndroidImageEngineConfig {
    pub source_root: PathBuf,
    pub output_root: PathBuf,
    pub source_cache_path: PathBuf,
    pub build_script: PathBuf,
    pub sdk: AndroidSdkEngineConfig,
    pub distribution_base_url: String,
    pub distribution_official_base_url: String,
    pub distribution_official_fallback: bool,
    pub distribution_branch_templates: Vec<String>,
    pub distribution_target_candidates: Vec<String>,
    pub distribution_channels: BTreeMap<String, AndroidDistributionChannelEngineConfig>,
    pub distribution_providers: BTreeMap<String, AndroidDistributionProviderEngineConfig>,
    pub distribution_branch: String,
    pub distribution_target: String,
    pub distribution_minimum_free_disk_gib: u64,
}

#[derive(Debug, Clone)]
pub struct AndroidSdkEngineConfig {
    pub tool_root: PathBuf,
    pub repository_base_url: String,
    pub package_index_url: String,
    pub emulator_package_path: String,
    pub architecture: String,
    pub emulator_console_port_min: u16,
    pub emulator_console_port_max: u16,
    pub variant_priority: Vec<String>,
    pub system_image_indexes: BTreeMap<String, String>,
    pub api_levels: BTreeMap<String, u32>,
}

#[derive(Debug, Clone)]
pub struct AndroidDistributionChannelEngineConfig {
    pub expected_sdk: u32,
    pub allow_device_bootloader_fallback: bool,
    pub branch_hints: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndroidDistributionProviderEngineConfig {
    AndroidSdk,
    AndroidCi,
    SourceBuild,
}

#[derive(Debug, Clone)]
pub struct AndroidGuestAgentEngineConfig {
    pub host_ip: Ipv4Addr,
    pub host_port_min: u16,
    pub host_port_max: u16,
    pub guest_port: u16,
    pub connect_timeout: Duration,
    pub io_timeout: Duration,
    pub ready_timeout: Duration,
    pub ready_poll_interval: Duration,
    pub secret_root: PathBuf,
    pub require_adb_root_for_provisioning: bool,
    pub update: AndroidGuestAgentUpdateEngineConfig,
}

#[derive(Debug, Clone)]
pub struct AndroidGuestAgentUpdateEngineConfig {
    pub enabled: bool,
    pub manifest_path: PathBuf,
    pub trusted_public_key_hex: String,
    pub trusted_apk_cert_sha256: String,
    pub apksigner_binary: Option<PathBuf>,
    pub rollback_root: PathBuf,
}

#[derive(Debug)]
pub enum EngineConfigError {
    Read(String),
    Parse(String),
    Invalid(String),
}

impl fmt::Display for EngineConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(message) => write!(formatter, "config read failed: {message}"),
            Self::Parse(message) => write!(formatter, "config parse failed: {message}"),
            Self::Invalid(message) => write!(formatter, "config invalid: {message}"),
        }
    }
}

impl std::error::Error for EngineConfigError {}

#[derive(Debug, Deserialize)]
struct RootConfig {
    schema_version: u16,
    host: HostConfig,
    storage: StorageConfig,
    downloads: DownloadsConfig,
    artifact_cache: ArtifactCacheConfig,
    qmp: QmpConfig,
    runtime: RuntimeConfig,
    network: NetworkConfig,
    guest: GuestConfig,
    engine_api: EngineApiConfig,
    #[serde(default)]
    gpu: GpuConfig,
    android: AndroidConfig,
    game_catalog: GameCatalogConfig,
    guest_catalog: GuestCatalogConfig,
}

#[derive(Debug, Deserialize)]
struct DownloadsConfig {
    sources_path: String,
    http: DownloadHttpConfig,
}

#[derive(Debug, Deserialize)]
struct DownloadHttpConfig {
    curl_binary: String,
    connect_timeout_seconds: u64,
    retry_count: u32,
    retry_delay_seconds: u64,
}

#[derive(Debug, Deserialize)]
struct DownloadSourcesFile {
    schema_version: u16,
    paths: DownloadPathsConfig,
    sources: DownloadSourceProvidersConfig,
}

#[derive(Debug, Deserialize)]
struct DownloadPathsConfig {
    installer_media: String,
    installer_media_source_cache: String,
    android_images: String,
    artifact_cache: String,
    android_source_cache: String,
    android_sdk_tools: String,
}

#[derive(Debug, Deserialize)]
struct DownloadSourceProvidersConfig {
    linux_media: LinuxMediaSourceConfig,
    android_release_policy: BTreeMap<String, AndroidReleaseProviderConfig>,
    android_sdk: AndroidSdkSourceConfig,
    android_ci: AndroidCiSourceConfig,
}

#[derive(Debug, Deserialize)]
struct LinuxMediaSourceConfig {
    #[serde(default = "default_true")]
    use_catalog_fallback: bool,
    providers: BTreeMap<String, LinuxMediaProviderConfig>,
}

#[derive(Debug, Deserialize)]
struct LinuxMediaProviderConfig {
    #[serde(default)]
    discovery_mode: LinuxMediaDiscoveryModeConfig,
    base_urls: Vec<String>,
    rules: BTreeMap<String, LinuxMediaRuleConfig>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LinuxMediaDiscoveryModeConfig {
    #[default]
    DirectoryIndex,
    OfficialPageMirrors,
}

#[derive(Debug, Deserialize)]
struct LinuxMediaRuleConfig {
    index_path_templates: Vec<String>,
    filename_tokens: Vec<String>,
    #[serde(default)]
    filename_excludes: Vec<String>,
    checksum_strategy: LinuxChecksumStrategyConfig,
    #[serde(default)]
    checksum_value: String,
    #[serde(default)]
    checksum_tokens: Vec<String>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LinuxChecksumStrategyConfig {
    FixedName,
    FileSuffix,
    Discover,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum AndroidReleaseProviderConfig {
    AndroidSdk,
    AndroidCi,
    SourceBuild,
}

#[derive(Debug, Deserialize)]
struct AndroidSdkSourceConfig {
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

#[derive(Debug, Deserialize)]
struct AndroidCiSourceConfig {
    base_url: String,
    official_base_url: String,
    #[serde(default = "default_true")]
    use_official_fallback: bool,
    default_branch: String,
    default_target: String,
    branch_templates: Vec<String>,
    target_candidates: Vec<String>,
    channels: BTreeMap<String, AndroidCiChannelConfig>,
}

#[derive(Debug, Deserialize)]
struct AndroidCiChannelConfig {
    expected_sdk: u32,
    #[serde(default)]
    allow_device_bootloader_fallback: bool,
    #[serde(default)]
    branch_hints: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct GuestConfig {
    installer_media_relative_path: String,
    firmware: GuestFirmwareConfig,
}

#[derive(Debug, Deserialize)]
struct GuestFirmwareConfig {
    uefi: GuestUefiConfig,
}

#[derive(Debug, Deserialize)]
struct GuestUefiConfig {
    enabled: bool,
    code_source_path: String,
    vars_template_source_path: String,
    code_relative_path: String,
    vars_relative_path: String,
}

#[derive(Debug, Deserialize)]
struct HostConfig {
    id: String,
    label: String,
}

#[derive(Debug, Deserialize)]
struct StorageConfig {
    data_root: PathBuf,
    #[serde(default)]
    image_root: Option<PathBuf>,
    default_disk: DefaultDiskConfig,
    portable_image: PortableImageConfig,
}

#[derive(Debug, Deserialize)]
struct DefaultDiskConfig {
    format: String,
    bus: String,
    size_gib: u64,
}

#[derive(Debug, Deserialize)]
struct PortableImageConfig {
    extension: String,
    container: String,
    private_copy_default: bool,
}

#[derive(Debug, Deserialize)]
struct ArtifactCacheConfig {
    enabled: bool,
    quota_gib: u64,
    verify_on_hit: bool,
    mutable: ArtifactCacheMutableConfig,
    downloader: ArtifactDownloaderConfig,
}

#[derive(Debug, Deserialize)]
struct ArtifactCacheMutableConfig {
    allow_stale_on_transient_error: bool,
    revalidation_timeout_seconds: u64,
    require_https: bool,
    allow_private_networks: bool,
    max_redirects: u32,
}

#[derive(Debug, Deserialize)]
struct ArtifactDownloaderConfig {
    retry_count: u32,
    retry_delay_seconds: u64,
    connect_timeout_seconds: u64,
}

#[derive(Debug, Deserialize)]
struct QmpConfig {
    bind_ip: String,
    connect_timeout_ms: u64,
    read_timeout_ms: u64,
    write_timeout_ms: u64,
    startup_timeout_ms: u64,
    startup_poll_interval_ms: u64,
    shutdown_timeout_ms: u64,
    shutdown_poll_interval_ms: u64,
}

#[derive(Debug, Deserialize)]
struct RuntimeConfig {
    display_mode: String,
    rfb_bind_ip: String,
    rfb_display_min: u16,
    rfb_display_max: u16,
    maintenance_interval_ms: u64,
    recovery: RuntimeRecoveryConfig,
}

#[derive(Debug, Deserialize)]
struct RuntimeRecoveryConfig {
    auto_restart: bool,
    retry_delay_ms: u64,
    max_attempts: u32,
    journal_retention: usize,
}

#[derive(Debug, Deserialize)]
struct NetworkConfig {
    default_network_id: String,
    private_network_id: String,
    default_device_model: String,
    default_profile: String,
    managed_nat: ManagedNetworkConfig,
    private: ManagedNetworkConfig,
    linux: LinuxNetworkConfig,
    windows: WindowsNetworkConfig,
}

#[derive(Debug, Deserialize)]
struct ManagedNetworkConfig {
    subnet: String,
    prefix_length: u8,
    gateway: String,
    pool_start: u8,
    pool_end: u8,
    #[serde(default)]
    dns_servers: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct LinuxNetworkConfig {
    bridge_helper_path: String,
    allow_managed_tap: bool,
}

#[derive(Debug, Deserialize)]
struct WindowsNetworkConfig {
    managed_helper_path: String,
    openvpn_package_id: String,
}

#[derive(Debug, Deserialize)]
struct GpuConfig {
    #[serde(default = "default_gpu_mode")]
    mode: String,
    #[serde(default = "default_gpu_hostmem_mib")]
    hostmem_mib: u64,
    #[serde(default)]
    allow_experimental_android_gfxstream: bool,
}

impl Default for GpuConfig {
    fn default() -> Self {
        Self {
            mode: default_gpu_mode(),
            hostmem_mib: default_gpu_hostmem_mib(),
            allow_experimental_android_gfxstream: false,
        }
    }
}

#[derive(Debug, Deserialize)]
struct AndroidConfig {
    package_root: String,
    image: AndroidImageConfig,
    adb: AndroidAdbConfig,
    guest_agent: AndroidGuestAgentConfig,
    default_device: AndroidDefaultDeviceConfig,
}


#[derive(Debug, Deserialize)]
struct AndroidImageConfig {
    source_root: String,
    build_script: String,
    distribution: AndroidImageDistributionConfig,
}

#[derive(Debug, Deserialize)]
struct AndroidImageDistributionConfig {
    minimum_free_disk_gib: u64,
}

#[derive(Debug, Deserialize)]
struct AndroidGuestAgentConfig {
    host_ip: String,
    host_port_min: u16,
    host_port_max: u16,
    guest_port: u16,
    connect_timeout_ms: u64,
    io_timeout_ms: u64,
    ready_timeout_ms: u64,
    ready_poll_interval_ms: u64,
    secret_root: String,
    require_adb_root_for_provisioning: bool,
    update: AndroidGuestAgentUpdateConfig,
}

#[derive(Debug, Deserialize)]
struct AndroidGuestAgentUpdateConfig {
    enabled: bool,
    manifest_path: String,
    trusted_public_key_hex: String,
    trusted_apk_cert_sha256: String,
    apksigner_path: String,
    rollback_root: String,
}

#[derive(Debug, Deserialize)]
struct GameCatalogConfig {
    path: String,
}

#[derive(Debug, Deserialize)]
struct GuestCatalogConfig {
    path: String,
}

#[derive(Debug, Deserialize)]
struct AndroidAdbConfig {
    binary_path: String,
    host_ip: String,
    host_port_min: u16,
    host_port_max: u16,
    guest_port: u16,
    command_timeout_ms: u64,
    ready_timeout_ms: u64,
    ready_poll_interval_ms: u64,
}

#[derive(Debug, Deserialize)]
struct AndroidDefaultDeviceConfig {
    width: u32,
    height: u32,
    density_dpi: u32,
    target_fps: u16,
}

#[derive(Debug, Deserialize)]
struct EngineApiConfig {
    bind_ip: String,
    port: u16,
    request_timeout_ms: u64,
    authentication: EngineAuthenticationConfig,
    tls: EngineTlsConfig,
}

#[derive(Debug, Deserialize)]
struct EngineAuthenticationConfig {
    mode: String,
    token_file: String,
}

#[derive(Debug, Deserialize)]
struct EngineTlsConfig {
    enabled: bool,
    certificate_path: String,
    private_key_path: String,
}

impl EngineConfig {
    pub fn load() -> Result<Self, EngineConfigError> {
        let path = locate_config().ok_or_else(|| {
            EngineConfigError::Read(String::from("TurkuazVM config file not found"))
        })?;
        Self::load_from(&path)
    }

    pub fn load_from(path: &Path) -> Result<Self, EngineConfigError> {
        let content = fs::read_to_string(path)
            .map_err(|error| EngineConfigError::Read(error.to_string()))?;
        let config: RootConfig = serde_yaml_ng::from_str(&content)
            .map_err(|error| EngineConfigError::Parse(error.to_string()))?;
        if config.schema_version != EXPECTED_CONFIG_SCHEMA_VERSION {
            return Err(EngineConfigError::Invalid(format!(
                "unsupported config schema {}; expected {}",
                config.schema_version, EXPECTED_CONFIG_SCHEMA_VERSION
            )));
        }
        let project_root = project_root(path)?;
        let download_sources_path = resolve_path(
            &project_root,
            required_path(&config.downloads.sources_path, "downloads.sources_path")?,
        );
        let download_sources = load_download_sources(&download_sources_path)?;
        let download_http_curl = required_text(&config.downloads.http.curl_binary, "downloads.http.curl_binary")?;
        if config.downloads.http.connect_timeout_seconds == 0
            || config.downloads.http.retry_count == 0
            || config.downloads.http.retry_delay_seconds == 0
        {
            return Err(EngineConfigError::Invalid(String::from(
                "downloads.http retry/timeout values must be greater than zero",
            )));
        }
        let download_http = DownloadHttpEngineConfig {
            curl_binary: PathBuf::from(download_http_curl),
            connect_timeout_seconds: config.downloads.http.connect_timeout_seconds,
            retry_count: config.downloads.http.retry_count,
            retry_delay_seconds: config.downloads.http.retry_delay_seconds,
        };

        validate_identity(&config.host)?;
        let qmp_bind_ip = parse_ip(&config.qmp.bind_ip, "qmp.bind_ip")?;
        if !qmp_bind_ip.is_loopback() {
            return Err(EngineConfigError::Invalid(String::from(
                "QMP must bind to a loopback address",
            )));
        }
        let api_bind_ip = parse_ip(&config.engine_api.bind_ip, "engine_api.bind_ip")?;
        let api_auth_token = load_auth_token(&config.engine_api.authentication, &project_root)?;
        let api_tls = load_tls_settings(&config.engine_api.tls, &project_root)?;
        validate_remote_security(api_bind_ip, api_auth_token.as_deref(), api_tls.as_ref())?;

        let display_mode = match config.runtime.display_mode.as_str() {
            "native_rfb" => QemuDisplayMode::NativeRfb,
            "default" => QemuDisplayMode::Default,
            "none" => QemuDisplayMode::None,
            other => {
                return Err(EngineConfigError::Invalid(format!(
                    "Unsupported runtime.display_mode: {other}"
                )))
            }
        };
        let rfb_bind_ip = parse_ip(&config.runtime.rfb_bind_ip, "runtime.rfb_bind_ip")?;
        if rfb_bind_ip != IpAddr::V4(Ipv4Addr::LOCALHOST) {
            return Err(EngineConfigError::Invalid(String::from(
                "runtime.rfb_bind_ip must be 127.0.0.1",
            )));
        }
        if config.runtime.rfb_display_min > config.runtime.rfb_display_max {
            return Err(EngineConfigError::Invalid(String::from(
                "runtime.rfb_display_min must not exceed runtime.rfb_display_max",
            )));
        }

        let data_root = resolve_path(&project_root, config.storage.data_root);
        let image_root = config
            .storage
            .image_root
            .clone()
            .filter(|path| !path.as_os_str().is_empty())
            .map(|path| resolve_path(&project_root, path))
            .unwrap_or_else(|| data_root.join(LEGACY_MACHINE_DIRECTORY));
        let artifact_cache_root = resolve_path(
            &project_root,
            required_path(&download_sources.paths.artifact_cache, "download-sources.paths.artifact_cache")?,
        );
        if config.artifact_cache.quota_gib == 0
            || config.artifact_cache.mutable.revalidation_timeout_seconds == 0
            || config.artifact_cache.mutable.max_redirects == 0
            || config.artifact_cache.downloader.retry_count == 0
            || config.artifact_cache.downloader.retry_delay_seconds == 0
            || config.artifact_cache.downloader.connect_timeout_seconds == 0
        {
            return Err(EngineConfigError::Invalid(String::from(
                "artifact_cache quota/downloader values must be greater than zero",
            )));
        }
        let artifact_cache_quota_bytes = config
            .artifact_cache
            .quota_gib
            .checked_mul(BYTES_PER_GIB)
            .ok_or_else(|| EngineConfigError::Invalid(String::from("artifact_cache.quota_gib overflow")))?;
        let default_disk_format = match config.storage.default_disk.format.as_str() {
            "qcow2" => DiskFormat::Qcow2,
            "raw" => DiskFormat::Raw,
            other => return Err(EngineConfigError::Invalid(format!("Unsupported storage.default_disk.format: {other}"))),
        };
        let default_disk_bus = match config.storage.default_disk.bus.as_str() {
            "virtio" => DiskBus::Virtio,
            "ide" => DiskBus::Ide,
            other => return Err(EngineConfigError::Invalid(format!("Unsupported storage.default_disk.bus: {other}"))),
        };
        if config.storage.default_disk.size_gib == 0 {
            return Err(EngineConfigError::Invalid(String::from("storage.default_disk.size_gib must be greater than zero")));
        }
        let portable_extension = required_text(&config.storage.portable_image.extension, "storage.portable_image.extension")?;
        if !portable_extension.starts_with('.') || portable_extension.contains('/') || portable_extension.contains('\\') {
            return Err(EngineConfigError::Invalid(String::from("storage.portable_image.extension must be a simple dotted extension")));
        }
        let portable_container = required_text(&config.storage.portable_image.container, "storage.portable_image.container")?;
        if portable_container != "zip64" {
            return Err(EngineConfigError::Invalid(String::from("storage.portable_image.container must be zip64 in schema 22")));
        }
        let default_network_id = required_text(&config.network.default_network_id, "network.default_network_id")?;
        validate_distribution_token(&default_network_id, "network.default_network_id")?;
        let private_network_id = required_text(&config.network.private_network_id, "network.private_network_id")?;
        validate_distribution_token(&private_network_id, "network.private_network_id")?;
        let default_network_device_model = match config.network.default_device_model.as_str() {
            "virtio_net_pci" => NetworkDeviceModel::VirtioNetPci,
            "e1000" => NetworkDeviceModel::E1000,
            other => return Err(EngineConfigError::Invalid(format!("Unsupported network.default_device_model: {other}"))),
        };
        let default_network_profile = match config.network.default_profile.as_str() {
            "managed_nat" => NetworkMode::ManagedNat,
            "private" => NetworkMode::Private,
            "user_nat" => NetworkMode::UserNat,
            other => return Err(EngineConfigError::Invalid(format!("Unsupported network.default_profile: {other}"))),
        };
        let managed_nat_policy = parse_managed_network_pool(&config.network.managed_nat, "network.managed_nat")?;
        let private_network_policy = parse_managed_network_pool(&config.network.private, "network.private")?;
        let bridge_helper = non_empty_path(config.network.linux.bridge_helper_path)
            .map(|value| resolve_path(&project_root, value));
        let windows_managed_helper = non_empty_path(config.network.windows.managed_helper_path.clone())
            .map(|value| resolve_path(&project_root, value));
        let _advanced_openvpn_package_id = config.network.windows.openvpn_package_id.trim();
        let adb_host_ip = config
            .android
            .adb
            .host_ip
            .parse::<Ipv4Addr>()
            .map_err(|_| EngineConfigError::Invalid(String::from("android.adb.host_ip must be IPv4")))?;
        if !adb_host_ip.is_loopback() {
            return Err(EngineConfigError::Invalid(String::from(
                "android.adb.host_ip must be loopback",
            )));
        }
        if config.android.adb.host_port_min == 0
            || config.android.adb.host_port_min > config.android.adb.host_port_max
            || config.android.adb.guest_port == 0
        {
            return Err(EngineConfigError::Invalid(String::from(
                "android.adb port range is invalid",
            )));
        }
        if config.android.adb.command_timeout_ms == 0
            || config.android.adb.ready_timeout_ms == 0
            || config.android.adb.ready_poll_interval_ms == 0
        {
            return Err(EngineConfigError::Invalid(String::from(
                "android.adb timeout values must be greater than zero",
            )));
        }
        AndroidDisplayProfile::create(
            config.android.default_device.width,
            config.android.default_device.height,
            config.android.default_device.density_dpi,
            config.android.default_device.target_fps,
        )
        .map_err(|error| {
            EngineConfigError::Invalid(format!("android.default_device is invalid: {error:?}"))
        })?;
        let adb_binary = non_empty_path(config.android.adb.binary_path)
            .map(|value| resolve_path(&project_root, value));
        let package_root = resolve_path(
            &project_root,
            required_path(&config.android.package_root, "android.package_root")?,
        );
        let android_image_source_root = non_empty_path(config.android.image.source_root.clone())
            .map(|value| resolve_path(&project_root, value))
            .unwrap_or_default();
        let android_image_output_root = resolve_path(
            &project_root,
            required_path(&download_sources.paths.android_images, "download-sources.paths.android_images")?,
        );
        let android_distribution_source_cache_path = resolve_path(
            &project_root,
            required_path(&download_sources.paths.android_source_cache, "download-sources.paths.android_source_cache")?,
        );
        let android_image_build_script = resolve_path(
            &project_root,
            required_path(&config.android.image.build_script, "android.image.build_script")?,
        );
        let android_sdk_tool_root = resolve_path(
            &project_root,
            required_path(&download_sources.paths.android_sdk_tools, "download-sources.paths.android_sdk_tools")?,
        );
        let android_sdk_repository_base_url = validate_base_url(
            &download_sources.sources.android_sdk.repository_base_url,
            "download-sources.sources.android_sdk.repository_base_url",
        )?;
        let android_sdk_package_index_url = validate_base_url(
            &download_sources.sources.android_sdk.package_index_url,
            "download-sources.sources.android_sdk.package_index_url",
        )?;
        let android_sdk_emulator_package_path = required_text(
            &download_sources.sources.android_sdk.emulator_package_path,
            "download-sources.sources.android_sdk.emulator_package_path",
        )?;
        let android_sdk_architecture = required_text(
            &download_sources.sources.android_sdk.architecture,
            "download-sources.sources.android_sdk.architecture",
        )?;
        if android_sdk_architecture != "x86_64" {
            return Err(EngineConfigError::Invalid(String::from("Android SDK architecture must be x86_64")));
        }
        let android_sdk_console_min = download_sources.sources.android_sdk.emulator_console_port_min;
        let android_sdk_console_max = download_sources.sources.android_sdk.emulator_console_port_max;
        if android_sdk_console_min < 5554 || android_sdk_console_max > 5680 || android_sdk_console_min > android_sdk_console_max || android_sdk_console_min % 2 != 0 || android_sdk_console_max % 2 != 0 {
            return Err(EngineConfigError::Invalid(String::from("Android SDK emulator console ports must be an even range within 5554..5680")));
        }
        let android_sdk_variant_priority = parse_non_empty_text_list(
            &download_sources.sources.android_sdk.variant_priority,
            "download-sources.sources.android_sdk.variant_priority",
        )?;
        if download_sources.sources.android_sdk.system_image_indexes.is_empty() {
            return Err(EngineConfigError::Invalid(String::from("Android SDK system image indexes must not be empty")));
        }
        let mut android_sdk_system_image_indexes = BTreeMap::new();
        for (variant, url) in &download_sources.sources.android_sdk.system_image_indexes {
            let variant = required_text(variant, "Android SDK system image variant")?;
            let url = validate_base_url(url, "Android SDK system image index URL")?;
            android_sdk_system_image_indexes.insert(variant, url);
        }
        for variant in &android_sdk_variant_priority {
            if !android_sdk_system_image_indexes.contains_key(variant) {
                return Err(EngineConfigError::Invalid(format!("Android SDK variant {variant} has no package index")));
            }
        }
        if download_sources.sources.android_sdk.api_levels.is_empty() || download_sources.sources.android_sdk.api_levels.values().any(|value| *value == 0) {
            return Err(EngineConfigError::Invalid(String::from("Android SDK API level map must contain non-zero values")));
        }
        let android_sdk_api_levels = download_sources.sources.android_sdk.api_levels.clone();

        let android_distribution_base_url = validate_base_url(
            &download_sources.sources.android_ci.base_url,
            "download-sources.sources.android_ci.base_url",
        )?;
        let android_distribution_official_base_url = validate_base_url(
            &download_sources.sources.android_ci.official_base_url,
            "download-sources.sources.android_ci.official_base_url",
        )?;
        let android_image_distribution_branch = required_text(
            &download_sources.sources.android_ci.default_branch,
            "download-sources.sources.android_ci.default_branch",
        )?;
        let android_image_distribution_target = required_text(
            &download_sources.sources.android_ci.default_target,
            "download-sources.sources.android_ci.default_target",
        )?;
        validate_distribution_token(&android_image_distribution_branch, "download-sources.sources.android_ci.default_branch")?;
        validate_distribution_token(&android_image_distribution_target, "download-sources.sources.android_ci.default_target")?;
        let android_distribution_branch_templates = parse_android_branch_templates(
            &download_sources.sources.android_ci.branch_templates,
        )?;
        let android_distribution_target_candidates = parse_distribution_tokens(
            &download_sources.sources.android_ci.target_candidates,
            "download-sources.sources.android_ci.target_candidates",
        )?;
        let android_distribution_channels = parse_android_distribution_channels(
            &download_sources.sources.android_ci.channels,
        )?;
        let android_distribution_providers = parse_android_distribution_providers(
            &download_sources.sources.android_release_policy,
            &android_distribution_channels,
            &android_sdk_api_levels,
        )?;
        if config.android.image.distribution.minimum_free_disk_gib < 16 {
            return Err(EngineConfigError::Invalid(String::from(
                "android.image.distribution.minimum_free_disk_gib must be at least 16",
            )));
        }
        let guest_agent_secret_root = resolve_path(
            &project_root,
            required_path(&config.android.guest_agent.secret_root, "android.guest_agent.secret_root")?,
        );
        let guest_agent_host_ip = config.android.guest_agent.host_ip.parse::<Ipv4Addr>().map_err(|_| EngineConfigError::Invalid(String::from("android.guest_agent.host_ip must be IPv4")))?;
        if !guest_agent_host_ip.is_loopback() { return Err(EngineConfigError::Invalid(String::from("android.guest_agent.host_ip must be loopback"))); }
        if config.android.guest_agent.host_port_min == 0 || config.android.guest_agent.host_port_min > config.android.guest_agent.host_port_max || config.android.guest_agent.guest_port == 0 {
            return Err(EngineConfigError::Invalid(String::from("android.guest_agent port range is invalid")));
        }
        if config.android.guest_agent.connect_timeout_ms == 0
            || config.android.guest_agent.io_timeout_ms == 0
            || config.android.guest_agent.ready_timeout_ms == 0
            || config.android.guest_agent.ready_poll_interval_ms == 0
        {
            return Err(EngineConfigError::Invalid(String::from("android.guest_agent timeout values must be greater than zero")));
        }
        if config.runtime.maintenance_interval_ms == 0
            || config.runtime.recovery.retry_delay_ms == 0
            || config.runtime.recovery.max_attempts == 0
            || config.runtime.recovery.journal_retention < 128
        {
            return Err(EngineConfigError::Invalid(String::from(
                "runtime maintenance/recovery values are invalid",
            )));
        }
        let guest_agent_update_manifest_path = resolve_path(
            &project_root,
            required_path(&config.android.guest_agent.update.manifest_path, "android.guest_agent.update.manifest_path")?,
        );
        let guest_agent_update_rollback_root = resolve_path(
            &project_root,
            required_path(&config.android.guest_agent.update.rollback_root, "android.guest_agent.update.rollback_root")?,
        );
        let guest_agent_update_public_key = config.android.guest_agent.update.trusted_public_key_hex.trim().to_owned();
        let guest_agent_update_apk_cert = config.android.guest_agent.update.trusted_apk_cert_sha256.trim().to_owned();
        let guest_agent_update_apksigner = non_empty_path(config.android.guest_agent.update.apksigner_path.clone())
            .map(|value| resolve_path(&project_root, value));
        if config.android.guest_agent.update.enabled
            && (!is_lower_hex(&guest_agent_update_public_key, 64) || !is_lower_hex(&guest_agent_update_apk_cert, 64))
        {
            return Err(EngineConfigError::Invalid(String::from(
                "android.guest_agent.update trust fields must be 64 lowercase hex characters when updates are enabled",
            )));
        }
        let game_catalog_path = resolve_path(&project_root, required_path(&config.game_catalog.path, "game_catalog.path")?);
        let guest_catalog_path = resolve_path(&project_root, required_path(&config.guest_catalog.path, "guest_catalog.path")?);
        let installer_media_relative_path = required_text(&config.guest.installer_media_relative_path, "guest.installer_media_relative_path")?;
        let installer_media_download_root = resolve_path(
            &project_root,
            required_path(&download_sources.paths.installer_media, "download-sources.paths.installer_media")?,
        );
        let installer_media_source_cache_path = resolve_path(
            &project_root,
            required_path(
                &download_sources.paths.installer_media_source_cache,
                "download-sources.paths.installer_media_source_cache",
            )?,
        );
        let installer_media_resolver_policy = parse_installer_media_resolver_policy(
            &download_sources.sources.linux_media,
        )?;
        if !is_safe_relative_iso_path(&installer_media_relative_path) {
            return Err(EngineConfigError::Invalid(String::from(
                "guest.installer_media_relative_path must be a safe relative .iso path",
            )));
        }
        let uefi_code_relative_path = required_text(&config.guest.firmware.uefi.code_relative_path, "guest.firmware.uefi.code_relative_path")?;
        let uefi_vars_relative_path = required_text(&config.guest.firmware.uefi.vars_relative_path, "guest.firmware.uefi.vars_relative_path")?;
        let uefi_code_source_path = if config.guest.firmware.uefi.enabled {
            resolve_path(&project_root, required_path(&config.guest.firmware.uefi.code_source_path, "guest.firmware.uefi.code_source_path")?)
        } else {
            PathBuf::new()
        };
        let uefi_vars_template_source_path = if config.guest.firmware.uefi.enabled {
            resolve_path(&project_root, required_path(&config.guest.firmware.uefi.vars_template_source_path, "guest.firmware.uefi.vars_template_source_path")?)
        } else {
            PathBuf::new()
        };
        let gpu_preference = parse_gpu_preference(&config.gpu.mode)?;
        let gpu_policy = GamingGpuPolicy::create(
            gpu_preference,
            config.gpu.hostmem_mib,
            config.gpu.allow_experimental_android_gfxstream,
        )
        .map_err(|error| EngineConfigError::Invalid(format!("gpu config is invalid: {error:?}")))?;

        Ok(Self {
            host_id: config.host.id,
            host_label: config.host.label,
            api_bind_ip,
            api_port: config.engine_api.port,
            api_timeout: Duration::from_millis(config.engine_api.request_timeout_ms),
            api_auth_token,
            api_tls,
            data_root: data_root.clone(),
            image_root: image_root.clone(),
            storage_policy: StorageEnginePolicy {
                default_disk_format,
                default_disk_bus,
                default_disk_size_gib: config.storage.default_disk.size_gib,
                portable_image: PortableImagePolicy {
                    extension: portable_extension,
                    container: portable_container,
                    private_copy_default: config.storage.portable_image.private_copy_default,
                },
            },
            download_http,
            artifact_cache: ArtifactCacheEngineConfig {
                enabled: config.artifact_cache.enabled,
                root: artifact_cache_root,
                quota_bytes: artifact_cache_quota_bytes,
                verify_on_hit: config.artifact_cache.verify_on_hit,
                mutable: ArtifactCacheMutableEngineConfig {
                    allow_stale_on_transient_error: config.artifact_cache.mutable.allow_stale_on_transient_error,
                    revalidation_timeout: Duration::from_secs(config.artifact_cache.mutable.revalidation_timeout_seconds),
                    require_https: config.artifact_cache.mutable.require_https,
                    allow_private_networks: config.artifact_cache.mutable.allow_private_networks,
                    max_redirects: config.artifact_cache.mutable.max_redirects,
                },
                downloader: ArtifactDownloaderEngineConfig {
                    retry_count: config.artifact_cache.downloader.retry_count,
                    retry_delay_seconds: config.artifact_cache.downloader.retry_delay_seconds,
                    connect_timeout_seconds: config.artifact_cache.downloader.connect_timeout_seconds,
                },
            },
            qemu_runtime: QemuRuntimeSettings {
                qmp_bind_ip,
                qmp_connect_timeout: Duration::from_millis(config.qmp.connect_timeout_ms),
                qmp_read_timeout: Duration::from_millis(config.qmp.read_timeout_ms),
                qmp_write_timeout: Duration::from_millis(config.qmp.write_timeout_ms),
                startup_timeout: Duration::from_millis(config.qmp.startup_timeout_ms),
                startup_poll_interval: Duration::from_millis(config.qmp.startup_poll_interval_ms),
                shutdown_timeout: Duration::from_millis(config.qmp.shutdown_timeout_ms),
                shutdown_poll_interval: Duration::from_millis(config.qmp.shutdown_poll_interval_ms),
                display_mode,
                gpu: QemuGpuRuntimeSettings {
                    backend: GpuBackend::Software,
                    hostmem_mib: gpu_policy.hostmem_mib,
                    experimental: false,
                },
                rfb_bind_ip,
                rfb_display_min: config.runtime.rfb_display_min,
                rfb_display_max: config.runtime.rfb_display_max,
                data_root: data_root.clone(),
                image_root: image_root.clone(),
            },
            runtime_maintenance_interval: Duration::from_millis(config.runtime.maintenance_interval_ms),
            runtime_recovery: RuntimeRecoveryEngineConfig {
                auto_restart: config.runtime.recovery.auto_restart,
                retry_delay: Duration::from_millis(config.runtime.recovery.retry_delay_ms),
                max_attempts: config.runtime.recovery.max_attempts,
                journal_retention: config.runtime.recovery.journal_retention,
            },
            network: NativeNetworkSettings {
                state_root: data_root,
                linux_bridge_helper: bridge_helper,
                linux_allow_managed_tap: config.network.linux.allow_managed_tap,
                windows_managed_helper,
            },
            network_policy: NetworkEnginePolicy {
                default_network_id,
                private_network_id,
                default_device_model: default_network_device_model,
                default_profile: default_network_profile,
                managed_nat: managed_nat_policy,
                private_network: private_network_policy,
            },
            guest: GuestEngineConfig {
                installer_media_relative_path,
                installer_media_download_root,
                installer_media_source_cache_path,
                installer_media_resolver_policy,
                uefi_enabled: config.guest.firmware.uefi.enabled,
                uefi_code_source_path,
                uefi_vars_template_source_path,
                uefi_code_relative_path,
                uefi_vars_relative_path,
            },
            gpu_policy,
            android: AndroidEngineConfig {
                adb_binary,
                adb_host_ip,
                adb_host_port_min: config.android.adb.host_port_min,
                adb_host_port_max: config.android.adb.host_port_max,
                adb_guest_port: config.android.adb.guest_port,
                command_timeout: Duration::from_millis(config.android.adb.command_timeout_ms),
                ready_timeout: Duration::from_millis(config.android.adb.ready_timeout_ms),
                ready_poll_interval: Duration::from_millis(config.android.adb.ready_poll_interval_ms),
                package_root,
                default_width: config.android.default_device.width,
                default_height: config.android.default_device.height,
                default_density_dpi: config.android.default_device.density_dpi,
                default_target_fps: config.android.default_device.target_fps,
                guest_agent: AndroidGuestAgentEngineConfig {
                    host_ip: guest_agent_host_ip,
                    host_port_min: config.android.guest_agent.host_port_min,
                    host_port_max: config.android.guest_agent.host_port_max,
                    guest_port: config.android.guest_agent.guest_port,
                    connect_timeout: Duration::from_millis(config.android.guest_agent.connect_timeout_ms),
                    io_timeout: Duration::from_millis(config.android.guest_agent.io_timeout_ms),
                    ready_timeout: Duration::from_millis(config.android.guest_agent.ready_timeout_ms),
                    ready_poll_interval: Duration::from_millis(config.android.guest_agent.ready_poll_interval_ms),
                    secret_root: guest_agent_secret_root,
                    require_adb_root_for_provisioning: config.android.guest_agent.require_adb_root_for_provisioning,
                    update: AndroidGuestAgentUpdateEngineConfig {
                        enabled: config.android.guest_agent.update.enabled,
                        manifest_path: guest_agent_update_manifest_path,
                        trusted_public_key_hex: guest_agent_update_public_key,
                        trusted_apk_cert_sha256: guest_agent_update_apk_cert,
                        apksigner_binary: guest_agent_update_apksigner,
                        rollback_root: guest_agent_update_rollback_root,
                    },
                },
            },
            android_image: AndroidImageEngineConfig {
                source_root: android_image_source_root,
                output_root: android_image_output_root,
                source_cache_path: android_distribution_source_cache_path,
                build_script: android_image_build_script,
                sdk: AndroidSdkEngineConfig {
                    tool_root: android_sdk_tool_root,
                    repository_base_url: android_sdk_repository_base_url,
                    package_index_url: android_sdk_package_index_url,
                    emulator_package_path: android_sdk_emulator_package_path,
                    architecture: android_sdk_architecture,
                    emulator_console_port_min: android_sdk_console_min,
                    emulator_console_port_max: android_sdk_console_max,
                    variant_priority: android_sdk_variant_priority,
                    system_image_indexes: android_sdk_system_image_indexes,
                    api_levels: android_sdk_api_levels,
                },
                distribution_base_url: android_distribution_base_url,
                distribution_official_base_url: android_distribution_official_base_url,
                distribution_official_fallback: download_sources.sources.android_ci.use_official_fallback,
                distribution_branch_templates: android_distribution_branch_templates,
                distribution_target_candidates: android_distribution_target_candidates,
                distribution_channels: android_distribution_channels,
                distribution_providers: android_distribution_providers,
                distribution_branch: android_image_distribution_branch,
                distribution_target: android_image_distribution_target,
                distribution_minimum_free_disk_gib: config.android.image.distribution.minimum_free_disk_gib,
            },
            game_catalog_path,
            guest_catalog_path,
        })
    }
}

fn load_download_sources(path: &Path) -> Result<DownloadSourcesFile, EngineConfigError> {
    let content = fs::read_to_string(path)
        .map_err(|error| EngineConfigError::Read(format!("download sources read failed: {error}")))?;
    let config: DownloadSourcesFile = serde_yaml_ng::from_str(&content)
        .map_err(|error| EngineConfigError::Parse(format!("download sources parse failed: {error}")))?;
    if config.schema_version != 5 {
        return Err(EngineConfigError::Invalid(format!(
            "unsupported download sources schema {}; expected 5",
            config.schema_version
        )));
    }
    Ok(config)
}

fn parse_installer_media_resolver_policy(
    config: &LinuxMediaSourceConfig,
) -> Result<InstallerMediaResolverPolicy, EngineConfigError> {
    if config.providers.is_empty() {
        return Err(EngineConfigError::Invalid(String::from(
            "Linux installer media provider catalog must not be empty",
        )));
    }
    let mut providers = BTreeMap::new();
    for (provider_id, provider) in &config.providers {
        let provider_id = required_text(provider_id, "Linux media provider id")?;
        let mut base_urls = Vec::new();
        for url in &provider.base_urls {
            let url = validate_base_url(url, "Linux media provider base URL")?;
            if !base_urls.contains(&url) {
                base_urls.push(url);
            }
        }
        if base_urls.is_empty() {
            return Err(EngineConfigError::Invalid(format!(
                "Linux media provider {provider_id} base_urls must not be empty"
            )));
        }
        if provider.rules.is_empty() {
            return Err(EngineConfigError::Invalid(format!(
                "Linux media provider {provider_id} rules must not be empty"
            )));
        }
        let discovery_mode = match provider.discovery_mode {
            LinuxMediaDiscoveryModeConfig::DirectoryIndex => InstallerMediaDiscoveryMode::DirectoryIndex,
            LinuxMediaDiscoveryModeConfig::OfficialPageMirrors => InstallerMediaDiscoveryMode::OfficialPageMirrors,
        };
        let mut rules = BTreeMap::new();
        for (media_kind, rule) in &provider.rules {
            let media_kind = required_text(media_kind, "Linux media kind")?;
            let index_path_templates = if discovery_mode == InstallerMediaDiscoveryMode::DirectoryIndex {
                parse_non_empty_text_list(
                    &rule.index_path_templates,
                    "Linux media index path template",
                )?
            } else {
                parse_optional_text_list(
                    &rule.index_path_templates,
                    "Linux media index path template",
                )?
            };
            let filename_tokens = parse_non_empty_text_list(
                &rule.filename_tokens,
                "Linux media filename token",
            )?;
            let filename_excludes = parse_optional_text_list(
                &rule.filename_excludes,
                "Linux media filename exclude token",
            )?;
            let checksum_tokens = parse_optional_text_list(
                &rule.checksum_tokens,
                "Linux media checksum token",
            )?;
            let checksum_strategy = match rule.checksum_strategy {
                LinuxChecksumStrategyConfig::FixedName => InstallerMediaChecksumStrategy::FixedName,
                LinuxChecksumStrategyConfig::FileSuffix => InstallerMediaChecksumStrategy::FileSuffix,
                LinuxChecksumStrategyConfig::Discover => InstallerMediaChecksumStrategy::Discover,
            };
            let checksum_value = rule.checksum_value.trim().to_owned();
            if matches!(
                checksum_strategy,
                InstallerMediaChecksumStrategy::FixedName | InstallerMediaChecksumStrategy::FileSuffix
            ) && checksum_value.is_empty()
            {
                return Err(EngineConfigError::Invalid(format!(
                    "Linux media provider {provider_id}/{media_kind} checksum_value must not be empty"
                )));
            }
            if checksum_strategy == InstallerMediaChecksumStrategy::Discover && checksum_tokens.is_empty() {
                return Err(EngineConfigError::Invalid(format!(
                    "Linux media provider {provider_id}/{media_kind} discover checksum_tokens must not be empty"
                )));
            }
            rules.insert(
                media_kind,
                InstallerMediaProviderRule {
                    index_path_templates,
                    filename_tokens,
                    filename_excludes,
                    checksum_strategy,
                    checksum_value,
                    checksum_tokens,
                },
            );
        }
        providers.insert(provider_id, InstallerMediaProviderPolicy { discovery_mode, base_urls, rules });
    }
    Ok(InstallerMediaResolverPolicy {
        use_catalog_fallback: config.use_catalog_fallback,
        providers,
    })
}

fn parse_non_empty_text_list(values: &[String], field: &str) -> Result<Vec<String>, EngineConfigError> {
    if values.is_empty() {
        return Err(EngineConfigError::Invalid(format!("{field} must not be empty")));
    }
    parse_optional_text_list(values, field)
}

fn parse_optional_text_list(values: &[String], field: &str) -> Result<Vec<String>, EngineConfigError> {
    let mut parsed = Vec::new();
    for value in values {
        let value = required_text(value, field)?;
        if value.chars().any(char::is_whitespace) {
            return Err(EngineConfigError::Invalid(format!("{field} must not contain whitespace")));
        }
        if !parsed.contains(&value) {
            parsed.push(value);
        }
    }
    Ok(parsed)
}

fn parse_android_distribution_providers(
    policy: &BTreeMap<String, AndroidReleaseProviderConfig>,
    channels: &BTreeMap<String, AndroidDistributionChannelEngineConfig>,
    sdk_api_levels: &BTreeMap<String, u32>,
) -> Result<BTreeMap<String, AndroidDistributionProviderEngineConfig>, EngineConfigError> {
    if policy.is_empty() {
        return Err(EngineConfigError::Invalid(String::from("Android release provider policy must not be empty")));
    }
    let mut parsed = BTreeMap::new();
    for (release, provider) in policy {
        let release = required_text(release, "Android release provider policy")?;
        let provider = match provider {
            AndroidReleaseProviderConfig::AndroidSdk => {
                if !sdk_api_levels.contains_key(&release) {
                    return Err(EngineConfigError::Invalid(format!("Android {release} android_sdk provider requires a configured API level")));
                }
                AndroidDistributionProviderEngineConfig::AndroidSdk
            }
            AndroidReleaseProviderConfig::AndroidCi => {
                if !channels.contains_key(&release) {
                    return Err(EngineConfigError::Invalid(format!("Android {release} android_ci provider requires a configured CI channel")));
                }
                AndroidDistributionProviderEngineConfig::AndroidCi
            }
            AndroidReleaseProviderConfig::SourceBuild => AndroidDistributionProviderEngineConfig::SourceBuild,
        };
        parsed.insert(release, provider);
    }
    Ok(parsed)
}

fn parse_android_distribution_channels(
    channels: &BTreeMap<String, AndroidCiChannelConfig>,
) -> Result<BTreeMap<String, AndroidDistributionChannelEngineConfig>, EngineConfigError> {
    if channels.is_empty() {
        return Err(EngineConfigError::Invalid(String::from("Android CI channel catalog must not be empty")));
    }
    let mut parsed = BTreeMap::new();
    for (release, channel) in channels {
        let release = required_text(release, "download-sources Android release")?;
        let branch_hints = parse_distribution_tokens(
            &channel.branch_hints,
            "Android CI branch hint",
        )?;
        if channel.expected_sdk == 0 {
            return Err(EngineConfigError::Invalid(format!("Android {release} expected_sdk must be greater than zero")));
        }
        parsed.insert(release, AndroidDistributionChannelEngineConfig {
            expected_sdk: channel.expected_sdk,
            allow_device_bootloader_fallback: channel.allow_device_bootloader_fallback,
            branch_hints,
        });
    }
    Ok(parsed)
}

fn parse_distribution_tokens(values: &[String], field: &str) -> Result<Vec<String>, EngineConfigError> {
    if values.is_empty() {
        return Err(EngineConfigError::Invalid(format!("{field} must not be empty")));
    }
    let mut parsed = Vec::new();
    for value in values {
        let value = required_text(value, field)?;
        validate_distribution_token(&value, field)?;
        if !parsed.contains(&value) {
            parsed.push(value);
        }
    }
    Ok(parsed)
}

fn parse_android_branch_templates(values: &[String]) -> Result<Vec<String>, EngineConfigError> {
    if values.is_empty() {
        return Err(EngineConfigError::Invalid(String::from("Android CI branch templates must not be empty")));
    }
    let mut parsed = Vec::new();
    for value in values {
        let value = required_text(value, "Android CI branch template")?;
        if !value.contains("{release}") {
            return Err(EngineConfigError::Invalid(String::from(
                "Android CI branch template must contain {release}",
            )));
        }
        let sample = value.replace("{release}", "17");
        validate_distribution_token(&sample, "Android CI branch template")?;
        if !parsed.contains(&value) {
            parsed.push(value);
        }
    }
    Ok(parsed)
}

fn validate_base_url(value: &str, field: &str) -> Result<String, EngineConfigError> {
    let value = required_text(value, field)?;
    let normalized = value.trim_end_matches('/').to_owned();
    if !(normalized.starts_with("https://") || normalized.starts_with("http://"))
        || normalized.chars().any(char::is_whitespace)
    {
        return Err(EngineConfigError::Invalid(format!("{field} must be an HTTP(S) base URL")));
    }
    Ok(normalized)
}

fn default_true() -> bool { true }

fn is_safe_relative_iso_path(value: &str) -> bool {
    let path = Path::new(value);
    !value.trim().is_empty()
        && value.to_ascii_lowercase().ends_with(".iso")
        && !path.is_absolute()
        && !value.contains('\\')
        && !value.contains(':')
        && path.components().all(|component| matches!(component, std::path::Component::Normal(_) | std::path::Component::CurDir))
}

fn required_text(value: &str, field: &str) -> Result<String, EngineConfigError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(EngineConfigError::Invalid(format!("{field} must not be empty")));
    }
    Ok(value.to_owned())
}

fn validate_distribution_token(value: &str, field: &str) -> Result<(), EngineConfigError> {
    if !value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.')) {
        return Err(EngineConfigError::Invalid(format!("{field} contains unsupported characters")));
    }
    Ok(())
}

fn is_lower_hex(value: &str, expected_len: usize) -> bool {
    value.len() == expected_len
        && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn validate_identity(config: &HostConfig) -> Result<(), EngineConfigError> {
    if config.id.trim().is_empty() || config.label.trim().is_empty() {
        return Err(EngineConfigError::Invalid(String::from(
            "host.id and host.label must not be empty",
        )));
    }
    if !config
        .id
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(EngineConfigError::Invalid(String::from(
            "host.id contains unsupported characters",
        )));
    }
    Ok(())
}

fn load_auth_token(
    config: &EngineAuthenticationConfig,
    project_root: &Path,
) -> Result<Option<String>, EngineConfigError> {
    match config.mode.as_str() {
        "none" => Ok(None),
        "token" => {
            let value = std::env::var(ENGINE_TOKEN_ENV).ok().or_else(|| {
                non_empty_path(config.token_file.clone()).and_then(|path| {
                    fs::read_to_string(resolve_path(project_root, path)).ok()
                })
            });
            let token = value
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    EngineConfigError::Invalid(String::from(
                        "Token authentication is enabled but no token is available",
                    ))
                })?;
            if token.len() < MIN_REMOTE_TOKEN_LENGTH {
                return Err(EngineConfigError::Invalid(format!(
                    "Remote authentication token must be at least {MIN_REMOTE_TOKEN_LENGTH} characters"
                )));
            }
            Ok(Some(token))
        }
        other => Err(EngineConfigError::Invalid(format!(
            "Unsupported engine_api.authentication.mode: {other}"
        ))),
    }
}

fn load_tls_settings(
    config: &EngineTlsConfig,
    project_root: &Path,
) -> Result<Option<TlsServerSettings>, EngineConfigError> {
    if !config.enabled {
        return Ok(None);
    }
    let certificate_path = required_path(&config.certificate_path, "engine_api.tls.certificate_path")?;
    let private_key_path = required_path(&config.private_key_path, "engine_api.tls.private_key_path")?;
    Ok(Some(TlsServerSettings {
        certificate_path: resolve_path(project_root, certificate_path),
        private_key_path: resolve_path(project_root, private_key_path),
    }))
}

fn validate_remote_security(
    bind_ip: IpAddr,
    auth_token: Option<&str>,
    tls: Option<&TlsServerSettings>,
) -> Result<(), EngineConfigError> {
    if bind_ip.is_loopback() {
        return Ok(());
    }
    if auth_token.is_none() || tls.is_none() {
        return Err(EngineConfigError::Invalid(String::from(
            "Non-loopback Engine API requires TLS and token authentication",
        )));
    }
    Ok(())
}

fn default_gpu_mode() -> String {
    String::from(DEFAULT_GPU_MODE)
}

const fn default_gpu_hostmem_mib() -> u64 {
    DEFAULT_GPU_HOSTMEM_MIB
}

fn parse_gpu_preference(value: &str) -> Result<GpuBackendPreference, EngineConfigError> {
    match value {
        "auto" => Ok(GpuBackendPreference::Auto),
        "software" => Ok(GpuBackendPreference::Software),
        "virtio_2d" => Ok(GpuBackendPreference::Virtio2d),
        "virgl_venus" => Ok(GpuBackendPreference::VirglVenus),
        "gfxstream" => Ok(GpuBackendPreference::Gfxstream),
        other => Err(EngineConfigError::Invalid(format!(
            "Unsupported gpu.mode: {other}"
        ))),
    }
}

fn parse_managed_network_pool(
    config: &ManagedNetworkConfig,
    field: &str,
) -> Result<ManagedNetworkPoolPolicy, EngineConfigError> {
    let subnet = config
        .subnet
        .parse::<Ipv4Addr>()
        .map_err(|_| EngineConfigError::Invalid(format!("{field}.subnet must be IPv4")))?;
    let gateway = config
        .gateway
        .parse::<Ipv4Addr>()
        .map_err(|_| EngineConfigError::Invalid(format!("{field}.gateway must be IPv4")))?;
    if config.prefix_length != 24 {
        return Err(EngineConfigError::Invalid(format!(
            "{field}.prefix_length must be 24 in schema 20"
        )));
    }
    if config.pool_start < 2 || config.pool_start > config.pool_end || config.pool_end > 254 {
        return Err(EngineConfigError::Invalid(format!(
            "{field} pool must stay between host 2 and 254"
        )));
    }
    let subnet_octets = subnet.octets();
    let gateway_octets = gateway.octets();
    if subnet_octets[..3] != gateway_octets[..3] || subnet_octets[3] != 0 {
        return Err(EngineConfigError::Invalid(format!(
            "{field}.gateway must belong to the configured /24 subnet"
        )));
    }
    let mut dns_servers = Vec::with_capacity(config.dns_servers.len());
    for value in &config.dns_servers {
        dns_servers.push(value.parse::<Ipv4Addr>().map_err(|_| {
            EngineConfigError::Invalid(format!("{field}.dns_servers must contain IPv4 addresses"))
        })?);
    }
    Ok(ManagedNetworkPoolPolicy {
        subnet,
        prefix_length: config.prefix_length,
        gateway,
        pool_start: config.pool_start,
        pool_end: config.pool_end,
        dns_servers,
    })
}

fn locate_config() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os(CONFIG_ENV).map(PathBuf::from) {
        if path.is_file() {
            return Some(path);
        }
    }

    let current = std::env::current_dir().ok()?;
    for directory in current.ancestors() {
        let candidate = directory.join(DEFAULT_CONFIG_PATH);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn project_root(path: &Path) -> Result<PathBuf, EngineConfigError> {
    path.parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| EngineConfigError::Invalid(String::from("Invalid config path")))
}

fn parse_ip(value: &str, field: &str) -> Result<IpAddr, EngineConfigError> {
    value
        .parse::<IpAddr>()
        .map_err(|_| EngineConfigError::Invalid(format!("Invalid IP in {field}: {value}")))
}

fn non_empty_path(value: String) -> Option<PathBuf> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(PathBuf::from(trimmed))
    }
}

fn required_path(value: &str, field: &str) -> Result<PathBuf, EngineConfigError> {
    if value.trim().is_empty() {
        Err(EngineConfigError::Invalid(format!("{field} must not be empty")))
    } else {
        Ok(PathBuf::from(value.trim()))
    }
}

fn resolve_path(root: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        root.join(path)
    }
}
