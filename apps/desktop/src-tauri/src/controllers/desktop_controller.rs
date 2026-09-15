// # 📄 Dosya Yolu: /turkuazvm/apps/desktop/src-tauri/src/controllers/desktop_controller.rs
// # 📌 Amac: Tauri invoke requestlerini DesktopService use-case'lerine yonlendirir
// # 📌 Modul - Rust
// # Version: 0.36.0
// # Aciklama: Controller logic eklemeden VM, ag, medya, Connection Center, log ve Android Runtime requestlerini Service katmanina aktarir
// # Bagimli Oldugu Katman: Service

use serde::Deserialize;
use tauri::State;
use turkuazvm_engine_api::{
    AndroidDeviceStatusDto, AndroidImageAssignmentDto, AndroidImageBuildPlanDto, AndroidImageDto,
    AndroidInputDto, AndroidPackageDto, AndroidRuntimeProfileDto,
    ArtifactCacheEntryDto, ArtifactCacheOverviewDto, ArtifactCacheRevalidationDto,
    ArtifactCacheRevalidationSummaryDto, ArtifactCacheVerificationDto, CloneModeDto, DashboardDto, DisplaySessionDto, GamingInputCapabilitiesDto,
    GamingInputProfileConfigDto, GamingInputProfileDto, GameCatalogEntryDto,
    GameCompatibilityDto, GuestAgentStatusDto, GpuCapabilitiesDto, GuestProfileDto, GuestTemplateDto,
    InstallerMediaDownloadDto, NetworkOverviewDto, NetworkProfileDto, NetworkProtocolDto, SnapshotDto, StorageOverviewDto, VmSummaryDto,
};

use crate::services::app_state::DesktopAppState;
use crate::services::desktop_service::{AppliedGameProfileSummary, ConnectionProbeSummary, DesktopService, DownloadSettingsSummary, HostProfileSummary, LocalLogEntrySummary};

#[derive(Debug, Deserialize)]
pub struct ArtifactCacheSourceRequest {
    pub source_key: String,
}


#[derive(Debug, Deserialize)]
pub struct ArtifactCacheFetchRequest {
    pub source_key: String,
    pub source_url: String,
    pub pinned: bool,
}

#[derive(Debug, Deserialize)]
pub struct ArtifactCachePinRequest {
    pub source_key: String,
    pub pinned: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateVmRequest {
    pub vm_id: String,
    pub name: String,
    pub vcpu_count: u16,
    pub memory_mib: u64,
    pub guest_profile: GuestProfileDto,
    #[serde(default)]
    pub guest_template_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateVmDiskRequest {
    pub vm_id: String,
    pub disk_id: String,
    pub size_gib: u64,
}

#[derive(Debug, Deserialize)]
pub struct VmDiskActionRequest {
    pub vm_id: String,
    pub disk_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ResizeVmDiskRequest {
    pub vm_id: String,
    pub disk_id: String,
    pub size_gib: u64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVmRequest {
    pub vm_id: String,
    pub name: String,
    pub vcpu_count: u16,
    pub memory_mib: u64,
}

#[derive(Debug, Deserialize)]
pub struct ConfigureInstallerMediaRequest {
    pub vm_id: String,
    pub source_path: String,
}

#[derive(Debug, Deserialize)]
pub struct InstallerMediaDownloadRequest {
    pub guest_template_id: String,
    pub media_id: String,
}

#[derive(Debug, Deserialize)]
pub struct AttachDownloadedInstallerMediaRequest {
    pub vm_id: String,
    pub guest_template_id: String,
    pub media_id: String,
}


#[derive(Debug, Deserialize)]
pub struct AttachDefaultNetworkRequest {
    pub vm_id: String,
    pub network_id: String,
}

#[derive(Debug, Deserialize)]
pub struct AttachNetworkProfileRequest {
    pub vm_id: String,
    pub network_id: String,
    pub profile: NetworkProfileDto,
    pub bridge_name: Option<String>,
    pub tap_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PublishVmServiceRequest {
    pub vm_id: String,
    pub network_id: String,
    pub protocol: NetworkProtocolDto,
    pub host_port: u16,
    pub guest_port: u16,
}

#[derive(Debug, Deserialize)]
pub struct UnpublishVmServiceRequest {
    pub vm_id: String,
    pub network_id: String,
    pub protocol: NetworkProtocolDto,
    pub host_port: u16,
}

#[derive(Debug, Deserialize)]
pub struct DetachVmNetworkRequest {
    pub vm_id: String,
    pub network_id: String,
}

#[derive(Debug, Deserialize)]
pub struct PrepareVmSshAccessRequest {
    pub vm_id: String,
    pub network_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ConnectionTargetRequest {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct SshConnectionRequest {
    pub username: String,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct CreateSnapshotRequest {
    pub vm_id: String,
    pub snapshot_id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct SnapshotActionRequest {
    pub vm_id: String,
    pub snapshot_id: String,
}

#[derive(Debug, Deserialize)]
pub struct CloneVmRequest {
    pub source_vm_id: String,
    pub target_vm_id: String,
    pub target_name: String,
    pub mode: CloneModeDto,
}


#[derive(Debug, Deserialize)]
pub struct DefineAndroidImageRequest {
    pub image_id: String,
    pub name: String,
    pub requested_release: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SaveDownloadSettingsRequest {
    pub installer_media_path: String,
    pub android_images_path: String,
    pub artifact_cache_path: String,
    pub android_ci_base_url: String,
    pub official_fallback: bool,
}

#[derive(Debug, Deserialize)]
pub struct AndroidImageActionRequest {
    pub image_id: String,
}

#[derive(Debug, Deserialize)]
pub struct AssignAndroidImageRequest {
    pub vm_id: String,
    pub image_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfigureAndroidRequest {
    pub vm_id: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub density_dpi: Option<u32>,
    pub target_fps: Option<u16>,
}

#[derive(Debug, Deserialize)]
pub struct AndroidApkRequest {
    pub vm_id: String,
    pub relative_apk_path: String,
}

#[derive(Debug, Deserialize)]
pub struct AndroidPackageActionRequest {
    pub vm_id: String,
    pub package_name: String,
}

#[derive(Debug, Deserialize)]
pub struct AndroidInputRequest {
    pub vm_id: String,
    pub input: AndroidInputDto,
}


#[derive(Debug, Deserialize)]
pub struct GamingInputProfileRequest {
    pub profile: GamingInputProfileConfigDto,
}

#[derive(Debug, Deserialize)]
pub struct GameProfileRequest {
    pub vm_id: String,
    pub game_id: String,
}

#[tauri::command]
pub fn list_hosts(state: State<'_, DesktopAppState>) -> Result<Vec<HostProfileSummary>, String> {
    Ok(state
        .service
        .lock()
        .map_err(|_| String::from("Desktop service lock poisoned"))?
        .list_hosts())
}

#[tauri::command]
pub fn select_host(
    host_id: String,
    state: State<'_, DesktopAppState>,
) -> Result<DashboardDto, String> {
    state
        .service
        .lock()
        .map_err(|_| String::from("Desktop service lock poisoned"))?
        .select_host(host_id)
}

#[tauri::command]
pub fn get_dashboard(state: State<'_, DesktopAppState>) -> Result<DashboardDto, String> {
    state
        .service
        .lock()
        .map_err(|_| String::from("Desktop service lock poisoned"))?
        .dashboard()
}

#[tauri::command]
pub fn get_artifact_cache_overview(
    state: State<'_, DesktopAppState>,
) -> Result<ArtifactCacheOverviewDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .artifact_cache_overview()
}

#[tauri::command]
pub fn list_artifact_cache_entries(
    state: State<'_, DesktopAppState>,
) -> Result<Vec<ArtifactCacheEntryDto>, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .list_artifact_cache_entries()
}

#[tauri::command]
pub fn set_artifact_cache_pinned(
    request: ArtifactCachePinRequest,
    state: State<'_, DesktopAppState>,
) -> Result<(), String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .set_artifact_cache_pinned(request.source_key, request.pinned)
}

#[tauri::command]
pub fn remove_artifact_cache_entry(
    request: ArtifactCacheSourceRequest,
    state: State<'_, DesktopAppState>,
) -> Result<(), String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .remove_artifact_cache_entry(request.source_key)
}

#[tauri::command]
pub fn verify_artifact_cache(
    state: State<'_, DesktopAppState>,
) -> Result<ArtifactCacheVerificationDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .verify_artifact_cache()
}

#[tauri::command]
pub fn cleanup_artifact_cache(
    state: State<'_, DesktopAppState>,
) -> Result<ArtifactCacheOverviewDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .cleanup_artifact_cache()
}

#[tauri::command]
pub fn revalidate_artifact_cache(
    request: ArtifactCacheSourceRequest,
    state: State<'_, DesktopAppState>,
) -> Result<ArtifactCacheRevalidationDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .revalidate_artifact_cache(request.source_key)
}

#[tauri::command]
pub fn revalidate_all_artifact_cache(
    state: State<'_, DesktopAppState>,
) -> Result<ArtifactCacheRevalidationSummaryDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .revalidate_all_artifact_cache()
}

#[tauri::command]
pub fn fetch_mutable_artifact_cache(
    request: ArtifactCacheFetchRequest,
    state: State<'_, DesktopAppState>,
) -> Result<ArtifactCacheEntryDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .fetch_mutable_artifact_cache(request.source_key, request.source_url, request.pinned)
}

#[tauri::command]
pub fn get_gpu_capabilities(
    state: State<'_, DesktopAppState>,
) -> Result<GpuCapabilitiesDto, String> {
    state
        .service
        .lock()
        .map_err(|_| String::from("Desktop service lock poisoned"))?
        .gpu_capabilities()
}

#[tauri::command]
pub fn get_storage_overview(
    state: State<'_, DesktopAppState>,
) -> Result<StorageOverviewDto, String> {
    state
        .service
        .lock()
        .map_err(|_| String::from("Desktop service lock poisoned"))?
        .storage_overview()
}

#[tauri::command]
pub fn create_vm_disk(
    request: CreateVmDiskRequest,
    state: State<'_, DesktopAppState>,
) -> Result<VmSummaryDto, String> {
    state
        .service
        .lock()
        .map_err(|_| String::from("Desktop service lock poisoned"))?
        .create_vm_disk(request.vm_id, request.disk_id, request.size_gib)
}

#[tauri::command]
pub fn resize_vm_disk(request: ResizeVmDiskRequest, state: State<'_, DesktopAppState>) -> Result<VmSummaryDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .resize_vm_disk(request.vm_id, request.disk_id, request.size_gib)
}

#[tauri::command]
pub fn delete_vm_disk(request: VmDiskActionRequest, state: State<'_, DesktopAppState>) -> Result<VmSummaryDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .delete_vm_disk(request.vm_id, request.disk_id)
}

#[tauri::command]
pub fn update_vm(request: UpdateVmRequest, state: State<'_, DesktopAppState>) -> Result<VmSummaryDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .update_vm(request.vm_id, request.name, request.vcpu_count, request.memory_mib)
}

#[tauri::command]
pub fn delete_vm(vm_id: String, state: State<'_, DesktopAppState>) -> Result<(), String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?.delete_vm(vm_id)
}

#[tauri::command]
pub fn configure_installer_media(request: ConfigureInstallerMediaRequest, state: State<'_, DesktopAppState>) -> Result<VmSummaryDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .configure_installer_media(request.vm_id, request.source_path)
}

#[tauri::command]
pub fn eject_installer_media(vm_id: String, state: State<'_, DesktopAppState>) -> Result<VmSummaryDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .eject_installer_media(vm_id)
}

#[tauri::command]
pub fn start_installer_media_download(
    request: InstallerMediaDownloadRequest,
    state: State<'_, DesktopAppState>,
) -> Result<InstallerMediaDownloadDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .start_installer_media_download(request.guest_template_id, request.media_id)
}

#[tauri::command]
pub fn get_installer_media_download(
    request: InstallerMediaDownloadRequest,
    state: State<'_, DesktopAppState>,
) -> Result<InstallerMediaDownloadDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .get_installer_media_download(request.guest_template_id, request.media_id)
}

#[tauri::command]
pub fn cancel_installer_media_download(
    request: InstallerMediaDownloadRequest,
    state: State<'_, DesktopAppState>,
) -> Result<InstallerMediaDownloadDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .cancel_installer_media_download(request.guest_template_id, request.media_id)
}

#[tauri::command]
pub fn attach_downloaded_installer_media(
    request: AttachDownloadedInstallerMediaRequest,
    state: State<'_, DesktopAppState>,
) -> Result<VmSummaryDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .attach_downloaded_installer_media(request.vm_id, request.guest_template_id, request.media_id)
}

#[tauri::command]
pub fn open_external_url(url: String) -> Result<(), String> {
    DesktopService::open_external_url(url)
}

#[tauri::command]
pub fn pick_installer_iso() -> Result<Option<String>, String> {
    DesktopService::pick_installer_iso()
}

#[tauri::command]
pub fn list_local_logs(state: State<'_, DesktopAppState>) -> Result<Vec<LocalLogEntrySummary>, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?.list_local_logs()
}

#[tauri::command]
pub fn open_local_log(relative_path: String, state: State<'_, DesktopAppState>) -> Result<(), String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?.open_local_log(relative_path)
}

#[tauri::command]
pub fn get_network_overview(
    state: State<'_, DesktopAppState>,
) -> Result<NetworkOverviewDto, String> {
    state
        .service
        .lock()
        .map_err(|_| String::from("Desktop service lock poisoned"))?
        .network_overview()
}

#[tauri::command]
pub fn attach_default_network(
    request: AttachDefaultNetworkRequest,
    state: State<'_, DesktopAppState>,
) -> Result<VmSummaryDto, String> {
    state
        .service
        .lock()
        .map_err(|_| String::from("Desktop service lock poisoned"))?
        .attach_default_network(request.vm_id, request.network_id)
}

#[tauri::command]
pub fn attach_network_profile(
    request: AttachNetworkProfileRequest,
    state: State<'_, DesktopAppState>,
) -> Result<VmSummaryDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .attach_network_profile(request.vm_id, request.network_id, request.profile, request.bridge_name, request.tap_name)
}

#[tauri::command]
pub fn publish_vm_service(
    request: PublishVmServiceRequest,
    state: State<'_, DesktopAppState>,
) -> Result<VmSummaryDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .publish_vm_service(request.vm_id, request.network_id, request.protocol, request.host_port, request.guest_port)
}

#[tauri::command]
pub fn unpublish_vm_service(
    request: UnpublishVmServiceRequest,
    state: State<'_, DesktopAppState>,
) -> Result<VmSummaryDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .unpublish_vm_service(request.vm_id, request.network_id, request.protocol, request.host_port)
}

#[tauri::command]
pub fn prepare_vm_ssh_access(
    request: PrepareVmSshAccessRequest,
    state: State<'_, DesktopAppState>,
) -> Result<VmSummaryDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .prepare_vm_ssh_access(request.vm_id, request.network_id)
}

#[tauri::command]
pub fn prepare_vm_rdp_access(
    request: PrepareVmSshAccessRequest,
    state: State<'_, DesktopAppState>,
) -> Result<VmSummaryDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .prepare_vm_rdp_access(request.vm_id, request.network_id)
}

#[tauri::command]
pub fn test_tcp_connection(request: ConnectionTargetRequest) -> Result<ConnectionProbeSummary, String> {
    DesktopService::test_tcp_connection(request.host, request.port)
}

#[tauri::command]
pub fn open_ssh_connection(request: SshConnectionRequest) -> Result<(), String> {
    DesktopService::open_ssh_connection(request.username, request.host, request.port)
}

#[tauri::command]
pub fn open_rdp_connection(request: ConnectionTargetRequest) -> Result<(), String> {
    DesktopService::open_rdp_connection(request.host, request.port)
}

#[tauri::command]
pub fn detach_vm_network(
    request: DetachVmNetworkRequest,
    state: State<'_, DesktopAppState>,
) -> Result<VmSummaryDto, String> {
    state
        .service
        .lock()
        .map_err(|_| String::from("Desktop service lock poisoned"))?
        .detach_vm_network(request.vm_id, request.network_id)
}

#[tauri::command]
pub fn list_vms(state: State<'_, DesktopAppState>) -> Result<Vec<VmSummaryDto>, String> {
    state
        .service
        .lock()
        .map_err(|_| String::from("Desktop service lock poisoned"))?
        .list_vms()
}

#[tauri::command]
pub fn create_vm(
    request: CreateVmRequest,
    state: State<'_, DesktopAppState>,
) -> Result<VmSummaryDto, String> {
    state
        .service
        .lock()
        .map_err(|_| String::from("Desktop service lock poisoned"))?
        .create_vm(
            request.vm_id,
            request.name,
            request.vcpu_count,
            request.memory_mib,
            request.guest_profile,
            request.guest_template_id,
        )
}

#[tauri::command]
pub fn start_vm(vm_id: String, state: State<'_, DesktopAppState>) -> Result<VmSummaryDto, String> {
    state
        .service
        .lock()
        .map_err(|_| String::from("Desktop service lock poisoned"))?
        .start_vm(vm_id)
}

#[tauri::command]
pub fn stop_vm(vm_id: String, state: State<'_, DesktopAppState>) -> Result<VmSummaryDto, String> {
    state
        .service
        .lock()
        .map_err(|_| String::from("Desktop service lock poisoned"))?
        .stop_vm(vm_id)
}


#[tauri::command]
pub fn open_display(
    vm_id: String,
    state: State<'_, DesktopAppState>,
) -> Result<DisplaySessionDto, String> {
    state
        .service
        .lock()
        .map_err(|_| String::from("Desktop service lock poisoned"))?
        .open_display(vm_id)
}

#[tauri::command]
pub fn create_snapshot(
    request: CreateSnapshotRequest,
    state: State<'_, DesktopAppState>,
) -> Result<SnapshotDto, String> {
    state
        .service
        .lock()
        .map_err(|_| String::from("Desktop service lock poisoned"))?
        .create_snapshot(request.vm_id, request.snapshot_id, request.name)
}

#[tauri::command]
pub fn list_snapshots(
    vm_id: String,
    state: State<'_, DesktopAppState>,
) -> Result<Vec<SnapshotDto>, String> {
    state
        .service
        .lock()
        .map_err(|_| String::from("Desktop service lock poisoned"))?
        .list_snapshots(vm_id)
}

#[tauri::command]
pub fn restore_snapshot(
    request: SnapshotActionRequest,
    state: State<'_, DesktopAppState>,
) -> Result<SnapshotDto, String> {
    state
        .service
        .lock()
        .map_err(|_| String::from("Desktop service lock poisoned"))?
        .restore_snapshot(request.vm_id, request.snapshot_id)
}

#[tauri::command]
pub fn delete_snapshot(
    request: SnapshotActionRequest,
    state: State<'_, DesktopAppState>,
) -> Result<(), String> {
    state
        .service
        .lock()
        .map_err(|_| String::from("Desktop service lock poisoned"))?
        .delete_snapshot(request.vm_id, request.snapshot_id)
}

#[tauri::command]
pub fn clone_vm(
    request: CloneVmRequest,
    state: State<'_, DesktopAppState>,
) -> Result<VmSummaryDto, String> {
    state
        .service
        .lock()
        .map_err(|_| String::from("Desktop service lock poisoned"))?
        .clone_vm(
            request.source_vm_id,
            request.target_vm_id,
            request.target_name,
            request.mode,
        )
}


#[tauri::command]
pub fn get_download_settings(
    state: State<'_, DesktopAppState>,
) -> Result<DownloadSettingsSummary, String> {
    state
        .service
        .lock()
        .map_err(|_| String::from("Desktop service lock poisoned"))?
        .get_download_settings()
}

#[tauri::command]
pub fn save_download_settings(
    request: SaveDownloadSettingsRequest,
    state: State<'_, DesktopAppState>,
) -> Result<DownloadSettingsSummary, String> {
    state
        .service
        .lock()
        .map_err(|_| String::from("Desktop service lock poisoned"))?
        .save_download_settings(
            request.installer_media_path,
            request.android_images_path,
            request.artifact_cache_path,
            request.android_ci_base_url,
            request.official_fallback,
        )
}

#[tauri::command]
pub fn list_android_images(
    state: State<'_, DesktopAppState>,
) -> Result<Vec<AndroidImageDto>, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .list_android_images()
}

#[tauri::command]
pub fn define_android_image(
    request: DefineAndroidImageRequest,
    state: State<'_, DesktopAppState>,
) -> Result<AndroidImageDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .define_android_image(request.image_id, request.name, request.requested_release)
}

#[tauri::command]
pub fn prepare_android_image_build(
    request: AndroidImageActionRequest,
    state: State<'_, DesktopAppState>,
) -> Result<AndroidImageBuildPlanDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .prepare_android_image_build(request.image_id)
}

#[tauri::command]
pub fn register_android_image_build(
    request: AndroidImageActionRequest,
    state: State<'_, DesktopAppState>,
) -> Result<AndroidImageDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .register_android_image_build(request.image_id)
}

#[tauri::command]
pub fn install_android_image_distribution(
    request: AndroidImageActionRequest,
    state: State<'_, DesktopAppState>,
) -> Result<AndroidImageDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .install_android_image_distribution(request.image_id)
}

#[tauri::command]
pub fn cancel_android_image_distribution(
    request: AndroidImageActionRequest,
    state: State<'_, DesktopAppState>,
) -> Result<AndroidImageDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .cancel_android_image_distribution(request.image_id)
}

#[tauri::command]
pub fn cleanup_android_image_distribution(
    request: AndroidImageActionRequest,
    state: State<'_, DesktopAppState>,
) -> Result<AndroidImageDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .cleanup_android_image_distribution(request.image_id)
}

#[tauri::command]
pub fn open_android_image_install_log(
    log_path: String,
    state: State<'_, DesktopAppState>,
) -> Result<(), String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .open_android_image_install_log(log_path)
}

#[tauri::command]
pub fn assign_android_image(
    request: AssignAndroidImageRequest,
    state: State<'_, DesktopAppState>,
) -> Result<AndroidImageAssignmentDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .assign_android_image(request.vm_id, request.image_id)
}

#[tauri::command]
pub fn get_android_image_assignment(
    vm_id: String,
    state: State<'_, DesktopAppState>,
) -> Result<Option<AndroidImageAssignmentDto>, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .get_android_image_assignment(vm_id)
}

#[tauri::command]
pub fn configure_android_runtime(
    request: ConfigureAndroidRequest,
    state: State<'_, DesktopAppState>,
) -> Result<AndroidRuntimeProfileDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .configure_android_runtime(request.vm_id, request.width, request.height, request.density_dpi, request.target_fps)
}

#[tauri::command]
pub fn get_android_profile(vm_id: String, state: State<'_, DesktopAppState>) -> Result<AndroidRuntimeProfileDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?.get_android_profile(vm_id)
}

#[tauri::command]
pub fn get_android_status(vm_id: String, state: State<'_, DesktopAppState>) -> Result<AndroidDeviceStatusDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?.get_android_status(vm_id)
}

#[tauri::command]
pub fn wait_android_ready(vm_id: String, state: State<'_, DesktopAppState>) -> Result<AndroidDeviceStatusDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?.wait_android_ready(vm_id)
}

#[tauri::command]
pub fn apply_android_display(vm_id: String, state: State<'_, DesktopAppState>) -> Result<(), String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?.apply_android_display(vm_id)
}

#[tauri::command]
pub fn list_android_packages(vm_id: String, state: State<'_, DesktopAppState>) -> Result<Vec<AndroidPackageDto>, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?.list_android_packages(vm_id)
}

#[tauri::command]
pub fn install_android_apk(request: AndroidApkRequest, state: State<'_, DesktopAppState>) -> Result<(), String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?.install_android_apk(request.vm_id, request.relative_apk_path)
}

#[tauri::command]
pub fn uninstall_android_package(request: AndroidPackageActionRequest, state: State<'_, DesktopAppState>) -> Result<(), String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?.uninstall_android_package(request.vm_id, request.package_name)
}

#[tauri::command]
pub fn launch_android_package(request: AndroidPackageActionRequest, state: State<'_, DesktopAppState>) -> Result<(), String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?.launch_android_package(request.vm_id, request.package_name)
}

#[tauri::command]
pub fn stop_android_package(request: AndroidPackageActionRequest, state: State<'_, DesktopAppState>) -> Result<(), String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?.stop_android_package(request.vm_id, request.package_name)
}

#[tauri::command]
pub fn inject_android_input(request: AndroidInputRequest, state: State<'_, DesktopAppState>) -> Result<(), String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?.inject_android_input(request.vm_id, request.input)
}

#[tauri::command]
pub fn get_gaming_input_capabilities(
    vm_id: String,
    state: State<'_, DesktopAppState>,
) -> Result<GamingInputCapabilitiesDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .get_gaming_input_capabilities(vm_id)
}

#[tauri::command]
pub fn get_gaming_input_profile(
    vm_id: String,
    state: State<'_, DesktopAppState>,
) -> Result<GamingInputProfileDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .get_gaming_input_profile(vm_id)
}

#[tauri::command]
pub fn configure_gaming_input_profile(
    request: GamingInputProfileRequest,
    state: State<'_, DesktopAppState>,
) -> Result<GamingInputProfileDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .configure_gaming_input_profile(request.profile)
}

#[tauri::command]
pub fn reset_gaming_input_state(
    vm_id: String,
    state: State<'_, DesktopAppState>,
) -> Result<(), String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .reset_gaming_input_state(vm_id)
}
#[tauri::command]
pub fn get_guest_agent_status(
    vm_id: String,
    state: State<'_, DesktopAppState>,
) -> Result<GuestAgentStatusDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .get_guest_agent_status(vm_id)
}

#[tauri::command]
pub fn list_game_catalog(
    state: State<'_, DesktopAppState>,
) -> Result<Vec<GameCatalogEntryDto>, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .list_game_catalog()
}

#[tauri::command]
pub fn list_guest_catalog(
    state: State<'_, DesktopAppState>,
) -> Result<Vec<GuestTemplateDto>, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .list_guest_catalog()
}

#[tauri::command]
pub fn detect_games(
    vm_id: String,
    state: State<'_, DesktopAppState>,
) -> Result<Vec<GameCatalogEntryDto>, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .detect_games(vm_id)
}

#[tauri::command]
pub fn get_game_compatibility(
    request: GameProfileRequest,
    state: State<'_, DesktopAppState>,
) -> Result<GameCompatibilityDto, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .get_game_compatibility(request.vm_id, request.game_id)
}

#[tauri::command]
pub fn apply_game_profile(
    request: GameProfileRequest,
    state: State<'_, DesktopAppState>,
) -> Result<AppliedGameProfileSummary, String> {
    state.service.lock().map_err(|_| String::from("Desktop service lock poisoned"))?
        .apply_game_profile(request.vm_id, request.game_id)
}
