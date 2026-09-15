// # 📄 Dosya Yolu: /turkuazvm/crates/engine-api/src/lib.rs
// # 📌 Amac: TurkuazVM Engine API public contractlarini disariya acar
// # 📌 Modul - Rust
// # Version: 0.40.1
// # Aciklama: API v24; installer medya icin geriye uyumlu managed-download capability bilgisini de tasiyan Engine kontratlarini tanimlar
// # Bagimli Oldugu Katman: Controller | Service | Tool | View

use serde::{Deserialize, Serialize};

pub const ENGINE_API_VERSION: u16 = 24;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EngineRequest {
    pub request_id: u64,
    pub api_version: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,
    #[serde(flatten)]
    pub action: EngineAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum EngineAction {
    Ping,
    Dashboard,
    GetGpuCapabilities,
    ListVms,
    CreateVm {
        vm_id: String,
        name: String,
        vcpu_count: u16,
        memory_mib: u64,
        guest_profile: GuestProfileDto,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        guest_template_id: Option<String>,
    },
    GetStorageOverview,
    GetArtifactCacheOverview,
    ListArtifactCacheEntries,
    SetArtifactCachePinned { source_key: String, pinned: bool },
    RemoveArtifactCacheEntry { source_key: String },
    VerifyArtifactCache,
    CleanupArtifactCache,
    RevalidateArtifactCache { source_key: String },
    RevalidateAllArtifactCache,
    FetchMutableArtifactCache { source_key: String, source_url: String, pinned: bool },
    CreateVmDisk { vm_id: String, disk_id: String, size_gib: u64 },
    ResizeVmDisk { vm_id: String, disk_id: String, size_gib: u64 },
    DeleteVmDisk { vm_id: String, disk_id: String },
    UpdateVm { vm_id: String, name: String, vcpu_count: u16, memory_mib: u64 },
    DeleteVm { vm_id: String },
    ConfigureInstallerMedia { vm_id: String, source_path: String },
    EjectInstallerMedia { vm_id: String },
    StartInstallerMediaDownload { guest_template_id: String, media_id: String },
    GetInstallerMediaDownload { guest_template_id: String, media_id: String },
    CancelInstallerMediaDownload { guest_template_id: String, media_id: String },
    AttachDownloadedInstallerMedia { vm_id: String, guest_template_id: String, media_id: String },
    GetNetworkOverview,
    AttachDefaultNetwork { vm_id: String, network_id: String },
    AttachNetworkProfile {
        vm_id: String,
        network_id: String,
        profile: NetworkProfileDto,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        bridge_name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tap_name: Option<String>,
    },
    PublishVmService {
        vm_id: String,
        network_id: String,
        protocol: NetworkProtocolDto,
        host_port: u16,
        guest_port: u16,
    },
    UnpublishVmService {
        vm_id: String,
        network_id: String,
        protocol: NetworkProtocolDto,
        host_port: u16,
    },
    DetachVmNetwork { vm_id: String, network_id: String },
    StartVm { vm_id: String },
    StopVm { vm_id: String },
    GetDisplaySession { vm_id: String },
    CreateSnapshot {
        vm_id: String,
        snapshot_id: String,
        name: String,
    },
    ListSnapshots { vm_id: String },
    RestoreSnapshot { vm_id: String, snapshot_id: String },
    DeleteSnapshot { vm_id: String, snapshot_id: String },
    CloneVm {
        source_vm_id: String,
        target_vm_id: String,
        target_name: String,
        mode: CloneModeDto,
    },
    ConfigureAndroidRuntime {
        vm_id: String,
        width: Option<u32>,
        height: Option<u32>,
        density_dpi: Option<u32>,
        target_fps: Option<u16>,
    },
    GetAndroidProfile { vm_id: String },
    GetAndroidStatus { vm_id: String },
    WaitAndroidReady { vm_id: String },
    ApplyAndroidDisplay { vm_id: String },
    ListAndroidPackages { vm_id: String },
    InstallAndroidApk {
        vm_id: String,
        relative_apk_path: String,
    },
    UninstallAndroidPackage { vm_id: String, package_name: String },
    LaunchAndroidPackage { vm_id: String, package_name: String },
    StopAndroidPackage { vm_id: String, package_name: String },
    InjectAndroidInput { vm_id: String, input: AndroidInputDto },
    GetGamingInputCapabilities { vm_id: String },
    GetGamingInputProfile { vm_id: String },
    ConfigureGamingInputProfile { profile: GamingInputProfileConfigDto },
    InjectGamingInputEvent { vm_id: String, event: GamingInputEventDto },
    ResetGamingInputState { vm_id: String },
    GetGuestAgentStatus { vm_id: String },
    GetGuestClipboard { vm_id: String },
    SetGuestClipboard { vm_id: String, text: String },
    ListGameCatalog,
    ListGuestCatalog,
    DetectGames { vm_id: String },
    GetGameCompatibility { vm_id: String, game_id: String },
    ApplyGameProfile { vm_id: String, game_id: String },
    ListAndroidImages,
    DefineAndroidImage { image_id: String, name: String, #[serde(default, skip_serializing_if = "Option::is_none")] requested_release: Option<String> },
    PrepareAndroidImageBuild { image_id: String },
    RegisterAndroidImageBuild { image_id: String },
    InstallAndroidImageDistribution { image_id: String },
    CancelAndroidImageDistribution { image_id: String },
    CleanupAndroidImageDistribution { image_id: String },
    AssignAndroidImage { vm_id: String, image_id: String },
    GetAndroidImageAssignment { vm_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EngineResponse {
    pub request_id: u64,
    pub api_version: u16,
    pub ok: bool,
    pub data: Option<EngineResponseData>,
    pub error: Option<EngineApiError>,
}

impl EngineResponse {
    pub fn success(request_id: u64, data: EngineResponseData) -> Self {
        Self {
            request_id,
            api_version: ENGINE_API_VERSION,
            ok: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn failure(request_id: u64, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            request_id,
            api_version: ENGINE_API_VERSION,
            ok: false,
            data: None,
            error: Some(EngineApiError {
                code: code.into(),
                message: message.into(),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EngineResponseData {
    Pong,
    Dashboard(DashboardDto),
    GpuCapabilities(GpuCapabilitiesDto),
    VmList { machines: Vec<VmSummaryDto> },
    Vm { machine: VmSummaryDto },
    StorageOverview(StorageOverviewDto),
    ArtifactCacheOverview(ArtifactCacheOverviewDto),
    ArtifactCacheList { entries: Vec<ArtifactCacheEntryDto> },
    ArtifactCacheEntry(ArtifactCacheEntryDto),
    ArtifactCacheVerification(ArtifactCacheVerificationDto),
    ArtifactCacheRevalidation(ArtifactCacheRevalidationDto),
    ArtifactCacheRevalidationSummary(ArtifactCacheRevalidationSummaryDto),
    InstallerMediaDownload(InstallerMediaDownloadDto),
    NetworkOverview(NetworkOverviewDto),
    Snapshot { snapshot: SnapshotDto },
    SnapshotList { snapshots: Vec<SnapshotDto> },
    DisplaySession(DisplaySessionDto),
    AndroidProfile(AndroidRuntimeProfileDto),
    AndroidStatus(AndroidDeviceStatusDto),
    AndroidPackageList { packages: Vec<AndroidPackageDto> },
    GamingInputCapabilities(GamingInputCapabilitiesDto),
    GamingInputProfile(GamingInputProfileDto),
    GuestAgentStatus(GuestAgentStatusDto),
    Clipboard { text: String },
    GameCatalogList { games: Vec<GameCatalogEntryDto> },
    GuestCatalogList { templates: Vec<GuestTemplateDto> },
    DetectedGames { games: Vec<GameCatalogEntryDto> },
    GameCompatibility(GameCompatibilityDto),
    GameProfileApplied { game: GameCatalogEntryDto, android: AndroidRuntimeProfileDto, gaming_input: GamingInputProfileDto },
    AndroidImageList { images: Vec<AndroidImageDto> },
    AndroidImage { image: AndroidImageDto },
    AndroidImageBuildPlan(AndroidImageBuildPlanDto),
    AndroidImageAssignment { assignment: Option<AndroidImageAssignmentDto> },
    Ack,
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StorageOverviewDto {
    pub data_root: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub qemu_img_available: bool,
    pub default_runtime_format: String,
    pub default_disk_size_gib: u64,
    pub portable_image_extension: String,
    pub portable_container: String,
    pub private_copy_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactCacheOverviewDto {
    pub enabled: bool,
    pub verify_on_hit: bool,
    pub allow_stale_on_transient_error: bool,
    pub artifact_count: usize,
    pub pinned_count: usize,
    pub mutable_count: usize,
    pub used_bytes: u64,
    pub quota_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactCacheEntryDto {
    pub source_key: String,
    pub source_url: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub immutable: bool,
    pub pinned: bool,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub created_at_unix_ms: u64,
    pub last_access_unix_ms: u64,
    pub last_revalidated_unix_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactCacheVerificationDto {
    pub checked: usize,
    pub valid: usize,
    pub invalid: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactCacheRevalidationStateDto {
    Immutable,
    NotModified,
    RemoteModified,
    ValidationFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactCacheRevalidationDto {
    pub source_key: String,
    pub state: ArtifactCacheRevalidationStateDto,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactCacheRevalidationSummaryDto {
    pub checked: usize,
    pub immutable: usize,
    pub not_modified: usize,
    pub remote_modified: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VmDiskDto {
    pub id: String,
    pub format: String,
    pub virtual_size_bytes: u64,
    pub relative_path: String,
    pub bus: String,
    pub boot_index: Option<u8>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NetworkProfileDto {
    ManagedNat,
    Private,
    Bridge,
    UserNat,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NetworkProtocolDto {
    Tcp,
    Udp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PublishedNetworkServiceDto {
    pub protocol: String,
    pub host_ip: String,
    pub host_port: u16,
    pub guest_ip: String,
    pub guest_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VmNetworkDto {
    pub id: String,
    pub fabric_id: Option<String>,
    pub mode: String,
    pub device_model: String,
    pub mac_address: Option<String>,
    pub ipv4_address: Option<String>,
    pub prefix_length: Option<u8>,
    pub gateway: Option<String>,
    pub dns_servers: Vec<String>,
    pub port_forward_count: usize,
    pub published_services: Vec<PublishedNetworkServiceDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkOverviewDto {
    pub managed_nat: bool,
    pub private_network: bool,
    pub user_nat: bool,
    pub bridge: bool,
    pub existing_tap: bool,
    pub managed_tap: bool,
    pub bridge_helper: bool,
    pub default_network_id: String,
    pub default_device_model: String,
    pub default_profile: String,
    pub managed_nat_subnet: String,
    pub managed_nat_gateway: String,
    pub managed_nat_pool: String,
    pub private_subnet: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GuestProfileDto {
    Generic,
    Linux,
    Windows,
    Android,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DisplaySessionDto {
    pub vm_id: String,
    pub transport: String,
    pub endpoint: String,
    pub local_only: bool,
    pub fullscreen: bool,
    pub absolute_pointer: bool,
    pub relative_pointer_capture: bool,
    pub keyboard: bool,
    pub gamepad_observation: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CloneModeDto {
    Full,
    Linked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapshotDto {
    pub id: String,
    pub name: String,
    pub created_at_unix_ms: u64,
    pub disk_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AndroidRuntimeProfileDto {
    pub vm_id: String,
    pub adb_endpoint: String,
    pub adb_guest_port: u16,
    pub width: u32,
    pub height: u32,
    pub density_dpi: u32,
    pub target_fps: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AndroidDeviceStatusDto {
    pub vm_id: String,
    pub serial: String,
    pub connection_state: String,
    pub boot_completed: bool,
    pub sdk_level: Option<u32>,
    pub abi: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AndroidPackageDto {
    pub package_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AndroidInputDto {
    Tap { x: u32, y: u32 },
    Swipe {
        from_x: u32,
        from_y: u32,
        to_x: u32,
        to_y: u32,
        duration_ms: u32,
    },
    KeyEvent { key_code: u32 },
    Text { value: String },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct NormalizedPointDto {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GamingKeyDto {
    W,
    A,
    S,
    D,
    Space,
    C,
    Z,
    R,
    F,
    Q,
    E,
    ShiftLeft,
    ControlLeft,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Tab,
    Escape,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GamingMouseButtonDto {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GamingInputSourceDto {
    Key { key: GamingKeyDto },
    MouseButton { button: GamingMouseButtonDto },
    GamepadButton { button: u16 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GamingInputTargetDto {
    Tap { position: NormalizedPointDto },
    HoldTouch { pointer_id: u8, position: NormalizedPointDto },
    AndroidKey { key_code: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GamingInputBindingDto {
    pub source: GamingInputSourceDto,
    pub target: GamingInputTargetDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VirtualJoystickDto {
    pub enabled: bool,
    pub center: NormalizedPointDto,
    pub radius: u16,
    pub pointer_id: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MouseLookDto {
    pub enabled: bool,
    pub anchor: NormalizedPointDto,
    pub sensitivity_x_milli: u16,
    pub sensitivity_y_milli: u16,
    pub pointer_id: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GamingInputProfileConfigDto {
    pub vm_id: String,
    pub package_name: Option<String>,
    pub joystick: VirtualJoystickDto,
    pub mouse_look: MouseLookDto,
    pub bindings: Vec<GamingInputBindingDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GamingInputProfileDto {
    pub vm_id: String,
    pub package_name: Option<String>,
    pub joystick: VirtualJoystickDto,
    pub mouse_look: MouseLookDto,
    pub bindings: Vec<GamingInputBindingDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuestAgentStatusDto {
    pub available: bool,
    pub persistent_multi_touch: bool,
    pub max_contacts: u8,
    pub continuation_api: bool,
    pub input_backend: Option<String>,
    pub endpoint: Option<String>,
    pub secure_transport_v2: bool,
    pub clipboard: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GameCatalogMaturityDto { Experimental, Playable, Recommended }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GameCatalogGpuBackendDto { Software, Virtio2d, VirglVenus, Gfxstream }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GameCompatibilityStatusDto { Blocked, Experimental, Playable, Recommended }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameCatalogEntryDto {
    pub id: String,
    pub name: String,
    pub packages: Vec<String>,
    pub maturity: GameCatalogMaturityDto,
    pub emulator_disclosure_required: bool,
    pub minimum_sdk: Option<u32>,
    pub persistent_multi_touch_required: bool,
    pub relative_mouse_look_required: bool,
    pub preferred_gpu_backends: Vec<GameCatalogGpuBackendDto>,
    pub width: u32,
    pub height: u32,
    pub density_dpi: u32,
    pub target_fps: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameCompatibilityDto {
    pub game: GameCatalogEntryDto,
    pub status: GameCompatibilityStatusDto,
    pub package_installed: bool,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GamingInputCapabilitiesDto {
    pub profile_editor: bool,
    pub raw_keyboard: bool,
    pub raw_mouse_motion: bool,
    pub adb_single_touch_fallback: bool,
    pub persistent_multi_touch: bool,
    pub virtual_joystick: bool,
    pub relative_mouse_look: bool,
    pub host_gamepad_observation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GamingInputEventDto {
    Key { key: GamingKeyDto, pressed: bool },
    MouseButton { button: GamingMouseButtonDto, pressed: bool },
    MouseMotion { delta_x: i32, delta_y: i32 },
    MouseCapture { captured: bool },
    GamepadButton { button: u16, pressed: bool },
    GamepadAxis { axis: u16, value_milli: i16 },
}


#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AndroidImageStateDto {
    Defined,
    BuildPlanned,
    Installing,
    Ready,
    Failed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AndroidImageArchitectureDto {
    X86_64,
    Arm64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AndroidImageArtifactRoleDto {
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
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AndroidImageArtifactDto {
    pub role: AndroidImageArtifactRoleDto,
    pub relative_path: String,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AndroidImageCapabilitiesDto {
    pub adb_tcp: bool,
    pub guest_agent_included: bool,
    pub persistent_multi_touch: bool,
    pub native_x86_64: bool,
    pub arm_translation: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AndroidImageInstallStageDto {
    Discovering,
    CheckingDisk,
    DownloadingDevice,
    DownloadingHost,
    Validating,
    Extracting,
    Assembling,
    Finalizing,
    Cancelling,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AndroidImageInstallProgressDto {
    pub stage: AndroidImageInstallStageDto,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub detail: String,
    pub log_path: String,
    pub elapsed_seconds: u64,
    pub bytes_per_second: Option<u64>,
    pub eta_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AndroidImageDto {
    pub id: String,
    pub name: String,
    pub architecture: AndroidImageArchitectureDto,
    pub state: AndroidImageStateDto,
    pub branch: String,
    pub lunch_target: String,
    pub capabilities: AndroidImageCapabilitiesDto,
    pub requested_release: Option<String>,
    pub source_revision: Option<String>,
    pub android_release: Option<String>,
    pub sdk_level: Option<u32>,
    pub artifacts: Vec<AndroidImageArtifactDto>,
    pub last_error: Option<String>,
    pub install_progress: Option<AndroidImageInstallProgressDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AndroidImageBuildPlanDto {
    pub supported_host: bool,
    pub source_root: String,
    pub output_root: String,
    pub branch: String,
    pub lunch_target: String,
    pub build_script: String,
    pub minimum_free_disk_gib: u64,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AndroidImageProvisioningStateDto {
    PendingFirstBoot,
    Ready,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AndroidImageAssignmentDto {
    pub vm_id: String,
    pub image_id: String,
    pub provisioning_state: AndroidImageProvisioningStateDto,
    pub boot_attempts: u32,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EngineApiError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GpuCapabilitiesDto {
    pub requested_mode: String,
    pub effective_backend: String,
    pub hostmem_mib: u64,
    pub experimental: bool,
    pub vulkan_loader_available: bool,
    pub vulkan_probe_available: bool,
    pub vulkan_summary: Option<String>,
    pub opengl_available: bool,
    pub qemu_virtio_2d: bool,
    pub qemu_virgl: bool,
    pub qemu_venus: bool,
    pub qemu_rutabaga: bool,
    pub qemu_gfxstream_vulkan: bool,
    pub android_gfxstream_experimental: bool,
    pub resolution_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstallerMediaSourceDto {
    pub id: String,
    pub mode: String,
    #[serde(default)]
    pub managed_download: bool,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstallerMediaDownloadDto {
    pub guest_template_id: String,
    pub media_id: String,
    pub state: String,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub local_path: Option<String>,
    pub sha256: Option<String>,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuestTemplateDto {
    pub id: String,
    pub family: String,
    pub family_label: String,
    pub product_id: String,
    pub product_label: String,
    pub release_id: String,
    pub release_label: String,
    pub profile_id: String,
    pub profile_label: String,
    pub guest_profile: GuestProfileDto,
    pub architecture: String,
    pub source_kind: String,
    pub source_label: String,
    pub firmware: String,
    pub recommended_vcpu_count: u16,
    pub recommended_memory_mib: u64,
    pub recommended_disk_size_gib: u64,
    pub installer_media: Option<InstallerMediaSourceDto>,
    pub installer_media_options: Vec<InstallerMediaSourceDto>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DashboardDto {
    pub engine_ready: bool,
    pub host_id: String,
    pub host_label: String,
    pub transport_security: String,
    pub authentication_required: bool,
    pub platform: String,
    pub architecture: String,
    pub acceleration: String,
    pub qemu_ready: bool,
    pub qemu_img_ready: bool,
    pub qemu_version: Option<String>,
    pub gpu_backend: String,
    pub gpu_vulkan_ready: bool,
    pub gpu_accelerated: bool,
    pub android_adb_ready: bool,
    pub android_adb_version: Option<String>,
    pub machines: Vec<VmSummaryDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VmSummaryDto {
    pub id: String,
    pub name: String,
    pub state: String,
    pub vcpu_count: u16,
    pub memory_mib: u64,
    pub acceleration: String,
    pub disk_count: usize,
    pub network_count: usize,
    pub snapshot_count: usize,
    pub guest_profile: String,
    pub guest_template_id: Option<String>,
    pub installer_media: Option<String>,
    pub disks: Vec<VmDiskDto>,
    pub networks: Vec<VmNetworkDto>,
    pub last_error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{
        AndroidImageAssignmentDto, AndroidInputDto, DisplaySessionDto, EngineAction, EngineRequest, EngineResponse,
        EngineResponseData, GamingInputEventDto, GamingKeyDto, GuestProfileDto, ENGINE_API_VERSION,
    };

    const REQUEST_ID: u64 = 42;
    const VM_ID: &str = "vm-alpha";
    const TOKEN: &str = "test-token";

    #[test]
    fn request_round_trip_preserves_versioned_authenticated_action() {
        let request = EngineRequest {
            request_id: REQUEST_ID,
            api_version: ENGINE_API_VERSION,
            auth_token: Some(TOKEN.to_owned()),
            action: EngineAction::StartVm {
                vm_id: VM_ID.to_owned(),
            },
        };
        let encoded = serde_json::to_string(&request).expect("request serialization failed");
        let decoded: EngineRequest = serde_json::from_str(&encoded).expect("request deserialization failed");
        assert_eq!(decoded, request);
    }

    #[test]
    fn installer_media_eject_round_trip_preserves_vm_id() {
        let request = EngineRequest {
            request_id: REQUEST_ID,
            api_version: ENGINE_API_VERSION,
            auth_token: None,
            action: EngineAction::EjectInstallerMedia {
                vm_id: VM_ID.to_owned(),
            },
        };
        let encoded = serde_json::to_string(&request).expect("eject request serialization failed");
        let decoded: EngineRequest = serde_json::from_str(&encoded).expect("eject request deserialization failed");
        assert_eq!(decoded, request);
    }

    #[test]
    fn create_android_vm_round_trip_preserves_guest_profile() {
        let request = EngineRequest {
            request_id: REQUEST_ID,
            api_version: ENGINE_API_VERSION,
            auth_token: None,
            action: EngineAction::CreateVm {
                vm_id: VM_ID.to_owned(),
                name: String::from("Android Gaming"),
                vcpu_count: 8,
                memory_mib: 8192,
                guest_profile: GuestProfileDto::Android,
                guest_template_id: Some(String::from("android-17-gaming-phone")),
            },
        };
        let encoded = serde_json::to_string(&request).expect("request serialization failed");
        let decoded: EngineRequest = serde_json::from_str(&encoded).expect("request deserialization failed");
        assert_eq!(decoded, request);
    }

    #[test]
    fn android_input_round_trip_is_typed() {
        let input = AndroidInputDto::Swipe {
            from_x: 100,
            from_y: 200,
            to_x: 300,
            to_y: 400,
            duration_ms: 120,
        };
        let encoded = serde_json::to_string(&input).expect("input serialization failed");
        let decoded: AndroidInputDto = serde_json::from_str(&encoded).expect("input deserialization failed");
        assert_eq!(decoded, input);
    }

    #[test]
    fn display_session_round_trip_preserves_capabilities() {
        let session = DisplaySessionDto {
            vm_id: VM_ID.to_owned(),
            transport: "rfb".to_owned(),
            endpoint: "127.0.0.1:5910".to_owned(),
            local_only: true,
            fullscreen: true,
            absolute_pointer: true,
            relative_pointer_capture: true,
            keyboard: true,
            gamepad_observation: false,
        };
        let response = EngineResponse::success(
            REQUEST_ID,
            EngineResponseData::DisplaySession(session.clone()),
        );
        let encoded = serde_json::to_string(&response).expect("response serialization failed");
        let decoded: EngineResponse = serde_json::from_str(&encoded).expect("response deserialization failed");
        assert_eq!(decoded.data, Some(EngineResponseData::DisplaySession(session)));
    }


    #[test]
    fn gaming_input_event_round_trip_is_typed() {
        let request = EngineRequest {
            request_id: REQUEST_ID,
            api_version: ENGINE_API_VERSION,
            auth_token: None,
            action: EngineAction::InjectGamingInputEvent {
                vm_id: VM_ID.to_owned(),
                event: GamingInputEventDto::Key {
                    key: GamingKeyDto::Space,
                    pressed: true,
                },
            },
        };
        let encoded = serde_json::to_string(&request).expect("gaming request serialization failed");
        let decoded: EngineRequest = serde_json::from_str(&encoded).expect("gaming request deserialization failed");
        assert_eq!(decoded, request);
    }

    #[test]
    fn gpu_capability_request_round_trip_is_versioned() {
        let request = EngineRequest {
            request_id: REQUEST_ID,
            api_version: ENGINE_API_VERSION,
            auth_token: None,
            action: EngineAction::GetGpuCapabilities,
        };
        let encoded = serde_json::to_string(&request).expect("request serialization failed");
        let decoded: EngineRequest = serde_json::from_str(&encoded).expect("request deserialization failed");
        assert_eq!(decoded, request);
    }


    #[test]
    fn android_image_assignment_round_trip_is_versioned() {
        let response = EngineResponse::success(
            REQUEST_ID,
            EngineResponseData::AndroidImageAssignment {
                assignment: Some(AndroidImageAssignmentDto {
                    vm_id: VM_ID.to_owned(),
                    image_id: String::from("turkuaz-android-x86_64"),
                    provisioning_state: super::AndroidImageProvisioningStateDto::PendingFirstBoot,
                    boot_attempts: 0,
                    last_error: None,
                }),
            },
        );
        let encoded = serde_json::to_string(&response).expect("android image response serialization failed");
        let decoded: EngineResponse = serde_json::from_str(&encoded).expect("android image response deserialization failed");
        assert_eq!(decoded, response);
    }

    #[test]
    fn android_image_cancel_action_round_trip_is_versioned() {
        let request = EngineRequest {
            request_id: REQUEST_ID,
            api_version: ENGINE_API_VERSION,
            auth_token: None,
            action: EngineAction::CancelAndroidImageDistribution {
                image_id: String::from("turkuaz-android-x86_64"),
            },
        };
        let encoded = serde_json::to_string(&request).expect("android image cancel serialization failed");
        let decoded: EngineRequest = serde_json::from_str(&encoded).expect("android image cancel deserialization failed");
        assert_eq!(decoded, request);
    }

    #[test]
    fn failure_response_has_no_data() {
        let response = EngineResponse::failure(REQUEST_ID, "test_error", "test failure");
        assert!(!response.ok);
        assert!(response.data.is_none());
        assert!(response.error.is_some());
    }
}
