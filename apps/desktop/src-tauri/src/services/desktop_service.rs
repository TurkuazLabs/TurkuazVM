// # 📄 Dosya Yolu: /turkuazvm/apps/desktop/src-tauri/src/services/desktop_service.rs
// # 📌 Amac: Desktop use-case'lerini secili local/remote Engine host profili uzerinden orkestre eder
// # 📌 Modul - Rust
// # Version: 0.41.2
// # Aciklama: Connection Center erisim hazirligini calisan VM icin guvenli stop -> publish -> restart orkestrasyonu ile yonetir; mevcut Engine API ve host switching davranislarini korur
// # Bagimli Oldugu Katman: Tool

use std::collections::HashMap;
use std::thread;
use std::time::Instant;

use serde::Serialize;
use turkuazvm_engine_api::{
    AndroidDeviceStatusDto, AndroidImageAssignmentDto, AndroidImageBuildPlanDto, AndroidImageDto,
    AndroidInputDto, AndroidPackageDto, AndroidRuntimeProfileDto,
    ArtifactCacheEntryDto, ArtifactCacheOverviewDto, ArtifactCacheRevalidationDto,
    ArtifactCacheRevalidationSummaryDto, ArtifactCacheVerificationDto, CloneModeDto, DashboardDto, DisplaySessionDto, EngineAction, EngineRequest, EngineResponseData,
    InstallerMediaDownloadDto,
    GamingInputCapabilitiesDto, GamingInputProfileConfigDto, GamingInputProfileDto,
    GameCatalogEntryDto, GameCompatibilityDto, GuestAgentStatusDto, GpuCapabilitiesDto,
    GuestProfileDto, GuestTemplateDto, NetworkOverviewDto, NetworkProfileDto, NetworkProtocolDto, SnapshotDto, StorageOverviewDto, VmSummaryDto, ENGINE_API_VERSION,
};

use crate::config::desktop_config::{DesktopConfig, DesktopHostMode, DesktopHostProfile};
use crate::tools::display_process_tool::DisplayProcessTool;
use crate::tools::external_connection_tool::ExternalConnectionTool;
use crate::tools::engine_api_client_tool::{EngineApiClientError, EngineApiClientTool};
use crate::tools::engine_process_tool::{EngineProcessHandle, EngineProcessTool};
use crate::tools::local_file_viewer_tool::LocalFileViewerTool;
use crate::tools::external_url_opener_tool::ExternalUrlOpenerTool;
use crate::tools::local_file_picker_tool::LocalFilePickerTool;
use crate::tools::local_log_catalog_tool::LocalLogCatalogTool;
use crate::tools::download_settings_tool::{DownloadSettingsTool, DownloadSettingsUpdate};

#[derive(Debug, Clone, Serialize)]
pub struct HostProfileSummary {
    pub id: String,
    pub label: String,
    pub mode: String,
    pub endpoint: String,
    pub tls: bool,
    pub authenticated: bool,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LocalLogEntrySummary {
    pub relative_path: String,
    pub category: String,
    pub size_bytes: u64,
    pub modified_at_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppliedGameProfileSummary {
    pub game: GameCatalogEntryDto,
    pub android: AndroidRuntimeProfileDto,
    pub gaming_input: GamingInputProfileDto,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectionProbeSummary {
    pub reachable: bool,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct DownloadSettingsSummary {
    pub installer_media_path: String,
    pub android_images_path: String,
    pub artifact_cache_path: String,
    pub android_ci_base_url: String,
    pub official_fallback: bool,
}

const LOCAL_ENGINE_MODE_REQUIRED: &str = "Otomatik baglanti hazirlama local Engine host profili gerektirir";
const VM_STATE_STOPPED: &str = "stopped";
const VM_STATE_RUNNING: &str = "running";
const NETWORK_MODE_MANAGED_NAT: &str = "managed_nat";
const NETWORK_MODE_USER_NAT: &str = "user_nat";
const SSH_HOST_PORT_START: u16 = 2222;
const SSH_HOST_PORT_END: u16 = 2299;
const SSH_GUEST_PORT: u16 = 22;
const RDP_HOST_PORT_START: u16 = 33890;
const RDP_HOST_PORT_END: u16 = 33950;
const RDP_GUEST_PORT: u16 = 3389;
const NETWORK_PROTOCOL_TCP_NAME: &str = "tcp";

pub struct DesktopService {
    config: DesktopConfig,
    active_host_id: String,
    next_request_id: u64,
    clients: HashMap<String, EngineApiClientTool>,
    engine_processes: HashMap<String, EngineProcessHandle>,
}

impl DesktopService {
    pub fn new(config: DesktopConfig) -> Self {
        Self {
            active_host_id: config.active_host_id.clone(),
            config,
            next_request_id: 1,
            clients: HashMap::new(),
            engine_processes: HashMap::new(),
        }
    }

    pub fn list_hosts(&self) -> Vec<HostProfileSummary> {
        self.config
            .hosts
            .iter()
            .map(|host| HostProfileSummary {
                id: host.id.clone(),
                label: host.label.clone(),
                mode: host_mode_name(host.mode).to_owned(),
                endpoint: host.endpoint.to_string(),
                tls: host.tls.is_some(),
                authenticated: host.auth_token.is_some(),
                active: host.id == self.active_host_id,
            })
            .collect()
    }

    pub fn select_host(&mut self, host_id: String) -> Result<DashboardDto, String> {
        if !self.config.hosts.iter().any(|host| host.id == host_id) {
            return Err(format!("Unknown host profile: {host_id}"));
        }
        self.active_host_id = host_id;
        self.dashboard()
    }

    pub fn dashboard(&mut self) -> Result<DashboardDto, String> {
        match self.execute(EngineAction::Dashboard)? {
            EngineResponseData::Dashboard(value) => Ok(value),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn gpu_capabilities(&mut self) -> Result<GpuCapabilitiesDto, String> {
        match self.execute(EngineAction::GetGpuCapabilities)? {
            EngineResponseData::GpuCapabilities(value) => Ok(value),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn list_vms(&mut self) -> Result<Vec<VmSummaryDto>, String> {
        match self.execute(EngineAction::ListVms)? {
            EngineResponseData::VmList { machines } => Ok(machines),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn storage_overview(&mut self) -> Result<StorageOverviewDto, String> {
        match self.execute(EngineAction::GetStorageOverview)? {
            EngineResponseData::StorageOverview(value) => Ok(value),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn artifact_cache_overview(&mut self) -> Result<ArtifactCacheOverviewDto, String> {
        match self.execute(EngineAction::GetArtifactCacheOverview)? {
            EngineResponseData::ArtifactCacheOverview(value) => Ok(value),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn list_artifact_cache_entries(&mut self) -> Result<Vec<ArtifactCacheEntryDto>, String> {
        match self.execute(EngineAction::ListArtifactCacheEntries)? {
            EngineResponseData::ArtifactCacheList { entries } => Ok(entries),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn set_artifact_cache_pinned(&mut self, source_key: String, pinned: bool) -> Result<(), String> {
        self.expect_ack(EngineAction::SetArtifactCachePinned { source_key, pinned })
    }

    pub fn remove_artifact_cache_entry(&mut self, source_key: String) -> Result<(), String> {
        self.expect_ack(EngineAction::RemoveArtifactCacheEntry { source_key })
    }

    pub fn verify_artifact_cache(&mut self) -> Result<ArtifactCacheVerificationDto, String> {
        match self.execute(EngineAction::VerifyArtifactCache)? {
            EngineResponseData::ArtifactCacheVerification(value) => Ok(value),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn cleanup_artifact_cache(&mut self) -> Result<ArtifactCacheOverviewDto, String> {
        match self.execute(EngineAction::CleanupArtifactCache)? {
            EngineResponseData::ArtifactCacheOverview(value) => Ok(value),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn revalidate_artifact_cache(&mut self, source_key: String) -> Result<ArtifactCacheRevalidationDto, String> {
        match self.execute(EngineAction::RevalidateArtifactCache { source_key })? {
            EngineResponseData::ArtifactCacheRevalidation(value) => Ok(value),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn revalidate_all_artifact_cache(&mut self) -> Result<ArtifactCacheRevalidationSummaryDto, String> {
        match self.execute(EngineAction::RevalidateAllArtifactCache)? {
            EngineResponseData::ArtifactCacheRevalidationSummary(value) => Ok(value),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn fetch_mutable_artifact_cache(
        &mut self, source_key: String, source_url: String, pinned: bool,
    ) -> Result<ArtifactCacheEntryDto, String> {
        match self.execute(EngineAction::FetchMutableArtifactCache { source_key, source_url, pinned })? {
            EngineResponseData::ArtifactCacheEntry(value) => Ok(value),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn create_vm_disk(
        &mut self,
        vm_id: String,
        disk_id: String,
        size_gib: u64,
    ) -> Result<VmSummaryDto, String> {
        match self.execute(EngineAction::CreateVmDisk { vm_id, disk_id, size_gib })? {
            EngineResponseData::Vm { machine } => Ok(machine),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn resize_vm_disk(&mut self, vm_id: String, disk_id: String, size_gib: u64) -> Result<VmSummaryDto, String> {
        match self.execute(EngineAction::ResizeVmDisk { vm_id, disk_id, size_gib })? {
            EngineResponseData::Vm { machine } => Ok(machine),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn delete_vm_disk(&mut self, vm_id: String, disk_id: String) -> Result<VmSummaryDto, String> {
        match self.execute(EngineAction::DeleteVmDisk { vm_id, disk_id })? {
            EngineResponseData::Vm { machine } => Ok(machine),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn update_vm(&mut self, vm_id: String, name: String, vcpu_count: u16, memory_mib: u64) -> Result<VmSummaryDto, String> {
        match self.execute(EngineAction::UpdateVm { vm_id, name, vcpu_count, memory_mib })? {
            EngineResponseData::Vm { machine } => Ok(machine),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn delete_vm(&mut self, vm_id: String) -> Result<(), String> {
        self.expect_ack(EngineAction::DeleteVm { vm_id })
    }

    pub fn configure_installer_media(&mut self, vm_id: String, source_path: String) -> Result<VmSummaryDto, String> {
        match self.execute(EngineAction::ConfigureInstallerMedia { vm_id, source_path })? {
            EngineResponseData::Vm { machine } => Ok(machine),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn eject_installer_media(&mut self, vm_id: String) -> Result<VmSummaryDto, String> {
        match self.execute(EngineAction::EjectInstallerMedia { vm_id })? {
            EngineResponseData::Vm { machine } => Ok(machine),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn start_installer_media_download(&mut self, guest_template_id: String, media_id: String) -> Result<InstallerMediaDownloadDto, String> {
        match self.execute(EngineAction::StartInstallerMediaDownload { guest_template_id, media_id })? {
            EngineResponseData::InstallerMediaDownload(value) => Ok(value),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn get_installer_media_download(&mut self, guest_template_id: String, media_id: String) -> Result<InstallerMediaDownloadDto, String> {
        match self.execute(EngineAction::GetInstallerMediaDownload { guest_template_id, media_id })? {
            EngineResponseData::InstallerMediaDownload(value) => Ok(value),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn cancel_installer_media_download(&mut self, guest_template_id: String, media_id: String) -> Result<InstallerMediaDownloadDto, String> {
        match self.execute(EngineAction::CancelInstallerMediaDownload { guest_template_id, media_id })? {
            EngineResponseData::InstallerMediaDownload(value) => Ok(value),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn attach_downloaded_installer_media(
        &mut self,
        vm_id: String,
        guest_template_id: String,
        media_id: String,
    ) -> Result<VmSummaryDto, String> {
        match self.execute(EngineAction::AttachDownloadedInstallerMedia { vm_id, guest_template_id, media_id })? {
            EngineResponseData::Vm { machine } => Ok(machine),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn open_external_url(url: String) -> Result<(), String> {
        ExternalUrlOpenerTool::open_https(&url)
    }

    pub fn pick_installer_iso() -> Result<Option<String>, String> {
        LocalFilePickerTool::pick_iso()
    }

    pub fn list_local_logs(&self) -> Result<Vec<LocalLogEntrySummary>, String> {
        LocalLogCatalogTool::list(&self.config.project_root).map(|entries| {
            entries.into_iter().map(|entry| LocalLogEntrySummary {
                relative_path: entry.relative_path,
                category: entry.category,
                size_bytes: entry.size_bytes,
                modified_at_unix_ms: entry.modified_at_unix_ms,
            }).collect()
        })
    }

    pub fn open_local_log(&self, relative_path: String) -> Result<(), String> {
        LocalFileViewerTool::open_data_log(&self.config.project_root, &relative_path)
    }

    pub fn network_overview(&mut self) -> Result<NetworkOverviewDto, String> {
        match self.execute(EngineAction::GetNetworkOverview)? {
            EngineResponseData::NetworkOverview(value) => Ok(value),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn attach_default_network(&mut self, vm_id: String, network_id: String) -> Result<VmSummaryDto, String> {
        match self.execute(EngineAction::AttachDefaultNetwork { vm_id, network_id })? {
            EngineResponseData::Vm { machine } => Ok(machine),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn attach_network_profile(
        &mut self,
        vm_id: String,
        network_id: String,
        profile: NetworkProfileDto,
        bridge_name: Option<String>,
        tap_name: Option<String>,
    ) -> Result<VmSummaryDto, String> {
        match self.execute(EngineAction::AttachNetworkProfile { vm_id, network_id, profile, bridge_name, tap_name })? {
            EngineResponseData::Vm { machine } => Ok(machine),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn publish_vm_service(
        &mut self, vm_id: String, network_id: String, protocol: NetworkProtocolDto, host_port: u16, guest_port: u16,
    ) -> Result<VmSummaryDto, String> {
        match self.execute(EngineAction::PublishVmService { vm_id, network_id, protocol, host_port, guest_port })? {
            EngineResponseData::Vm { machine } => Ok(machine),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn unpublish_vm_service(
        &mut self, vm_id: String, network_id: String, protocol: NetworkProtocolDto, host_port: u16,
    ) -> Result<VmSummaryDto, String> {
        match self.execute(EngineAction::UnpublishVmService { vm_id, network_id, protocol, host_port })? {
            EngineResponseData::Vm { machine } => Ok(machine),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn prepare_vm_ssh_access(&mut self, vm_id: String, network_id: String) -> Result<VmSummaryDto, String> {
        self.prepare_vm_tcp_access(
            vm_id,
            network_id,
            "SSH",
            SSH_HOST_PORT_START,
            SSH_HOST_PORT_END,
            SSH_GUEST_PORT,
        )
    }

    pub fn prepare_vm_rdp_access(&mut self, vm_id: String, network_id: String) -> Result<VmSummaryDto, String> {
        self.prepare_vm_tcp_access(
            vm_id,
            network_id,
            "RDP",
            RDP_HOST_PORT_START,
            RDP_HOST_PORT_END,
            RDP_GUEST_PORT,
        )
    }

    fn prepare_vm_tcp_access(
        &mut self,
        vm_id: String,
        network_id: String,
        service_label: &str,
        host_port_start: u16,
        host_port_end: u16,
        guest_port: u16,
    ) -> Result<VmSummaryDto, String> {
        if self.active_profile()?.mode != DesktopHostMode::Local {
            return Err(String::from(LOCAL_ENGINE_MODE_REQUIRED));
        }

        let machine = self.list_vms()?
            .into_iter()
            .find(|machine| machine.id == vm_id)
            .ok_or_else(|| format!("VM bulunamadi: {vm_id}"))?;
        let network = machine.networks.iter()
            .find(|network| network.id == network_id)
            .ok_or_else(|| format!("VM agi bulunamadi: {network_id}"))?;
        if !matches!(network.mode.as_str(), NETWORK_MODE_MANAGED_NAT | NETWORK_MODE_USER_NAT) {
            return Err(format!("Otomatik {service_label} hazirlama yalniz Turkuaz NAT aglarinda kullanilabilir"));
        }
        if network.published_services.iter().any(|service| {
            service.protocol == NETWORK_PROTOCOL_TCP_NAME && service.guest_port == guest_port
        }) {
            return Ok(machine);
        }

        let was_running = match machine.state.as_str() {
            VM_STATE_STOPPED => false,
            VM_STATE_RUNNING => true,
            other => {
                return Err(format!(
                    "{service_label} erisimi VM durumu '{other}' iken hazirlanamaz; VM calisiyor veya kapali olmali"
                ));
            }
        };

        if was_running {
            self.stop_vm(vm_id.clone())
                .map_err(|error| format!("{service_label} erisimi icin VM durdurulamadi: {error}"))?;
        }

        let prepare_result = (|| {
            let host_port = ExternalConnectionTool::find_available_loopback_tcp_port(host_port_start, host_port_end)?;
            self.publish_vm_service(
                vm_id.clone(),
                network_id,
                NetworkProtocolDto::Tcp,
                host_port,
                guest_port,
            )
        })();

        match prepare_result {
            Ok(machine) if !was_running => Ok(machine),
            Ok(_) => self.start_vm(vm_id)
                .map_err(|error| format!("{service_label} host yayini hazirlandi ancak VM yeniden baslatilamadi: {error}")),
            Err(prepare_error) if !was_running => Err(prepare_error),
            Err(prepare_error) => match self.start_vm(vm_id) {
                Ok(_) => Err(format!(
                    "{service_label} host yayini hazirlanamadi; VM onceki calisma durumuna geri getirildi: {prepare_error}"
                )),
                Err(restart_error) => Err(format!(
                    "{service_label} host yayini hazirlanamadi ve VM yeniden baslatilamadi: {prepare_error}; restart: {restart_error}"
                )),
            },
        }
    }

    pub fn test_tcp_connection(host: String, port: u16) -> Result<ConnectionProbeSummary, String> {
        let reachable = ExternalConnectionTool::test_tcp(&host, port)?;
        Ok(ConnectionProbeSummary { reachable, host, port })
    }

    pub fn open_ssh_connection(username: String, host: String, port: u16) -> Result<(), String> {
        ExternalConnectionTool::open_ssh(&username, &host, port)
    }

    pub fn open_rdp_connection(host: String, port: u16) -> Result<(), String> {
        ExternalConnectionTool::open_rdp(&host, port)
    }

    pub fn detach_vm_network(
        &mut self,
        vm_id: String,
        network_id: String,
    ) -> Result<VmSummaryDto, String> {
        match self.execute(EngineAction::DetachVmNetwork { vm_id, network_id })? {
            EngineResponseData::Vm { machine } => Ok(machine),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn create_vm(
        &mut self,
        vm_id: String,
        name: String,
        vcpu_count: u16,
        memory_mib: u64,
        guest_profile: GuestProfileDto,
        guest_template_id: Option<String>,
    ) -> Result<VmSummaryDto, String> {
        match self.execute(EngineAction::CreateVm {
            vm_id,
            name,
            vcpu_count,
            memory_mib,
            guest_profile,
            guest_template_id,
        })? {
            EngineResponseData::Vm { machine } => Ok(machine),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn start_vm(&mut self, vm_id: String) -> Result<VmSummaryDto, String> {
        match self.execute(EngineAction::StartVm { vm_id })? {
            EngineResponseData::Vm { machine } => Ok(machine),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn stop_vm(&mut self, vm_id: String) -> Result<VmSummaryDto, String> {
        match self.execute(EngineAction::StopVm { vm_id })? {
            EngineResponseData::Vm { machine } => Ok(machine),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }


    pub fn open_display(&mut self, vm_id: String) -> Result<DisplaySessionDto, String> {
        let profile = self.active_profile()?.clone();
        if profile.mode != DesktopHostMode::Local {
            return Err(String::from(
                "Remote native display transport is not available",
            ));
        }
        let display = match self.execute(EngineAction::GetDisplaySession {
            vm_id: vm_id.clone(),
        })? {
            EngineResponseData::DisplaySession(value) => value,
            other => return Err(format!("Unexpected Engine response: {other:?}")),
        };
        if !display.local_only {
            return Err(String::from(
                "TurkuazDisplay accepts local-only display sessions",
            ));
        }
        let gaming_input = profile.tls.is_none()
            && matches!(
                self.execute(EngineAction::GetGamingInputProfile { vm_id: vm_id.clone() }),
                Ok(EngineResponseData::GamingInputProfile(_))
            );
        DisplayProcessTool::new(
            self.config.display_executable.clone(),
            self.config.project_root.clone(),
        )
        .spawn(
            &vm_id,
            &display.endpoint,
            "TurkuazVM",
            profile.endpoint,
            profile.auth_token.as_deref(),
            gaming_input,
        )?;
        Ok(display)
    }

    pub fn create_snapshot(
        &mut self,
        vm_id: String,
        snapshot_id: String,
        name: String,
    ) -> Result<SnapshotDto, String> {
        match self.execute(EngineAction::CreateSnapshot {
            vm_id,
            snapshot_id,
            name,
        })? {
            EngineResponseData::Snapshot { snapshot } => Ok(snapshot),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn list_snapshots(&mut self, vm_id: String) -> Result<Vec<SnapshotDto>, String> {
        match self.execute(EngineAction::ListSnapshots { vm_id })? {
            EngineResponseData::SnapshotList { snapshots } => Ok(snapshots),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn restore_snapshot(
        &mut self,
        vm_id: String,
        snapshot_id: String,
    ) -> Result<SnapshotDto, String> {
        match self.execute(EngineAction::RestoreSnapshot { vm_id, snapshot_id })? {
            EngineResponseData::Snapshot { snapshot } => Ok(snapshot),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn delete_snapshot(&mut self, vm_id: String, snapshot_id: String) -> Result<(), String> {
        match self.execute(EngineAction::DeleteSnapshot { vm_id, snapshot_id })? {
            EngineResponseData::Ack => Ok(()),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn clone_vm(
        &mut self,
        source_vm_id: String,
        target_vm_id: String,
        target_name: String,
        mode: CloneModeDto,
    ) -> Result<VmSummaryDto, String> {
        match self.execute(EngineAction::CloneVm {
            source_vm_id,
            target_vm_id,
            target_name,
            mode,
        })? {
            EngineResponseData::Vm { machine } => Ok(machine),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn get_download_settings(&self) -> Result<DownloadSettingsSummary, String> {
        let settings = DownloadSettingsTool::read(&self.config.project_root)?;
        Ok(DownloadSettingsSummary {
            installer_media_path: settings.installer_media_path,
            android_images_path: settings.android_images_path,
            artifact_cache_path: settings.artifact_cache_path,
            android_ci_base_url: settings.android_ci_base_url,
            official_fallback: settings.official_fallback,
        })
    }

    pub fn save_download_settings(
        &self,
        installer_media_path: String,
        android_images_path: String,
        artifact_cache_path: String,
        android_ci_base_url: String,
        official_fallback: bool,
    ) -> Result<DownloadSettingsSummary, String> {
        let settings = DownloadSettingsTool::update(
            &self.config.project_root,
            DownloadSettingsUpdate {
                installer_media_path,
                android_images_path,
                artifact_cache_path,
                android_ci_base_url,
                official_fallback,
            },
        )?;
        Ok(DownloadSettingsSummary {
            installer_media_path: settings.installer_media_path,
            android_images_path: settings.android_images_path,
            artifact_cache_path: settings.artifact_cache_path,
            android_ci_base_url: settings.android_ci_base_url,
            official_fallback: settings.official_fallback,
        })
    }

    pub fn list_android_images(&mut self) -> Result<Vec<AndroidImageDto>, String> {
        match self.execute(EngineAction::ListAndroidImages)? {
            EngineResponseData::AndroidImageList { images } => Ok(images),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn define_android_image(
        &mut self,
        image_id: String,
        name: String,
        requested_release: Option<String>,
    ) -> Result<AndroidImageDto, String> {
        match self.execute(EngineAction::DefineAndroidImage { image_id, name, requested_release })? {
            EngineResponseData::AndroidImage { image } => Ok(image),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn prepare_android_image_build(
        &mut self,
        image_id: String,
    ) -> Result<AndroidImageBuildPlanDto, String> {
        match self.execute(EngineAction::PrepareAndroidImageBuild { image_id })? {
            EngineResponseData::AndroidImageBuildPlan(plan) => Ok(plan),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn register_android_image_build(
        &mut self,
        image_id: String,
    ) -> Result<AndroidImageDto, String> {
        match self.execute(EngineAction::RegisterAndroidImageBuild { image_id })? {
            EngineResponseData::AndroidImage { image } => Ok(image),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn install_android_image_distribution(
        &mut self,
        image_id: String,
    ) -> Result<AndroidImageDto, String> {
        match self.execute(EngineAction::InstallAndroidImageDistribution { image_id })? {
            EngineResponseData::AndroidImage { image } => Ok(image),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn cancel_android_image_distribution(
        &mut self,
        image_id: String,
    ) -> Result<AndroidImageDto, String> {
        match self.execute(EngineAction::CancelAndroidImageDistribution { image_id })? {
            EngineResponseData::AndroidImage { image } => Ok(image),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn cleanup_android_image_distribution(
        &mut self,
        image_id: String,
    ) -> Result<AndroidImageDto, String> {
        match self.execute(EngineAction::CleanupAndroidImageDistribution { image_id })? {
            EngineResponseData::AndroidImage { image } => Ok(image),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn open_android_image_install_log(&self, log_path: String) -> Result<(), String> {
        let profile = self.active_profile()?;
        if profile.mode != DesktopHostMode::Local {
            return Err(String::from("Remote Android install log opening is not available"));
        }
        LocalFileViewerTool::open_install_log(&self.config.project_root, &log_path)
    }

    pub fn assign_android_image(
        &mut self,
        vm_id: String,
        image_id: String,
    ) -> Result<AndroidImageAssignmentDto, String> {
        match self.execute(EngineAction::AssignAndroidImage { vm_id, image_id })? {
            EngineResponseData::AndroidImageAssignment { assignment: Some(assignment) } => Ok(assignment),
            EngineResponseData::AndroidImageAssignment { assignment: None } => {
                Err(String::from("Android image assignment was not returned"))
            }
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn get_android_image_assignment(
        &mut self,
        vm_id: String,
    ) -> Result<Option<AndroidImageAssignmentDto>, String> {
        match self.execute(EngineAction::GetAndroidImageAssignment { vm_id })? {
            EngineResponseData::AndroidImageAssignment { assignment } => Ok(assignment),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn configure_android_runtime(
        &mut self,
        vm_id: String,
        width: Option<u32>,
        height: Option<u32>,
        density_dpi: Option<u32>,
        target_fps: Option<u16>,
    ) -> Result<AndroidRuntimeProfileDto, String> {
        match self.execute(EngineAction::ConfigureAndroidRuntime { vm_id, width, height, density_dpi, target_fps })? {
            EngineResponseData::AndroidProfile(profile) => Ok(profile),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn get_android_profile(&mut self, vm_id: String) -> Result<AndroidRuntimeProfileDto, String> {
        match self.execute(EngineAction::GetAndroidProfile { vm_id })? {
            EngineResponseData::AndroidProfile(profile) => Ok(profile),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn get_android_status(&mut self, vm_id: String) -> Result<AndroidDeviceStatusDto, String> {
        match self.execute(EngineAction::GetAndroidStatus { vm_id })? {
            EngineResponseData::AndroidStatus(status) => Ok(status),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn wait_android_ready(&mut self, vm_id: String) -> Result<AndroidDeviceStatusDto, String> {
        match self.execute(EngineAction::WaitAndroidReady { vm_id })? {
            EngineResponseData::AndroidStatus(status) => Ok(status),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn apply_android_display(&mut self, vm_id: String) -> Result<(), String> {
        self.expect_ack(EngineAction::ApplyAndroidDisplay { vm_id })
    }

    pub fn list_android_packages(&mut self, vm_id: String) -> Result<Vec<AndroidPackageDto>, String> {
        match self.execute(EngineAction::ListAndroidPackages { vm_id })? {
            EngineResponseData::AndroidPackageList { packages } => Ok(packages),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn install_android_apk(&mut self, vm_id: String, relative_apk_path: String) -> Result<(), String> {
        self.expect_ack(EngineAction::InstallAndroidApk { vm_id, relative_apk_path })
    }

    pub fn uninstall_android_package(&mut self, vm_id: String, package_name: String) -> Result<(), String> {
        self.expect_ack(EngineAction::UninstallAndroidPackage { vm_id, package_name })
    }

    pub fn launch_android_package(&mut self, vm_id: String, package_name: String) -> Result<(), String> {
        self.expect_ack(EngineAction::LaunchAndroidPackage { vm_id, package_name })
    }

    pub fn stop_android_package(&mut self, vm_id: String, package_name: String) -> Result<(), String> {
        self.expect_ack(EngineAction::StopAndroidPackage { vm_id, package_name })
    }

    pub fn inject_android_input(&mut self, vm_id: String, input: AndroidInputDto) -> Result<(), String> {
        self.expect_ack(EngineAction::InjectAndroidInput { vm_id, input })
    }

    pub fn get_gaming_input_capabilities(&mut self, vm_id: String) -> Result<GamingInputCapabilitiesDto, String> {
        match self.execute(EngineAction::GetGamingInputCapabilities { vm_id })? {
            EngineResponseData::GamingInputCapabilities(value) => Ok(value),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn get_gaming_input_profile(&mut self, vm_id: String) -> Result<GamingInputProfileDto, String> {
        match self.execute(EngineAction::GetGamingInputProfile { vm_id })? {
            EngineResponseData::GamingInputProfile(value) => Ok(value),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn configure_gaming_input_profile(
        &mut self,
        profile: GamingInputProfileConfigDto,
    ) -> Result<GamingInputProfileDto, String> {
        match self.execute(EngineAction::ConfigureGamingInputProfile { profile })? {
            EngineResponseData::GamingInputProfile(value) => Ok(value),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn reset_gaming_input_state(&mut self, vm_id: String) -> Result<(), String> {
        self.expect_ack(EngineAction::ResetGamingInputState { vm_id })
    }

    pub fn get_guest_agent_status(&mut self, vm_id: String) -> Result<GuestAgentStatusDto, String> {
        match self.execute(EngineAction::GetGuestAgentStatus { vm_id })? {
            EngineResponseData::GuestAgentStatus(value) => Ok(value),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn list_game_catalog(&mut self) -> Result<Vec<GameCatalogEntryDto>, String> {
        match self.execute(EngineAction::ListGameCatalog)? {
            EngineResponseData::GameCatalogList { games } => Ok(games),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn list_guest_catalog(&mut self) -> Result<Vec<GuestTemplateDto>, String> {
        match self.execute(EngineAction::ListGuestCatalog)? {
            EngineResponseData::GuestCatalogList { templates } => Ok(templates),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn detect_games(&mut self, vm_id: String) -> Result<Vec<GameCatalogEntryDto>, String> {
        match self.execute(EngineAction::DetectGames { vm_id })? {
            EngineResponseData::DetectedGames { games } => Ok(games),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn get_game_compatibility(
        &mut self,
        vm_id: String,
        game_id: String,
    ) -> Result<GameCompatibilityDto, String> {
        match self.execute(EngineAction::GetGameCompatibility { vm_id, game_id })? {
            EngineResponseData::GameCompatibility(value) => Ok(value),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    pub fn apply_game_profile(
        &mut self,
        vm_id: String,
        game_id: String,
    ) -> Result<AppliedGameProfileSummary, String> {
        match self.execute(EngineAction::ApplyGameProfile { vm_id, game_id })? {
            EngineResponseData::GameProfileApplied { game, android, gaming_input } => {
                Ok(AppliedGameProfileSummary { game, android, gaming_input })
            }
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    fn expect_ack(&mut self, action: EngineAction) -> Result<(), String> {
        match self.execute(action)? {
            EngineResponseData::Ack => Ok(()),
            other => Err(format!("Unexpected Engine response: {other:?}")),
        }
    }

    fn execute(&mut self, action: EngineAction) -> Result<EngineResponseData, String> {
        let profile = self.active_profile()?.clone();
        let client = self.client_for(&profile)?;
        let request_timeout = request_timeout_for_action(&profile, &action);
        let request = EngineRequest {
            request_id: self.take_request_id(),
            api_version: ENGINE_API_VERSION,
            auth_token: None,
            action,
        };

        match client.send_with_timeout(&request, request_timeout) {
            Ok(response) => Self::extract_response(response),
            Err(EngineApiClientError::Connect(_)) if profile.auto_start_engine => {
                self.ensure_local_engine_started(&profile, &client)?;
                self.send_after_readiness(&client, &request, request_timeout)
            }
            Err(EngineApiClientError::Read(error)) => {
                let process_detail = self.take_engine_exit_detail(&profile.id)?;
                match process_detail {
                    Some(detail) => Err(format!(
                        "Engine API baglantisi request sonrasinda kapandi ve local Engine processi sonlandi. Request tekrar gonderilmedi. Transport={error}. {detail}"
                    )),
                    None => Err(format!(
                        "Engine API response failed after request was sent. Request tekrar gonderilmedi; Engine processi halen calisiyor olabilir. Detail={error}"
                    )),
                }
            },
            Err(EngineApiClientError::Write(error)) => Err(format!(
                "Engine API request transport failed while sending. Request otomatik tekrar edilmedi. Detail={error}"
            )),
            Err(error) => Err(format!("Engine API error: {error}")),
        }
    }

    fn active_profile(&self) -> Result<&DesktopHostProfile, String> {
        self.config
            .hosts
            .iter()
            .find(|host| host.id == self.active_host_id)
            .ok_or_else(|| format!("Active host profile not found: {}", self.active_host_id))
    }

    fn client_for(&mut self, profile: &DesktopHostProfile) -> Result<EngineApiClientTool, String> {
        if let Some(client) = self.clients.get(&profile.id) {
            return Ok(client.clone());
        }
        let client = EngineApiClientTool::new(
            profile.endpoint,
            profile.auth_token.clone(),
            profile.tls.clone(),
        )
        .map_err(|error| format!("Engine API client config error: {error}"))?;
        self.clients.insert(profile.id.clone(), client.clone());
        Ok(client)
    }

    fn spawn_local_engine(&mut self, profile: &DesktopHostProfile) -> Result<(), String> {
        let process_state = self
            .engine_processes
            .get_mut(&profile.id)
            .map(EngineProcessHandle::try_wait)
            .transpose()?;
        match process_state {
            Some(None) => return Ok(()),
            Some(Some(_)) => {
                self.engine_processes.remove(&profile.id);
            }
            None => {}
        }

        let executable = profile
            .engine_executable
            .clone()
            .ok_or_else(|| String::from("Local Engine executable is not configured"))?;
        let process = EngineProcessTool::new(
            executable,
            self.config.config_path.clone(),
            self.config.project_root.clone(),
        )
        .spawn()?;
        self.engine_processes.insert(profile.id.clone(), process);
        Ok(())
    }

    fn ensure_local_engine_started(
        &mut self,
        profile: &DesktopHostProfile,
        client: &EngineApiClientTool,
    ) -> Result<(), String> {
        self.spawn_local_engine(profile)?;
        self.wait_for_engine_ready(profile, client)
    }

    fn wait_for_engine_ready(
        &mut self,
        profile: &DesktopHostProfile,
        client: &EngineApiClientTool,
    ) -> Result<(), String> {
        let started_at = Instant::now();
        loop {
            let ping = EngineRequest {
                request_id: self.take_request_id(),
                api_version: ENGINE_API_VERSION,
                auth_token: None,
                action: EngineAction::Ping,
            };
            match client.send_with_timeout(&ping, profile.request_timeout) {
                Ok(response) if response.ok => return Ok(()),
                Ok(response) => {
                    return Err(format!(
                        "Engine startup failed: readiness ping rejected: {:?}",
                        response.error
                    ));
                }
                Err(EngineApiClientError::Connect(error))
                | Err(EngineApiClientError::Read(error))
                | Err(EngineApiClientError::Write(error)) => {
                    if let Some(detail) = self.take_engine_exit_detail(&profile.id)? {
                        return Err(format!("Engine startup failed: {detail}"));
                    }
                    if started_at.elapsed() >= profile.startup_timeout {
                        return Err(self.engine_not_ready_message(profile, &error));
                    }
                    thread::sleep(profile.startup_poll_interval);
                }
                Err(error) => return Err(format!("Engine startup failed: {error}")),
            }
        }
    }

    fn send_after_readiness(
        &mut self,
        client: &EngineApiClientTool,
        request: &EngineRequest,
        request_timeout: std::time::Duration,
    ) -> Result<EngineResponseData, String> {
        match client.send_with_timeout(request, request_timeout) {
            Ok(response) => Self::extract_response(response),
            Err(EngineApiClientError::Read(error)) => Err(format!(
                "Engine hazir, ancak istek gonderildikten sonra cevap zamaninda alinamadi. Mutating istek guvenlik icin tekrar edilmedi. Detail={error}"
            )),
            Err(EngineApiClientError::Write(error)) => Err(format!(
                "Engine hazir, ancak request yazimi tamamlanamadi. Guvenlik icin otomatik tekrar yapilmadi. Detail={error}"
            )),
            Err(error) => Err(format!("Engine API request failed after readiness: {error}")),
        }
    }

    fn take_engine_exit_detail(&mut self, host_id: &str) -> Result<Option<String>, String> {
        let detail = {
            let Some(process) = self.engine_processes.get_mut(host_id) else {
                return Ok(None);
            };
            let Some(status) = process.try_wait()? else {
                return Ok(None);
            };
            let log_path = process.log_path().display().to_string();
            let tail = process.diagnostic_tail();
            let process_id = process.process_id();
            format!(
                "Engine process API hazir olmadan kapandi. PID={process_id}, Status={status}, Log={log_path}, Tail={tail}"
            )
        };
        self.engine_processes.remove(host_id);
        Ok(Some(detail))
    }

    fn engine_not_ready_message(&self, profile: &DesktopHostProfile, connection_error: &str) -> String {
        if let Some(process) = self.engine_processes.get(&profile.id) {
            return format!(
                "Engine startup failed: API {} hazir olmadi. PID={}, Log={}, Connection={connection_error}",
                profile.endpoint,
                process.process_id(),
                process.log_path().display()
            );
        }
        format!(
            "Engine startup failed: API {} hazir olmadi. Connection={connection_error}",
            profile.endpoint
        )
    }

    fn extract_response(
        response: turkuazvm_engine_api::EngineResponse,
    ) -> Result<EngineResponseData, String> {
        if !response.ok {
            let error = response.error.unwrap_or(turkuazvm_engine_api::EngineApiError {
                code: String::from("unknown_engine_error"),
                message: String::from("Engine request failed without error detail"),
            });
            return Err(format!("{}: {}", error.code, error.message));
        }
        response
            .data
            .ok_or_else(|| String::from("Engine response has no data"))
    }

    fn take_request_id(&mut self) -> u64 {
        let current = self.next_request_id;
        self.next_request_id = self.next_request_id.saturating_add(1);
        current
    }
}


fn request_timeout_for_action(
    profile: &DesktopHostProfile,
    action: &EngineAction,
) -> std::time::Duration {
    if uses_long_request_timeout(action) {
        profile.long_request_timeout
    } else {
        profile.request_timeout
    }
}

fn uses_long_request_timeout(action: &EngineAction) -> bool {
    matches!(
        action,
        EngineAction::CreateVm { .. }
            | EngineAction::CreateVmDisk { .. }
            | EngineAction::ResizeVmDisk { .. }
            | EngineAction::DeleteVmDisk { .. }
            | EngineAction::UpdateVm { .. }
            | EngineAction::DeleteVm { .. }
            | EngineAction::ConfigureInstallerMedia { .. }
            | EngineAction::EjectInstallerMedia { .. }
            | EngineAction::AttachDownloadedInstallerMedia { .. }
            | EngineAction::AttachDefaultNetwork { .. }
            | EngineAction::AttachNetworkProfile { .. }
            | EngineAction::PublishVmService { .. }
            | EngineAction::UnpublishVmService { .. }
            | EngineAction::DetachVmNetwork { .. }
            | EngineAction::StartVm { .. }
            | EngineAction::StopVm { .. }
            | EngineAction::CreateSnapshot { .. }
            | EngineAction::RestoreSnapshot { .. }
            | EngineAction::DeleteSnapshot { .. }
            | EngineAction::CloneVm { .. }
            | EngineAction::WaitAndroidReady { .. }
            | EngineAction::InstallAndroidApk { .. }
            | EngineAction::VerifyArtifactCache
            | EngineAction::CleanupArtifactCache
            | EngineAction::RevalidateArtifactCache { .. }
            | EngineAction::RevalidateAllArtifactCache
            | EngineAction::FetchMutableArtifactCache { .. }
            | EngineAction::PrepareAndroidImageBuild { .. }
            | EngineAction::RegisterAndroidImageBuild { .. }
            | EngineAction::InstallAndroidImageDistribution { .. }
            | EngineAction::CleanupAndroidImageDistribution { .. }
            | EngineAction::AssignAndroidImage { .. }
    )
}

const fn host_mode_name(value: DesktopHostMode) -> &'static str {
    match value {
        DesktopHostMode::Local => "local",
        DesktopHostMode::Remote => "remote",
    }
}
