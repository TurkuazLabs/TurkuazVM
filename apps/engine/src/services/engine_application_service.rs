// # 📄 Dosya Yolu: /turkuazvm/apps/engine/src/services/engine_application_service.rs
// # 📌 Amac: Engine API requestlerini core use-case servislerine orkestre eder
// # 📌 Modul - Rust
// # Version: 0.40.13
// # Aciklama: API v24; installer medya managed-download kabiliyetini Desktop katmanina tasir ve mevcut runtime akislarini orkestre eder
// # Bagimli Oldugu Katman: Repo | Tool

use std::time::{SystemTime, UNIX_EPOCH};

use turkuazvm_artifact_cache::domain::artifact_cache::{
    ArtifactCacheRecord, ArtifactCacheRevalidation, ArtifactCacheRevalidationState,
};
use turkuazvm_android::domain::device::{AndroidConnectionState, AndroidDeviceReport};
use turkuazvm_android::domain::input::AndroidInputAction;
use turkuazvm_android::domain::guest_agent::{AndroidGuestAgentReport, AndroidTouchContact, AndroidTouchPhase};
use turkuazvm_android::domain::runtime_profile::AndroidRuntimeProfile;
use turkuazvm_android_image::domain::artifact::{AndroidImageArtifactRole};
use turkuazvm_android_image::domain::build_profile::AndroidImageArchitecture;
use turkuazvm_android_image::domain::image::{AndroidImage, AndroidImageAssignment, AndroidImageProvisioningState, AndroidImageState};
use turkuazvm_android_image::ports::android_image_distribution_port::{AndroidImageDistributionProgress, AndroidImageDistributionStage};
use turkuazvm_android_image::ports::android_image_builder_port::AndroidImageBuildPlan;
use turkuazvm_core::commands::attach_network_command::{AttachNetworkCommand, ManagedAddressCommand};
use turkuazvm_core::commands::clone_vm_command::CloneVmCommand;
use turkuazvm_core::commands::create_disk_command::CreateDiskCommand;
use turkuazvm_core::commands::delete_disk_command::DeleteDiskCommand;
use turkuazvm_core::commands::resize_disk_command::ResizeDiskCommand;
use turkuazvm_core::commands::update_vm_command::UpdateVmCommand;
use turkuazvm_core::commands::update_network_service_command::{PublishNetworkServiceCommand, UnpublishNetworkServiceCommand};
use turkuazvm_core::commands::delete_vm_command::DeleteVmCommand;
use turkuazvm_core::commands::eject_installer_media_command::EjectInstallerMediaCommand;
use turkuazvm_core::commands::prepare_guest_boot_command::{FirmwarePreference, InstallerIsoCommand, PrepareGuestBootCommand};
use turkuazvm_core::commands::create_snapshot_command::CreateSnapshotCommand;
use turkuazvm_core::commands::create_vm_command::CreateVmCommand;
use turkuazvm_core::commands::delete_snapshot_command::DeleteSnapshotCommand;
use turkuazvm_core::commands::detach_network_command::DetachNetworkCommand;
use turkuazvm_core::commands::restore_snapshot_command::RestoreSnapshotCommand;
use turkuazvm_core::commands::recover_vm_command::RecoverVmCommand;
use turkuazvm_core::commands::start_vm_command::StartVmCommand;
use turkuazvm_core::commands::stop_vm_command::StopVmCommand;
use turkuazvm_core::domain::clone::CloneMode;
use turkuazvm_core::domain::display::{DisplayRuntimeInfo, DisplayTransport};
use turkuazvm_core::domain::disk::{DiskBus, DiskFormat};
use turkuazvm_core::domain::guest_boot::{BootDevice, GuestProfile};
use turkuazvm_core::domain::host::{HostArchitecture, HostPlatform};
use turkuazvm_core::domain::hypervisor::AccelerationBackend;
use turkuazvm_core::domain::network::{NetworkDeviceModel, NetworkMode, PortProtocol};
use turkuazvm_core::domain::snapshot::SnapshotRecord;
use turkuazvm_core::domain::virtual_machine::VirtualMachine;
use turkuazvm_core::domain::vm_state::VmState;
use turkuazvm_core::ports::hypervisor_probe_port::HypervisorProbePort;
use turkuazvm_core::ports::network_port::NetworkPort;
use turkuazvm_core::services::clone_service::CloneService;
use turkuazvm_core::services::host_capability_service::HostCapabilityService;
use turkuazvm_core::services::network_service::NetworkService;
use turkuazvm_core::services::guest_boot_service::GuestBootService;
use turkuazvm_core::services::vm_configuration_service::VmConfigurationService;
use turkuazvm_core::services::runtime_recovery_service::{RuntimeRecoveryPolicy, RuntimeRecoveryService};
use turkuazvm_core::services::storage_host_service::StorageHostService;
use turkuazvm_core::services::storage_service::StorageService;
use turkuazvm_core::services::snapshot_service::SnapshotService;
use turkuazvm_core::services::vm_lifecycle_service::VmLifecycleService;
use turkuazvm_core::services::vm_query_service::VmQueryService;
use turkuazvm_engine_api::{
    AndroidDeviceStatusDto, AndroidImageArchitectureDto, AndroidImageArtifactDto,
    AndroidImageArtifactRoleDto, AndroidImageAssignmentDto, AndroidImageBuildPlanDto, AndroidImageProvisioningStateDto,
    AndroidImageCapabilitiesDto, AndroidImageDto, AndroidImageInstallProgressDto, AndroidImageInstallStageDto,
    AndroidImageStateDto, AndroidInputDto,
    AndroidPackageDto, AndroidRuntimeProfileDto,
    CloneModeDto, DashboardDto, DisplaySessionDto, EngineAction, EngineRequest, EngineResponse,
    EngineResponseData, GamingInputBindingDto, GamingInputCapabilitiesDto, GamingInputEventDto,
    GamingInputProfileConfigDto, GamingInputProfileDto, GamingInputSourceDto, GamingInputTargetDto,
    GamingKeyDto, GamingMouseButtonDto, GpuCapabilitiesDto, GuestProfileDto, MouseLookDto,
    GuestTemplateDto, InstallerMediaDownloadDto, InstallerMediaSourceDto, NetworkOverviewDto, NetworkProfileDto, NetworkProtocolDto, NormalizedPointDto, PublishedNetworkServiceDto, SnapshotDto, StorageOverviewDto, VirtualJoystickDto, VmDiskDto, VmNetworkDto, VmSummaryDto, ENGINE_API_VERSION,
};
use turkuazvm_engine_api::{
    ArtifactCacheEntryDto, ArtifactCacheOverviewDto, ArtifactCacheRevalidationDto,
    ArtifactCacheRevalidationStateDto, ArtifactCacheRevalidationSummaryDto,
    ArtifactCacheVerificationDto,
};
use turkuazvm_engine_api::{
    GameCatalogEntryDto, GameCatalogGpuBackendDto, GameCatalogMaturityDto,
    GameCompatibilityDto, GameCompatibilityStatusDto, GuestAgentStatusDto,
};
use turkuazvm_game_catalog::domain::compatibility::{CompatibilityStatus, GameRuntimeContext};
use turkuazvm_guest_catalog::domain::guest_template::{GuestProfileHint, GuestTemplate};
use turkuazvm_game_catalog::domain::game::{
    CatalogGpuBackend, CatalogInputSource, CatalogInputTarget, CatalogKey, CatalogMaturity,
    CatalogMouseButton, GameDefinition,
};
use turkuazvm_gpu::domain::capability::{
    GamingGpuCapabilityReport, GpuHostPlatform, HostGpuCapabilities, HypervisorGpuCapabilities,
};
use turkuazvm_gpu::domain::profile::{
    GpuBackend, GpuBackendPreference, GamingGpuPolicy, ResolvedGamingGpuProfile,
};
use turkuazvm_gpu::services::gaming_gpu_service::GamingGpuService;
use turkuazvm_gaming_input::domain::capability::GamingInputCapabilities;
use turkuazvm_gaming_input::domain::event::GamingInputEvent;
use turkuazvm_gaming_input::domain::plan::{GamingInputPlan, TouchPhase};
use turkuazvm_gaming_input::domain::profile::{
    GameInputProfile, GamingKey, GamingMouseButton, GamingVmId, InputBinding, InputSource,
    InputTarget, MouseLookProfile, NormalizedPoint, VirtualJoystickProfile,
};
use turkuazvm_platform::tools::native_gpu_probe_tool::NativeGpuProbeTool;
use turkuazvm_platform::tools::native_host_probe_tool::NativeHostProbeTool;
use turkuazvm_platform::tools::native_network_tool::NativeNetworkTool;
use turkuazvm_platform::tools::native_network_subnet_tool::NativeNetworkSubnetTool;
use turkuazvm_storage::tools::host_storage_tool::HostStorageTool;
use turkuazvm_guest::tools::http_download_tool::HttpDownloadSettings;
use turkuazvm_guest::tools::local_guest_media_tool::{LocalGuestMediaSettings, LocalGuestMediaTool};
use turkuazvm_guest::tools::uefi_firmware_tool::{UefiFirmwareSettings, UefiFirmwareTool};
use turkuazvm_qemu::domain::qemu_runtime::QemuGpuRuntimeSettings;
use turkuazvm_qemu::tools::qemu_discovery_tool::QemuDiscoveryTool;
use turkuazvm_qemu::tools::qemu_gpu_probe_tool::QemuGpuProbeTool;
use turkuazvm_repositories::repositories::shared_vm_repository::SharedVmRepository;
use turkuazvm_repositories::repositories::yaml_runtime_recovery_repository::YamlRuntimeRecoveryRepository;
use turkuazvm_repositories::repositories::yaml_vm_repository::{
    YamlVmRepository, YamlVmRepositorySettings,
};

use crate::config::engine_config::{EngineConfig, ManagedNetworkPoolPolicy};
use crate::services::android_application_service::AndroidApplicationService;
use crate::services::artifact_cache_application_service::ArtifactCacheApplicationService;
use crate::services::android_image_application_service::AndroidImageApplicationService;
use crate::services::engine_auth_service::EngineAuthService;
use crate::services::gaming_input_application_service::GamingInputApplicationService;
use crate::services::game_catalog_application_service::GameCatalogApplicationService;
use crate::services::guest_catalog_application_service::GuestCatalogApplicationService;
use crate::services::installer_media_download_application_service::{
    InstallerMediaDownloadApplicationService, InstallerMediaDownloadStatus,
};
use crate::tools::engine_hypervisor_runtime_tool::EngineHypervisorRuntimeTool;
use crate::tools::engine_storage_tool::EngineStorageTool;

type EngineRepository = SharedVmRepository<YamlVmRepository>;
type LifecycleService =
    VmLifecycleService<EngineRepository, EngineHypervisorRuntimeTool, NativeNetworkTool>;
type EngineSnapshotService = SnapshotService<EngineRepository, EngineStorageTool>;
type EngineCloneService = CloneService<EngineRepository, EngineStorageTool>;
type EngineStorageService = StorageService<EngineRepository, EngineStorageTool>;
type EngineStorageHostService = StorageHostService<HostStorageTool>;
type EngineNetworkService = NetworkService<EngineRepository, NativeNetworkTool>;
type EngineGuestBootService = GuestBootService<EngineRepository, LocalGuestMediaTool, UefiFirmwareTool>;
type EngineVmConfigurationService = VmConfigurationService<EngineRepository>;
type EngineRuntimeRecoveryService = RuntimeRecoveryService<YamlRuntimeRecoveryRepository>;

pub struct EngineApplicationService {
    host_service: HostCapabilityService<NativeHostProbeTool, QemuDiscoveryTool>,
    lifecycle_service: LifecycleService,
    query_service: VmQueryService<EngineRepository>,
    snapshot_service: EngineSnapshotService,
    clone_service: EngineCloneService,
    storage_service: EngineStorageService,
    storage_host_service: EngineStorageHostService,
    network_service: EngineNetworkService,
    guest_boot_service: EngineGuestBootService,
    vm_configuration_service: EngineVmConfigurationService,
    runtime_recovery_service: EngineRuntimeRecoveryService,
    default_disk_format: DiskFormat,
    default_disk_bus: DiskBus,
    default_network_id: String,
    private_network_id: String,
    default_network_device_model: NetworkDeviceModel,
    default_network_profile: NetworkMode,
    managed_nat_policy: ManagedNetworkPoolPolicy,
    private_network_policy: ManagedNetworkPoolPolicy,
    installer_media_relative_path: String,
    guest_firmware_preference: FirmwarePreference,
    android_service: AndroidApplicationService,
    android_image_service: AndroidImageApplicationService,
    artifact_cache_service: ArtifactCacheApplicationService,
    gaming_input_service: GamingInputApplicationService,
    game_catalog_service: GameCatalogApplicationService,
    guest_catalog_service: GuestCatalogApplicationService,
    installer_media_download_service: InstallerMediaDownloadApplicationService,
    gpu_capabilities: GpuCapabilitiesDto,
    effective_gpu_backend: GpuBackend,
    preferred_acceleration: AccelerationBackend,
    auth_service: EngineAuthService,
    host_id: String,
    host_label: String,
    transport_security: String,
}

impl EngineApplicationService {
    pub fn new(mut config: EngineConfig) -> Self {
        let host_id = config.host_id.clone();
        let host_label = config.host_label.clone();
        let transport_security = if config.api_tls.is_some() { String::from("tls") } else { String::from("plain_loopback") };
        let auth_service = EngineAuthService::new(config.api_auth_token.clone());
        let host_service = HostCapabilityService::new(NativeHostProbeTool, QemuDiscoveryTool);
        let report = host_service.inspect();
        let installation = QemuDiscoveryTool.probe().ok();
        let gpu_service = GamingGpuService::new(
            NativeGpuProbeTool,
            QemuGpuProbeTool::new(installation.as_ref().map(|value| value.system_binary.clone())),
        );
        let (gpu_report, gpu_profile, gpu_error) = resolve_gpu_profile(&gpu_service, config.gpu_policy);
        config.qemu_runtime.gpu = QemuGpuRuntimeSettings {
            backend: gpu_profile.backend,
            hostmem_mib: gpu_profile.hostmem_mib,
            experimental: gpu_profile.experimental,
        };
        let gpu_capabilities = gpu_capabilities_summary(
            config.gpu_policy,
            &gpu_report,
            gpu_profile,
            gpu_error,
        );
        let disk_binary = installation
            .as_ref()
            .and_then(|value| value.disk_binary.clone());
        let repository = SharedVmRepository::new(YamlVmRepository::new(YamlVmRepositorySettings {
            data_root: config.data_root.clone(),
        }));
        let runtime_recovery_service = RuntimeRecoveryService::new(
            YamlRuntimeRecoveryRepository::new(
                config.data_root.clone(),
                config.runtime_recovery.journal_retention,
            ),
            RuntimeRecoveryPolicy {
                auto_restart: config.runtime_recovery.auto_restart,
                retry_delay_ms: u64::try_from(config.runtime_recovery.retry_delay.as_millis()).unwrap_or(u64::MAX),
                max_attempts: config.runtime_recovery.max_attempts,
            },
        );
        let default_disk_format = config.storage_policy.default_disk_format;
        let default_disk_bus = config.storage_policy.default_disk_bus;
        let default_network_id = config.network_policy.default_network_id.clone();
        let private_network_id = config.network_policy.private_network_id.clone();
        let default_network_device_model = config.network_policy.default_device_model;
        let default_network_profile = config.network_policy.default_profile;
        let mut managed_nat_policy = config.network_policy.managed_nat.clone();
        let mut private_network_policy = config.network_policy.private_network.clone();
        if std::env::consts::OS == "windows" {
            let managed_plan = NativeNetworkSubnetTool::resolve_private_24(
                managed_nat_policy.subnet, managed_nat_policy.gateway, &[],
            );
            managed_nat_policy.subnet = managed_plan.subnet;
            managed_nat_policy.gateway = managed_plan.gateway;
            let private_plan = NativeNetworkSubnetTool::resolve_private_24(
                private_network_policy.subnet, private_network_policy.gateway, &[managed_nat_policy.subnet],
            );
            private_network_policy.subnet = private_plan.subnet;
            private_network_policy.gateway = private_plan.gateway;
        }
        let portable_image_policy = config.storage_policy.portable_image.clone();
        let query_service = VmQueryService::new(repository.clone());
        let storage = EngineStorageTool::new(
            disk_binary.clone(),
            config.data_root.clone(),
            config.image_root.clone(),
        );
        let snapshot_service = SnapshotService::new(repository.clone(), storage.clone());
        let clone_service = CloneService::new(repository.clone(), storage.clone());
        let storage_service = StorageService::new(repository.clone(), storage);
        let default_runtime_format = match default_disk_format {
            DiskFormat::Qcow2 => String::from("qcow2"),
            DiskFormat::Raw => String::from("raw"),
        };
        let storage_host_service = StorageHostService::new(
            HostStorageTool::new(config.image_root.clone()),
            disk_binary.is_some(),
            default_runtime_format,
            config.storage_policy.default_disk_size_gib,
            portable_image_policy,
        );
        let network_service = NetworkService::new(
            repository.clone(),
            NativeNetworkTool::new(config.network.clone()),
        );
        let vm_configuration_service = VmConfigurationService::new(repository.clone());
        let installer_media_relative_path = config.guest.installer_media_relative_path.clone();
        let guest_firmware_preference = if config.guest.uefi_enabled {
            FirmwarePreference::Uefi
        } else {
            FirmwarePreference::Bios
        };
        let guest_boot_service = GuestBootService::new(
            repository.clone(),
            LocalGuestMediaTool::new(LocalGuestMediaSettings {
                data_root: config.data_root.clone(),
                image_root: config.image_root.clone(),
            }),
            UefiFirmwareTool::new(UefiFirmwareSettings {
                data_root: config.data_root.clone(),
                code_source_path: config.guest.uefi_code_source_path.clone(),
                vars_template_source_path: config.guest.uefi_vars_template_source_path.clone(),
                code_relative_path: config.guest.uefi_code_relative_path.clone(),
                vars_relative_path: config.guest.uefi_vars_relative_path.clone(),
            }),
        );
        let android_service = AndroidApplicationService::new(
            config.android.clone(),
            repository.clone(),
            NativeNetworkTool::new(config.network.clone()),
            config.data_root.clone(),
        );
        let artifact_cache_service = ArtifactCacheApplicationService::new(
            config.artifact_cache.clone(),
            config.download_http.curl_binary.clone(),
        );
        let artifact_cache_client = artifact_cache_service.client();
        let android_image_service = AndroidImageApplicationService::new(
            config.android_image.clone(),
            config.download_http.clone(),
            config.artifact_cache.clone(),
            artifact_cache_client,
            repository.clone(),
            config.data_root.clone(),
            disk_binary,
        );
        let gaming_input_service = GamingInputApplicationService::new(
            repository.clone(),
            config.data_root.clone(),
        );
        let game_catalog_service = GameCatalogApplicationService::new(config.game_catalog_path.clone());
        let guest_catalog_service = GuestCatalogApplicationService::new(config.guest_catalog_path.clone());
        let installer_media_download_service = InstallerMediaDownloadApplicationService::new(
            config.guest_catalog_path.clone(),
            config.guest.installer_media_download_root.clone(),
            config.guest.installer_media_source_cache_path.clone(),
            config.guest.installer_media_resolver_policy.clone(),
            HttpDownloadSettings {
                curl_binary: config.download_http.curl_binary.clone(),
                connect_timeout_seconds: config.download_http.connect_timeout_seconds,
                retry_count: config.download_http.retry_count,
                retry_delay_seconds: config.download_http.retry_delay_seconds,
            },
        );
        let hypervisor = EngineHypervisorRuntimeTool::new(installation, config.qemu_runtime);
        let mut network = NativeNetworkTool::new(config.network);
        let _ = network.recover_runtime();
        let lifecycle_service = VmLifecycleService::new(repository, hypervisor, network);

        Self {
            host_service,
            lifecycle_service,
            query_service,
            snapshot_service,
            clone_service,
            storage_service,
            storage_host_service,
            network_service,
            guest_boot_service,
            vm_configuration_service,
            runtime_recovery_service,
            default_disk_format,
            default_disk_bus,
            default_network_id,
            private_network_id,
            default_network_device_model,
            default_network_profile,
            managed_nat_policy,
            private_network_policy,
            installer_media_relative_path,
            guest_firmware_preference,
            android_service,
            android_image_service,
            artifact_cache_service,
            gaming_input_service,
            game_catalog_service,
            guest_catalog_service,
            installer_media_download_service,
            gpu_capabilities,
            effective_gpu_backend: gpu_profile.backend,
            preferred_acceleration: report.preferred_acceleration,
            auth_service,
            host_id,
            host_label,
            transport_security,
        }
    }

    pub fn maintenance_tick(&mut self) -> Result<(), String> {
        let observed_at_unix_ms = now_unix_ms();
        let events = self
            .lifecycle_service
            .maintenance()
            .map_err(|error| format!("runtime maintenance failed: {error:?}"))?;
        self.runtime_recovery_service
            .record_hypervisor_events(&events, observed_at_unix_ms)
            .map_err(|error| format!("runtime event journal failed: {error:?}"))?;

        let requests = self.runtime_recovery_service
            .due_requests(observed_at_unix_ms)
            .map_err(|error| format!("runtime recovery queue read failed: {error:?}"))?;
        for request in requests {
            match self.start_virtual_machine(request.vm_id.as_str().to_owned()) {
                Ok(_) => {
                    self.runtime_recovery_service
                        .record_success(&request, now_unix_ms())
                        .map_err(|error| format!("runtime recovery success journal failed: {error:?}"))?;
                }
                Err((code, message)) => {
                    self.runtime_recovery_service
                        .record_failure(
                            &request,
                            now_unix_ms(),
                            format!("{code}: {message}"),
                        )
                        .map_err(|error| format!("runtime recovery failure journal failed: {error:?}"))?;
                }
            }
        }
        Ok(())
    }

    pub fn handle(&mut self, request: EngineRequest) -> EngineResponse {
        if request.api_version != ENGINE_API_VERSION {
            return EngineResponse::failure(
                request.request_id,
                "api_version_mismatch",
                format!(
                    "Unsupported API version {}. Engine expects {}",
                    request.api_version, ENGINE_API_VERSION
                ),
            );
        }
        if !self.auth_service.authorize(request.auth_token.as_deref()) {
            return EngineResponse::failure(
                request.request_id,
                "authentication_failed",
                "Engine API authentication failed",
            );
        }

        let request_id = request.request_id;
        match self.execute(request.action) {
            Ok(data) => EngineResponse::success(request_id, data),
            Err((code, message)) => EngineResponse::failure(request_id, code, message),
        }
    }

    fn execute(&mut self, action: EngineAction) -> Result<EngineResponseData, (String, String)> {
        match action {
            EngineAction::Ping => Ok(EngineResponseData::Pong),
            EngineAction::Dashboard => self.dashboard().map(EngineResponseData::Dashboard),
            EngineAction::GetGpuCapabilities => Ok(EngineResponseData::GpuCapabilities(
                self.gpu_capabilities.clone(),
            )),
            EngineAction::ListVms => {
                let machines = self
                    .query_service
                    .list()
                    .map_err(debug_error("vm_list_failed"))?
                    .iter()
                    .map(Self::vm_summary)
                    .collect();
                Ok(EngineResponseData::VmList { machines })
            }
            EngineAction::CreateVm {
                vm_id,
                name,
                vcpu_count,
                memory_mib,
                guest_profile,
                guest_template_id,
            } => {
                let resolved_guest_profile = match guest_template_id.as_deref() {
                    Some(template_id) => {
                        let template = self
                            .guest_catalog_service
                            .get(template_id)
                            .map_err(debug_error("guest_template_get_failed"))?;
                        guest_profile_from_hint(template.guest_profile)
                    }
                    None => guest_profile_from_dto(guest_profile),
                };
                let machine = self
                    .lifecycle_service
                    .create_vm(CreateVmCommand::new(
                        vm_id,
                        name,
                        vcpu_count,
                        memory_mib,
                        self.preferred_acceleration,
                        resolved_guest_profile,
                        guest_template_id,
                    ))
                    .map_err(debug_error("vm_create_failed"))?;
                Ok(EngineResponseData::Vm {
                    machine: Self::vm_summary(&machine),
                })
            }
            EngineAction::GetStorageOverview => {
                let report = self
                    .storage_host_service
                    .report()
                    .map_err(debug_error("storage_overview_failed"))?;
                Ok(EngineResponseData::StorageOverview(StorageOverviewDto {
                    data_root: report.data_root,
                    total_bytes: report.total_bytes,
                    available_bytes: report.available_bytes,
                    qemu_img_available: report.qemu_img_available,
                    default_runtime_format: report.default_runtime_format,
                    default_disk_size_gib: report.default_disk_size_gib,
                    portable_image_extension: report.portable_image_extension,
                    portable_container: report.portable_container,
                    private_copy_default: report.private_copy_default,
                }))
            }
            EngineAction::GetArtifactCacheOverview => {
                let stats = self.artifact_cache_service.stats().map_err(debug_error("artifact_cache_overview_failed"))?;
                Ok(EngineResponseData::ArtifactCacheOverview(ArtifactCacheOverviewDto {
                    enabled: self.artifact_cache_service.enabled(),
                    verify_on_hit: self.artifact_cache_service.verify_on_hit(),
                    allow_stale_on_transient_error: self.artifact_cache_service.allow_stale_on_transient_error(),
                    artifact_count: stats.artifact_count,
                    pinned_count: stats.pinned_count,
                    mutable_count: stats.mutable_count,
                    used_bytes: stats.used_bytes,
                    quota_bytes: stats.quota_bytes,
                }))
            }
            EngineAction::ListArtifactCacheEntries => {
                let entries = self.artifact_cache_service.list().map_err(debug_error("artifact_cache_list_failed"))?;
                Ok(EngineResponseData::ArtifactCacheList {
                    entries: entries.iter().map(artifact_cache_entry_summary).collect(),
                })
            }
            EngineAction::SetArtifactCachePinned { source_key, pinned } => {
                let changed = self.artifact_cache_service
                    .set_pinned(&source_key, pinned)
                    .map_err(debug_error("artifact_cache_pin_failed"))?;
                if !changed {
                    return Err((String::from("artifact_cache_not_found"), format!("Artifact cache source not found: {source_key}")));
                }
                Ok(EngineResponseData::Ack)
            }
            EngineAction::RemoveArtifactCacheEntry { source_key } => {
                let removed = self.artifact_cache_service
                    .remove(&source_key)
                    .map_err(debug_error("artifact_cache_remove_failed"))?;
                if !removed {
                    return Err((String::from("artifact_cache_not_found"), format!("Artifact cache source not found: {source_key}")));
                }
                Ok(EngineResponseData::Ack)
            }
            EngineAction::VerifyArtifactCache => {
                let report = self.artifact_cache_service.verify().map_err(debug_error("artifact_cache_verify_failed"))?;
                Ok(EngineResponseData::ArtifactCacheVerification(ArtifactCacheVerificationDto {
                    checked: report.checked,
                    valid: report.valid,
                    invalid: report.invalid,
                }))
            }
            EngineAction::CleanupArtifactCache => {
                let stats = self.artifact_cache_service.cleanup().map_err(debug_error("artifact_cache_cleanup_failed"))?;
                Ok(EngineResponseData::ArtifactCacheOverview(ArtifactCacheOverviewDto {
                    enabled: self.artifact_cache_service.enabled(),
                    verify_on_hit: self.artifact_cache_service.verify_on_hit(),
                    allow_stale_on_transient_error: self.artifact_cache_service.allow_stale_on_transient_error(),
                    artifact_count: stats.artifact_count,
                    pinned_count: stats.pinned_count,
                    mutable_count: stats.mutable_count,
                    used_bytes: stats.used_bytes,
                    quota_bytes: stats.quota_bytes,
                }))
            }
            EngineAction::RevalidateArtifactCache { source_key } => {
                let report = self.artifact_cache_service
                    .revalidate(&source_key)
                    .map_err(debug_error("artifact_cache_revalidate_failed"))?;
                Ok(EngineResponseData::ArtifactCacheRevalidation(artifact_cache_revalidation_summary(&report)))
            }
            EngineAction::RevalidateAllArtifactCache => {
                let report = self.artifact_cache_service.revalidate_all().map_err(debug_error("artifact_cache_revalidate_all_failed"))?;
                Ok(EngineResponseData::ArtifactCacheRevalidationSummary(ArtifactCacheRevalidationSummaryDto {
                    checked: report.checked,
                    immutable: report.immutable,
                    not_modified: report.not_modified,
                    remote_modified: report.remote_modified,
                    failed: report.failed,
                }))
            }
            EngineAction::FetchMutableArtifactCache { source_key, source_url, pinned } => {
                let record = self.artifact_cache_service
                    .fetch_mutable(&source_key, &source_url, pinned)
                    .map_err(debug_error("artifact_cache_mutable_fetch_failed"))?;
                Ok(EngineResponseData::ArtifactCacheEntry(artifact_cache_entry_summary(&record)))
            }

            EngineAction::CreateVmDisk { vm_id, disk_id, size_gib } => {
                const BYTES_PER_GIB: u64 = 1024 * 1024 * 1024;
                let machine = self
                    .query_service
                    .get(vm_id.clone())
                    .map_err(debug_error("vm_disk_query_failed"))?;
                let machine = if machine.state() == VmState::Error {
                    self.lifecycle_service
                        .recover_vm(RecoverVmCommand::new(vm_id.clone()))
                        .map_err(debug_error("vm_disk_recovery_failed"))?
                } else {
                    machine
                };
                let boot_index = if machine.disks().is_empty() { Some(0) } else { None };
                let size_bytes = size_gib
                    .checked_mul(BYTES_PER_GIB)
                    .ok_or_else(|| (String::from("vm_disk_size_invalid"), String::from("Disk size overflow")))?;
                let relative_path = format!("disks/{disk_id}.{}", disk_format_extension(self.default_disk_format));
                let machine = self
                    .storage_service
                    .create_and_attach_disk(CreateDiskCommand::new(
                        vm_id,
                        disk_id,
                        self.default_disk_format,
                        size_bytes,
                        relative_path,
                        self.default_disk_bus,
                        boot_index,
                    ))
                    .map_err(debug_error("vm_disk_create_failed"))?;
                Ok(EngineResponseData::Vm { machine: Self::vm_summary(&machine) })
            }
            EngineAction::ResizeVmDisk { vm_id, disk_id, size_gib } => {
                const BYTES_PER_GIB: u64 = 1024 * 1024 * 1024;
                let size_bytes = size_gib
                    .checked_mul(BYTES_PER_GIB)
                    .ok_or_else(|| (String::from("vm_disk_resize_invalid"), String::from("Disk size overflow")))?;
                let machine = self
                    .storage_service
                    .resize_disk(ResizeDiskCommand::new(vm_id, disk_id, size_bytes))
                    .map_err(debug_error("vm_disk_resize_failed"))?;
                Ok(EngineResponseData::Vm { machine: Self::vm_summary(&machine) })
            }
            EngineAction::DeleteVmDisk { vm_id, disk_id } => {
                let machine = self
                    .storage_service
                    .delete_disk(DeleteDiskCommand::new(vm_id, disk_id))
                    .map_err(debug_error("vm_disk_delete_failed"))?;
                Ok(EngineResponseData::Vm { machine: Self::vm_summary(&machine) })
            }
            EngineAction::UpdateVm { vm_id, name, vcpu_count, memory_mib } => {
                let machine = self
                    .vm_configuration_service
                    .update_vm(UpdateVmCommand::new(vm_id, name, vcpu_count, memory_mib))
                    .map_err(debug_error("vm_update_failed"))?;
                Ok(EngineResponseData::Vm { machine: Self::vm_summary(&machine) })
            }
            EngineAction::DeleteVm { vm_id } => {
                let machine = self
                    .query_service
                    .get(vm_id.clone())
                    .map_err(debug_error("vm_delete_query_failed"))?;
                if machine.state() == VmState::Error {
                    self.lifecycle_service
                        .recover_vm(RecoverVmCommand::new(vm_id.clone()))
                        .map_err(debug_error("vm_delete_recovery_failed"))?;
                }
                self.vm_configuration_service
                    .delete_vm(DeleteVmCommand::new(vm_id.clone()))
                    .map_err(debug_error("vm_delete_failed"))?;
                if let Ok(id) = turkuazvm_core::domain::virtual_machine::VmId::parse(vm_id) {
                    let _ = self.runtime_recovery_service.clear_request(&id);
                }
                Ok(EngineResponseData::Ack)
            }
            EngineAction::ConfigureInstallerMedia { vm_id, source_path } => {
                let machine = self
                    .query_service
                    .get(vm_id.clone())
                    .map_err(debug_error("installer_media_vm_query_failed"))?;
                if machine.guest_boot().profile() == GuestProfile::Android {
                    return Err((
                        String::from("installer_media_not_supported"),
                        String::from("Android VM uses Android Image assignment instead of installer ISO"),
                    ));
                }
                let profile = machine.guest_boot().profile();
                let machine = self
                    .guest_boot_service
                    .prepare_guest_boot(PrepareGuestBootCommand::new(
                        vm_id,
                        profile,
                        self.guest_firmware_preference,
                        vec![BootDevice::Cdrom, BootDevice::Disk],
                        true,
                        Some(InstallerIsoCommand::new(
                            "installer",
                            std::path::PathBuf::from(source_path),
                            self.installer_media_relative_path.clone(),
                        )),
                    ))
                    .map_err(debug_error("installer_media_configure_failed"))?;
                Ok(EngineResponseData::Vm { machine: Self::vm_summary(&machine) })
            }
            EngineAction::EjectInstallerMedia { vm_id } => {
                let machine = self
                    .guest_boot_service
                    .eject_installer_media(EjectInstallerMediaCommand::new(vm_id))
                    .map_err(debug_error("installer_media_eject_failed"))?;
                Ok(EngineResponseData::Vm { machine: Self::vm_summary(&machine) })
            }
            EngineAction::StartInstallerMediaDownload { guest_template_id, media_id } => {
                let status = self
                    .installer_media_download_service
                    .start(&guest_template_id, &media_id)
                    .map_err(|error| (String::from("installer_media_download_start_failed"), format!("{error:?}")))?;
                Ok(EngineResponseData::InstallerMediaDownload(installer_media_download_summary(&status)))
            }
            EngineAction::GetInstallerMediaDownload { guest_template_id, media_id } => {
                let status = self
                    .installer_media_download_service
                    .status(&guest_template_id, &media_id)
                    .map_err(|error| (String::from("installer_media_download_status_failed"), format!("{error:?}")))?;
                Ok(EngineResponseData::InstallerMediaDownload(installer_media_download_summary(&status)))
            }
            EngineAction::CancelInstallerMediaDownload { guest_template_id, media_id } => {
                let status = self
                    .installer_media_download_service
                    .cancel(&guest_template_id, &media_id)
                    .map_err(|error| (String::from("installer_media_download_cancel_failed"), format!("{error:?}")))?;
                Ok(EngineResponseData::InstallerMediaDownload(installer_media_download_summary(&status)))
            }
            EngineAction::AttachDownloadedInstallerMedia { vm_id, guest_template_id, media_id } => {
                let source_path = self
                    .installer_media_download_service
                    .ready_path(&guest_template_id, &media_id)
                    .map_err(|error| (String::from("installer_media_download_not_ready"), format!("{error:?}")))?;
                let machine = self
                    .query_service
                    .get(vm_id.clone())
                    .map_err(debug_error("installer_media_vm_query_failed"))?;
                if machine.guest_boot().profile() == GuestProfile::Android {
                    return Err((
                        String::from("installer_media_not_supported"),
                        String::from("Android VM uses Android Image assignment instead of installer ISO"),
                    ));
                }
                let profile = machine.guest_boot().profile();
                let machine = self
                    .guest_boot_service
                    .prepare_guest_boot(PrepareGuestBootCommand::new(
                        vm_id,
                        profile,
                        self.guest_firmware_preference,
                        vec![BootDevice::Cdrom, BootDevice::Disk],
                        true,
                        Some(InstallerIsoCommand::new(
                            "installer",
                            source_path,
                            self.installer_media_relative_path.clone(),
                        )),
                    ))
                    .map_err(debug_error("installer_media_configure_failed"))?;
                Ok(EngineResponseData::Vm { machine: Self::vm_summary(&machine) })
            }
            EngineAction::GetNetworkOverview => {
                let capabilities = self.network_service.capabilities();
                Ok(EngineResponseData::NetworkOverview(NetworkOverviewDto {
                    managed_nat: capabilities.managed_nat,
                    private_network: capabilities.private_network,
                    user_nat: capabilities.user_nat,
                    bridge: capabilities.bridge,
                    existing_tap: capabilities.existing_tap,
                    managed_tap: capabilities.managed_tap,
                    bridge_helper: capabilities.bridge_helper,
                    default_network_id: self.default_network_id.clone(),
                    default_device_model: network_device_model_name(self.default_network_device_model).to_owned(),
                    default_profile: network_mode_name(self.default_network_profile).to_owned(),
                    managed_nat_subnet: format!("{}/{}", self.managed_nat_policy.subnet, self.managed_nat_policy.prefix_length),
                    managed_nat_gateway: self.managed_nat_policy.gateway.to_string(),
                    managed_nat_pool: format!("{}.{}-{}", subnet_prefix(self.managed_nat_policy.subnet), self.managed_nat_policy.pool_start, self.managed_nat_policy.pool_end),
                    private_subnet: format!("{}/{}", self.private_network_policy.subnet, self.private_network_policy.prefix_length),
                }))
            }
            EngineAction::AttachDefaultNetwork { vm_id, network_id } => {
                let command = self.network_command_for_profile(vm_id, network_id, self.default_network_profile, None, None)?;
                let machine = self.network_service.attach_network(command)
                    .map_err(debug_error("vm_network_attach_failed"))?;
                Ok(EngineResponseData::Vm { machine: Self::vm_summary(&machine) })
            }
            EngineAction::AttachNetworkProfile { vm_id, network_id, profile, bridge_name, tap_name } => {
                let mode = network_profile_mode(profile);
                let command = self.network_command_for_profile(vm_id, network_id, mode, bridge_name, tap_name)?;
                let machine = self.network_service.attach_network(command)
                    .map_err(debug_error("vm_network_attach_failed"))?;
                Ok(EngineResponseData::Vm { machine: Self::vm_summary(&machine) })
            }
            EngineAction::PublishVmService { vm_id, network_id, protocol, host_port, guest_port } => {
                let protocol = network_protocol(protocol);
                let host_ip = std::net::Ipv4Addr::LOCALHOST;
                self.ensure_host_port_available(protocol, host_ip, host_port)?;
                let machine = self.network_service.publish_service(PublishNetworkServiceCommand::new(
                    vm_id,
                    network_id,
                    protocol,
                    Some(host_ip),
                    host_port,
                    guest_port,
                )).map_err(debug_error("vm_network_publish_failed"))?;
                Ok(EngineResponseData::Vm { machine: Self::vm_summary(&machine) })
            }
            EngineAction::UnpublishVmService { vm_id, network_id, protocol, host_port } => {
                let machine = self.network_service.unpublish_service(UnpublishNetworkServiceCommand::new(
                    vm_id, network_id, network_protocol(protocol), host_port,
                )).map_err(debug_error("vm_network_unpublish_failed"))?;
                Ok(EngineResponseData::Vm { machine: Self::vm_summary(&machine) })
            }
            EngineAction::DetachVmNetwork { vm_id, network_id } => {
                let machine = self
                    .network_service
                    .detach_network(DetachNetworkCommand::new(vm_id, network_id))
                    .map_err(debug_error("vm_network_detach_failed"))?;
                Ok(EngineResponseData::Vm { machine: Self::vm_summary(&machine) })
            }
            EngineAction::StartVm { vm_id } => {
                let machine = self.start_virtual_machine(vm_id)?;
                let _ = self.runtime_recovery_service.clear_request(machine.id());
                Ok(EngineResponseData::Vm {
                    machine: Self::vm_summary(&machine),
                })
            }
            EngineAction::StopVm { vm_id } => {
                if let Ok(id) = turkuazvm_core::domain::virtual_machine::VmId::parse(vm_id.clone()) {
                    let _ = self.runtime_recovery_service.clear_request(&id);
                }
                let machine = self
                    .lifecycle_service
                    .stop_vm(StopVmCommand::new(vm_id))
                    .map_err(debug_error("vm_stop_failed"))?;
                Ok(EngineResponseData::Vm {
                    machine: Self::vm_summary(&machine),
                })
            }
            EngineAction::GetDisplaySession { vm_id } => {
                let runtime = self
                    .lifecycle_service
                    .runtime_info(vm_id.clone())
                    .map_err(debug_error("display_session_failed"))?;
                let display = runtime.display.ok_or_else(|| {
                    (
                        String::from("display_unavailable"),
                        String::from("VM runtime has no native display session"),
                    )
                })?;
                Ok(EngineResponseData::DisplaySession(Self::display_summary(
                    vm_id, display,
                )))
            }
            EngineAction::CreateSnapshot {
                vm_id,
                snapshot_id,
                name,
            } => {
                let snapshot = self
                    .snapshot_service
                    .create_snapshot(CreateSnapshotCommand::new(
                        vm_id,
                        snapshot_id,
                        name,
                        now_unix_ms(),
                    ))
                    .map_err(debug_error("snapshot_create_failed"))?;
                Ok(EngineResponseData::Snapshot {
                    snapshot: Self::snapshot_summary(&snapshot),
                })
            }
            EngineAction::ListSnapshots { vm_id } => {
                let snapshots = self
                    .snapshot_service
                    .list_snapshots(&vm_id)
                    .map_err(debug_error("snapshot_list_failed"))?
                    .iter()
                    .map(Self::snapshot_summary)
                    .collect();
                Ok(EngineResponseData::SnapshotList { snapshots })
            }
            EngineAction::RestoreSnapshot { vm_id, snapshot_id } => {
                let snapshot = self
                    .snapshot_service
                    .restore_snapshot(RestoreSnapshotCommand::new(vm_id, snapshot_id))
                    .map_err(debug_error("snapshot_restore_failed"))?;
                Ok(EngineResponseData::Snapshot {
                    snapshot: Self::snapshot_summary(&snapshot),
                })
            }
            EngineAction::DeleteSnapshot { vm_id, snapshot_id } => {
                self.snapshot_service
                    .delete_snapshot(DeleteSnapshotCommand::new(vm_id, snapshot_id))
                    .map_err(debug_error("snapshot_delete_failed"))?;
                Ok(EngineResponseData::Ack)
            }
            EngineAction::CloneVm {
                source_vm_id,
                target_vm_id,
                target_name,
                mode,
            } => {
                let machine = self
                    .clone_service
                    .clone_vm(CloneVmCommand::new(
                        source_vm_id,
                        target_vm_id,
                        target_name,
                        clone_mode(mode),
                    ))
                    .map_err(debug_error("vm_clone_failed"))?;
                Ok(EngineResponseData::Vm {
                    machine: Self::vm_summary(&machine),
                })
            }
            EngineAction::ConfigureAndroidRuntime {
                vm_id,
                width,
                height,
                density_dpi,
                target_fps,
            } => {
                let profile = self
                    .android_service
                    .configure_runtime(vm_id, width, height, density_dpi, target_fps)
                    .map_err(debug_error("android_configure_failed"))?;
                Ok(EngineResponseData::AndroidProfile(Self::android_profile_summary(&profile)))
            }
            EngineAction::GetAndroidProfile { vm_id } => {
                let profile = self
                    .android_service
                    .profile(&vm_id)
                    .map_err(debug_error("android_profile_failed"))?;
                Ok(EngineResponseData::AndroidProfile(Self::android_profile_summary(&profile)))
            }
            EngineAction::GetAndroidStatus { vm_id } => {
                let report = self
                    .android_service
                    .status(&vm_id)
                    .map_err(debug_error("android_status_failed"))?;
                Ok(EngineResponseData::AndroidStatus(Self::android_status_summary(vm_id, &report)))
            }
            EngineAction::WaitAndroidReady { vm_id } => {
                let report = self
                    .android_service
                    .wait_until_ready(&vm_id)
                    .map_err(debug_error("android_ready_failed"))?;
                Ok(EngineResponseData::AndroidStatus(Self::android_status_summary(vm_id, &report)))
            }
            EngineAction::ApplyAndroidDisplay { vm_id } => {
                self.android_service
                    .apply_display(&vm_id)
                    .map_err(debug_error("android_display_failed"))?;
                Ok(EngineResponseData::Ack)
            }
            EngineAction::ListAndroidPackages { vm_id } => {
                let packages = self
                    .android_service
                    .list_packages(&vm_id)
                    .map_err(debug_error("android_package_list_failed"))?
                    .into_iter()
                    .map(|package| AndroidPackageDto {
                        package_name: package.package_name.as_str().to_owned(),
                    })
                    .collect();
                Ok(EngineResponseData::AndroidPackageList { packages })
            }
            EngineAction::InstallAndroidApk { vm_id, relative_apk_path } => {
                self.android_service
                    .install_apk(vm_id, relative_apk_path)
                    .map_err(debug_error("android_apk_install_failed"))?;
                Ok(EngineResponseData::Ack)
            }
            EngineAction::UninstallAndroidPackage { vm_id, package_name } => {
                self.android_service
                    .uninstall_package(vm_id, package_name)
                    .map_err(debug_error("android_package_uninstall_failed"))?;
                Ok(EngineResponseData::Ack)
            }
            EngineAction::LaunchAndroidPackage { vm_id, package_name } => {
                self.android_service
                    .launch_package(vm_id, package_name)
                    .map_err(debug_error("android_package_launch_failed"))?;
                Ok(EngineResponseData::Ack)
            }
            EngineAction::StopAndroidPackage { vm_id, package_name } => {
                self.android_service
                    .stop_package(vm_id, package_name)
                    .map_err(debug_error("android_package_stop_failed"))?;
                Ok(EngineResponseData::Ack)
            }
            EngineAction::InjectAndroidInput { vm_id, input } => {
                self.android_service
                    .inject_input(vm_id, android_input_from_dto(input))
                    .map_err(debug_error("android_input_failed"))?;
                Ok(EngineResponseData::Ack)
            }
            EngineAction::GetGamingInputCapabilities { vm_id } => {
                let base = self
                    .gaming_input_service
                    .capabilities_for(&vm_id)
                    .map_err(debug_error("gaming_input_capabilities_failed"))?;
                let persistent = self
                    .android_service
                    .guest_agent_status(&vm_id)
                    .map(|report| report.available && report.persistent_multi_touch)
                    .unwrap_or(false);
                Ok(EngineResponseData::GamingInputCapabilities(
                    gaming_input_capabilities_summary(base.with_persistent_multi_touch(persistent)),
                ))
            }
            EngineAction::GetGamingInputProfile { vm_id } => {
                let profile = self
                    .gaming_input_service
                    .profile(&vm_id)
                    .map_err(debug_error("gaming_input_profile_failed"))?;
                Ok(EngineResponseData::GamingInputProfile(
                    gaming_input_profile_summary(&profile),
                ))
            }
            EngineAction::ConfigureGamingInputProfile { profile } => {
                let profile = gaming_input_profile_from_dto(profile)
                    .map_err(debug_error("gaming_input_profile_invalid"))?;
                let profile = self
                    .gaming_input_service
                    .save_profile(profile)
                    .map_err(debug_error("gaming_input_profile_save_failed"))?;
                Ok(EngineResponseData::GamingInputProfile(
                    gaming_input_profile_summary(&profile),
                ))
            }
            EngineAction::InjectGamingInputEvent { vm_id, event } => {
                let plan = self
                    .gaming_input_service
                    .translate(&vm_id, gaming_input_event_from_dto(event))
                    .map_err(debug_error("gaming_input_translate_failed"))?;
                self.execute_gaming_input_plan(&vm_id, plan)?;
                Ok(EngineResponseData::Ack)
            }
            EngineAction::ResetGamingInputState { vm_id } => {
                self.gaming_input_service
                    .reset_state(&vm_id)
                    .map_err(debug_error("gaming_input_reset_failed"))?;
                let _ = self.android_service.reset_guest_input(&vm_id);
                Ok(EngineResponseData::Ack)
            }
            EngineAction::GetGuestAgentStatus { vm_id } => {
                let report = self
                    .android_service
                    .guest_agent_status(&vm_id)
                    .map_err(debug_error("guest_agent_status_failed"))?;
                Ok(EngineResponseData::GuestAgentStatus(guest_agent_status_summary(&report)))
            }
            EngineAction::GetGuestClipboard { vm_id } => {
                let text = self
                    .android_service
                    .read_guest_clipboard(&vm_id)
                    .map_err(debug_error("guest_clipboard_read_failed"))?;
                Ok(EngineResponseData::Clipboard { text })
            }
            EngineAction::SetGuestClipboard { vm_id, text } => {
                self.android_service
                    .write_guest_clipboard(&vm_id, &text)
                    .map_err(debug_error("guest_clipboard_write_failed"))?;
                Ok(EngineResponseData::Ack)
            }
            EngineAction::ListGameCatalog => {
                let games = self.game_catalog_service.list().map_err(debug_error("game_catalog_list_failed"))?;
                Ok(EngineResponseData::GameCatalogList { games: games.iter().map(game_catalog_entry_summary).collect() })
            }
            EngineAction::ListGuestCatalog => {
                let templates = self.guest_catalog_service.list().map_err(debug_error("guest_catalog_list_failed"))?;
                Ok(EngineResponseData::GuestCatalogList {
                    templates: templates.iter().map(guest_template_summary).collect(),
                })
            }
            EngineAction::DetectGames { vm_id } => {
                let packages = self.android_service.list_packages(&vm_id).map_err(debug_error("game_detect_package_list_failed"))?;
                let package_names = packages.iter().map(|value| value.package_name.as_str().to_owned()).collect::<Vec<_>>();
                let games = self.game_catalog_service.detect(&package_names).map_err(debug_error("game_detect_failed"))?;
                Ok(EngineResponseData::DetectedGames { games: games.iter().map(game_catalog_entry_summary).collect() })
            }
            EngineAction::GetGameCompatibility { vm_id, game_id } => {
                let game = self.game_catalog_service.get(&game_id).map_err(debug_error("game_catalog_get_failed"))?;
                let packages = self.android_service.list_packages(&vm_id).unwrap_or_default();
                let package_installed = packages.iter().any(|value| game.matches_package(value.package_name.as_str()));
                let status = self.android_service.status(&vm_id).ok();
                let persistent_multi_touch = self.android_service.guest_agent_status(&vm_id).map(|value| value.available && value.persistent_multi_touch).unwrap_or(false);
                let context = GameRuntimeContext {
                    sdk_level: status.as_ref().and_then(|value| value.sdk_level),
                    abi: status.as_ref().and_then(|value| value.abi.as_ref().map(|abi| abi.as_str().to_owned())),
                    persistent_multi_touch,
                    relative_mouse_look: true,
                    gpu_backend: catalog_gpu_backend(self.effective_gpu_backend),
                };
                let report = self.game_catalog_service.evaluate(&game, &context);
                Ok(EngineResponseData::GameCompatibility(GameCompatibilityDto {
                    game: game_catalog_entry_summary(&game),
                    status: compatibility_status_summary(report.status),
                    package_installed,
                    blockers: report.blockers,
                    warnings: report.warnings,
                }))
            }
            EngineAction::ApplyGameProfile { vm_id, game_id } => {
                let game = self.game_catalog_service.get(&game_id).map_err(debug_error("game_catalog_get_failed"))?;
                let old_input = self.gaming_input_service.profile(&vm_id).ok();
                let input_profile = game_input_profile_from_definition(&vm_id, &game).map_err(debug_error("game_profile_input_invalid"))?;
                let saved_input = self.gaming_input_service.save_profile(input_profile).map_err(debug_error("game_profile_input_save_failed"))?;
                let android_result = self.android_service.configure_runtime(
                    vm_id.clone(),
                    Some(game.android.width),
                    Some(game.android.height),
                    Some(game.android.density_dpi),
                    Some(game.android.target_fps),
                );
                let android = match android_result {
                    Ok(profile) => profile,
                    Err(error) => {
                        if let Some(profile) = old_input { let _ = self.gaming_input_service.save_profile(profile); }
                        else { let _ = self.gaming_input_service.delete_profile(&vm_id); }
                        return Err((String::from("game_profile_android_apply_failed"), format!("{error:?}")));
                    }
                };
                Ok(EngineResponseData::GameProfileApplied {
                    game: game_catalog_entry_summary(&game),
                    android: Self::android_profile_summary(&android),
                    gaming_input: gaming_input_profile_summary(&saved_input),
                })
            }
            EngineAction::ListAndroidImages => {
                let images = self.android_image_service.list().map_err(debug_error("android_image_list_failed"))?;
                let mut summaries = Vec::with_capacity(images.len());
                for image in &images {
                    let progress = self
                        .android_image_service
                        .install_progress(image.id.as_str())
                        .unwrap_or(None);
                    let mut summary = android_image_summary(image);
                    summary.install_progress = progress.as_ref().map(android_image_install_progress_summary);
                    summaries.push(summary);
                }
                Ok(EngineResponseData::AndroidImageList { images: summaries })
            }
            EngineAction::DefineAndroidImage { image_id, name, requested_release } => {
                let image = self.android_image_service.define(image_id, name, requested_release).map_err(debug_error("android_image_define_failed"))?;
                Ok(EngineResponseData::AndroidImage { image: android_image_summary(&image) })
            }
            EngineAction::PrepareAndroidImageBuild { image_id } => {
                let plan = self.android_image_service.prepare_build(image_id).map_err(debug_error("android_image_build_plan_failed"))?;
                Ok(EngineResponseData::AndroidImageBuildPlan(android_image_build_plan_summary(&plan)))
            }
            EngineAction::RegisterAndroidImageBuild { image_id } => {
                let image = self.android_image_service.register_build(image_id).map_err(debug_error("android_image_register_failed"))?;
                Ok(EngineResponseData::AndroidImage { image: android_image_summary(&image) })
            }
            EngineAction::InstallAndroidImageDistribution { image_id } => {
                let image = self.android_image_service.install_distribution(image_id).map_err(debug_error("android_image_distribution_install_failed"))?;
                Ok(EngineResponseData::AndroidImage { image: android_image_summary(&image) })
            }
            EngineAction::CancelAndroidImageDistribution { image_id } => {
                let image = self.android_image_service.cancel_distribution(image_id).map_err(debug_error("android_image_distribution_cancel_failed"))?;
                Ok(EngineResponseData::AndroidImage { image: android_image_summary(&image) })
            }
            EngineAction::CleanupAndroidImageDistribution { image_id } => {
                let image = self.android_image_service.cleanup_distribution(image_id).map_err(debug_error("android_image_distribution_cleanup_failed"))?;
                Ok(EngineResponseData::AndroidImage { image: android_image_summary(&image) })
            }
            EngineAction::AssignAndroidImage { vm_id, image_id } => {
                let assignment = self.android_image_service.assign(vm_id, image_id).map_err(debug_error("android_image_assign_failed"))?;
                Ok(EngineResponseData::AndroidImageAssignment { assignment: Some(android_image_assignment_summary(&assignment)) })
            }
            EngineAction::GetAndroidImageAssignment { vm_id } => {
                let assignment = self.android_image_service.assignment(&vm_id).map_err(debug_error("android_image_assignment_failed"))?;
                Ok(EngineResponseData::AndroidImageAssignment { assignment: assignment.as_ref().map(android_image_assignment_summary) })
            }
        }
    }

    fn start_virtual_machine(&mut self, vm_id: String) -> Result<VirtualMachine, (String, String)> {
        let machine = self
            .query_service
            .get(&vm_id)
            .map_err(debug_error("vm_start_failed"))?;
        let machine = if machine.state() == VmState::Error {
            self.lifecycle_service
                .recover_vm(RecoverVmCommand::new(vm_id.clone()))
                .map_err(debug_error("vm_recovery_failed"))?
        } else {
            machine
        };
        if machine.guest_boot().profile() != GuestProfile::Android {
            return self
                .lifecycle_service
                .start_vm(StartVmCommand::new(vm_id))
                .map_err(debug_error("vm_start_failed"));
        }

        let assignment = self
            .android_image_service
            .assignment(&vm_id)
            .map_err(debug_error("android_image_assignment_failed"))?
            .ok_or_else(|| (
                String::from("android_image_required"),
                String::from("Android VM requires an assigned Ready Android image before start"),
            ))?;
        let requires_first_boot = assignment.provisioning_state != AndroidImageProvisioningState::Ready;

        let runtime_profile = self.android_service
            .ensure_runtime_profile(&vm_id)
            .map_err(debug_error("android_runtime_profile_failed"))?;
        let runtime_media = self
            .android_image_service
            .prepare_runtime_media(&vm_id, &runtime_profile)
            .map_err(debug_error("android_runtime_media_failed"))?;

        let requires_guest_agent = if requires_first_boot {
            self.android_image_service
                .assignment_requires_guest_agent(&vm_id)
                .map_err(debug_error("android_image_capabilities_failed"))?
        } else {
            false
        };

        if requires_first_boot {
            self.android_image_service
                .begin_boot_attempt(&vm_id)
                .map_err(debug_error("android_boot_attempt_failed"))?;
        }

        let started = match self
            .lifecycle_service
            .start_vm(StartVmCommand::new(vm_id.clone()).with_runtime_media(runtime_media))
        {
            Ok(machine) => machine,
            Err(error) => {
                if requires_first_boot {
                    let _ = self.android_image_service.mark_failed(&vm_id, format!("vm_start_failed: {error:?}"));
                }
                return Err((String::from("vm_start_failed"), format!("{error:?}")));
            }
        };

        if !requires_first_boot {
            return Ok(started);
        }

        let first_boot_result = (|| -> Result<(), String> {
            self.android_service
                .wait_until_ready(&vm_id)
                .map_err(|error| format!("adb_ready_failed: {error:?}"))?;
            self.android_service
                .apply_display(&vm_id)
                .map_err(|error| format!("display_provision_failed: {error:?}"))?;
            if requires_guest_agent {
                self.android_service
                    .provision_guest_agent(&vm_id)
                    .map_err(|error| format!("guest_agent_provision_failed: {error:?}"))?;
            }
            self.android_image_service
                .mark_ready(&vm_id)
                .map_err(|error| format!("assignment_ready_failed: {error:?}"))?;
            Ok(())
        })();

        if let Err(error) = first_boot_result {
            let _ = self.android_image_service.mark_failed(&vm_id, error.clone());
            let _ = self.lifecycle_service.stop_vm(StopVmCommand::new(vm_id));
            return Err((String::from("android_first_boot_failed"), error));
        }

        Ok(started)
    }

    fn dashboard(&self) -> Result<DashboardDto, (String, String)> {
        let report = self.host_service.inspect();
        let machines = self
            .query_service
            .list()
            .map_err(debug_error("dashboard_vm_list_failed"))?
            .iter()
            .map(Self::vm_summary)
            .collect();

        let qemu_img_ready = report
            .hypervisor
            .as_ref()
            .and_then(|value| value.disk_binary.as_ref())
            .is_some();
        let android = self.android_service.capabilities();
        Ok(DashboardDto {
            engine_ready: true,
            host_id: self.host_id.clone(),
            host_label: self.host_label.clone(),
            transport_security: self.transport_security.clone(),
            authentication_required: self.auth_service.authentication_required(),
            platform: platform_name(&report.host.platform).to_owned(),
            architecture: architecture_name(&report.host.architecture).to_owned(),
            acceleration: acceleration_name(report.preferred_acceleration).to_owned(),
            qemu_ready: report.hypervisor.is_some(),
            qemu_img_ready,
            qemu_version: report.hypervisor.map(|value| value.version_text),
            gpu_backend: self.gpu_capabilities.effective_backend.clone(),
            gpu_vulkan_ready: self.gpu_capabilities.vulkan_loader_available,
            gpu_accelerated: matches!(
                self.gpu_capabilities.effective_backend.as_str(),
                "virgl_venus" | "gfxstream"
            ),
            android_adb_ready: android.available,
            android_adb_version: android.version_text,
            machines,
        })
    }

    fn vm_summary(machine: &VirtualMachine) -> VmSummaryDto {
        VmSummaryDto {
            id: machine.id().as_str().to_owned(),
            name: machine.name().to_owned(),
            state: state_name(machine.state()).to_owned(),
            vcpu_count: machine.resources().vcpu_count,
            memory_mib: machine.resources().memory_mib,
            acceleration: acceleration_name(machine.acceleration()).to_owned(),
            disk_count: machine.disks().len(),
            network_count: machine.networks().len(),
            snapshot_count: machine.snapshots().len(),
            guest_profile: guest_profile_name(machine.guest_boot().profile()).to_owned(),
            guest_template_id: machine.guest_boot().catalog_template_id().map(str::to_owned),
            installer_media: machine
                .guest_boot()
                .installer_iso()
                .map(|iso| iso.relative_path().to_owned()),
            disks: machine
                .disks()
                .iter()
                .map(|attachment| VmDiskDto {
                    id: attachment.image().id().as_str().to_owned(),
                    format: disk_format_extension(attachment.image().format()).to_owned(),
                    virtual_size_bytes: attachment.image().virtual_size_bytes(),
                    relative_path: attachment.image().relative_path().to_owned(),
                    bus: disk_bus_name(attachment.bus()).to_owned(),
                    boot_index: attachment.boot_index(),
                })
                .collect(),
            networks: machine
                .networks()
                .iter()
                .map(|attachment| {
                    let managed = attachment.managed_address();
                    VmNetworkDto {
                        id: attachment.id().as_str().to_owned(),
                        fabric_id: attachment.fabric_id().map(|value| value.as_str().to_owned()),
                        mode: network_mode_name(attachment.mode()).to_owned(),
                        device_model: network_device_model_name(attachment.device_model()).to_owned(),
                        mac_address: attachment.mac_address().map(|mac| mac.to_canonical_string()),
                        ipv4_address: managed.map(|address| address.ipv4_address().to_string()),
                        prefix_length: managed.map(|address| address.prefix_length()),
                        gateway: managed.and_then(|address| address.gateway()).map(|value| value.to_string()),
                        dns_servers: managed.map(|address| address.dns_servers().iter().map(ToString::to_string).collect()).unwrap_or_default(),
                        port_forward_count: attachment.port_forwards().len(),
                        published_services: attachment.port_forwards().iter().map(|rule| PublishedNetworkServiceDto {
                            protocol: port_protocol_name(rule.protocol()).to_owned(),
                            host_ip: rule.host_ip().unwrap_or(std::net::Ipv4Addr::UNSPECIFIED).to_string(),
                            host_port: rule.host_port(),
                            guest_ip: rule.guest_ip().or_else(|| managed.map(|address| address.ipv4_address())).map(|value| value.to_string()).unwrap_or_default(),
                            guest_port: rule.guest_port(),
                        }).collect(),
                    }
                })
                .collect(),
            last_error: machine.last_failure().map(|failure| failure.detail.clone()),
        }
    }

    fn network_command_for_profile(
        &self,
        vm_id: String,
        network_id: String,
        mode: NetworkMode,
        bridge_name: Option<String>,
        tap_name: Option<String>,
    ) -> Result<AttachNetworkCommand, (String, String)> {
        match mode {
            NetworkMode::ManagedNat | NetworkMode::Private => {
                let policy = if mode == NetworkMode::ManagedNat { &self.managed_nat_policy } else { &self.private_network_policy };
                let ip = self.allocate_managed_ipv4(policy)?;
                let mac = deterministic_vm_mac(&vm_id, &network_id);
                Ok(AttachNetworkCommand::new_managed(
                    vm_id,
                    network_id,
                    mode,
                    self.default_network_device_model,
                    mac,
                    ManagedAddressCommand::new(ip, policy.prefix_length, Some(policy.gateway), policy.dns_servers.clone()),
                    if mode == NetworkMode::ManagedNat { self.default_network_id.clone() } else { self.private_network_id.clone() },
                ))
            }
            NetworkMode::UserNat => Ok(AttachNetworkCommand::new_user_nat(
                vm_id, network_id, self.default_network_device_model, None, Vec::new(),
            )),
            NetworkMode::Bridge => Ok(AttachNetworkCommand::new_bridge(
                vm_id, network_id, self.default_network_device_model, None, bridge_name, tap_name,
            )),
        }
    }

    fn ensure_host_port_available(
        &self,
        protocol: PortProtocol,
        host_ip: std::net::Ipv4Addr,
        host_port: u16,
    ) -> Result<(), (String, String)> {
        let machines = self
            .query_service
            .list()
            .map_err(debug_error("vm_network_host_port_query_failed"))?;
        for machine in machines {
            for network in machine.networks() {
                for rule in network.port_forwards() {
                    let address_overlap = rule.host_ip().is_none() || rule.host_ip() == Some(host_ip);
                    if rule.protocol() == protocol && address_overlap && rule.host_port() == host_port {
                        return Err((
                            String::from("vm_network_host_port_in_use"),
                            format!(
                                "Host port {host_port} is already published by VM {} network {}",
                                machine.id().as_str(),
                                network.id().as_str()
                            ),
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn allocate_managed_ipv4(&self, policy: &ManagedNetworkPoolPolicy) -> Result<std::net::Ipv4Addr, (String, String)> {
        let machines = self.query_service.list().map_err(debug_error("vm_network_ipam_query_failed"))?;
        let used = machines.iter().flat_map(|machine| machine.networks().iter()).filter_map(|attachment| {
            attachment.managed_address().map(|address| address.ipv4_address())
        }).collect::<std::collections::HashSet<_>>();
        let base = policy.subnet.octets();
        for host in policy.pool_start..=policy.pool_end {
            let candidate = std::net::Ipv4Addr::new(base[0], base[1], base[2], host);
            if candidate != policy.gateway && !used.contains(&candidate) {
                return Ok(candidate);
            }
        }
        Err((
            String::from("vm_network_ipam_exhausted"),
            String::from("Managed network IPv4 pool is exhausted"),
        ))
    }

    fn display_summary(vm_id: String, display: DisplayRuntimeInfo) -> DisplaySessionDto {
        DisplaySessionDto {
            vm_id,
            transport: match display.transport {
                DisplayTransport::Rfb => String::from("rfb"),
            },
            endpoint: display.endpoint.to_string(),
            local_only: display.local_only,
            fullscreen: display.capabilities.fullscreen,
            absolute_pointer: display.capabilities.absolute_pointer,
            relative_pointer_capture: display.capabilities.relative_pointer_capture,
            keyboard: display.capabilities.keyboard,
            gamepad_observation: display.capabilities.gamepad_observation,
        }
    }

    fn android_profile_summary(profile: &AndroidRuntimeProfile) -> AndroidRuntimeProfileDto {
        AndroidRuntimeProfileDto {
            vm_id: profile.vm_id().as_str().to_owned(),
            adb_endpoint: profile.adb_serial(),
            adb_guest_port: profile.adb_guest_port(),
            width: profile.display().width(),
            height: profile.display().height(),
            density_dpi: profile.display().density_dpi(),
            target_fps: profile.display().target_fps(),
        }
    }

    fn android_status_summary(vm_id: String, report: &AndroidDeviceReport) -> AndroidDeviceStatusDto {
        AndroidDeviceStatusDto {
            vm_id,
            serial: report.serial.clone(),
            connection_state: android_connection_state_name(report.connection_state).to_owned(),
            boot_completed: report.boot_completed,
            sdk_level: report.sdk_level,
            abi: report.abi.as_ref().map(|value| value.as_str().to_owned()),
            model: report.model.clone(),
        }
    }

    fn execute_gaming_input_plan(
        &mut self,
        vm_id: &str,
        plan: GamingInputPlan,
    ) -> Result<(), (String, String)> {
        match plan {
            GamingInputPlan::Noop => Ok(()),
            GamingInputPlan::AndroidTap { position } => {
                let profile = self
                    .android_service
                    .profile(vm_id)
                    .map_err(debug_error("gaming_input_android_profile_failed"))?;
                let (x, y) = position.to_pixels(profile.display().width(), profile.display().height());
                self.android_service
                    .inject_input(vm_id.to_owned(), AndroidInputAction::Tap { x, y })
                    .map_err(debug_error("gaming_input_tap_failed"))
            }
            GamingInputPlan::AndroidKey { key_code } => self
                .android_service
                .inject_input(vm_id.to_owned(), AndroidInputAction::KeyEvent { key_code })
                .map_err(debug_error("gaming_input_key_failed")),
            GamingInputPlan::TouchFrame { contacts } => {
                let contacts = contacts.into_iter().map(|contact| AndroidTouchContact::create(
                    contact.pointer_id,
                    match contact.phase { TouchPhase::Down => AndroidTouchPhase::Down, TouchPhase::Move => AndroidTouchPhase::Move, TouchPhase::Up => AndroidTouchPhase::Up },
                    contact.position.x(),
                    contact.position.y(),
                ).map_err(debug_error("gaming_input_touch_contact_invalid"))).collect::<Result<Vec<_>, _>>()?;
                self.android_service
                    .apply_guest_touch_frame(vm_id, &contacts)
                    .map_err(debug_error("gaming_input_guest_agent_failed"))
            }
        }
    }

    fn snapshot_summary(snapshot: &SnapshotRecord) -> SnapshotDto {
        SnapshotDto {
            id: snapshot.id().as_str().to_owned(),
            name: snapshot.name().to_owned(),
            created_at_unix_ms: snapshot.created_at_unix_ms(),
            disk_count: snapshot.disk_ids().len(),
        }
    }
}

fn resolve_gpu_profile<H, V>(
    service: &GamingGpuService<H, V>,
    policy: GamingGpuPolicy,
) -> (GamingGpuCapabilityReport, ResolvedGamingGpuProfile, Option<String>)
where
    H: turkuazvm_gpu::ports::host_gpu_probe_port::HostGpuProbePort,
    V: turkuazvm_gpu::ports::hypervisor_gpu_probe_port::HypervisorGpuProbePort,
{
    match service.resolve(policy) {
        Ok((report, profile)) => (report, profile, None),
        Err(error) => {
            let report = service.inspect().unwrap_or_else(|_| fallback_gpu_report());
            (
                report,
                ResolvedGamingGpuProfile {
                    backend: GpuBackend::Software,
                    hostmem_mib: policy.hostmem_mib,
                    experimental: false,
                },
                Some(format!("{error:?}")),
            )
        }
    }
}

fn fallback_gpu_report() -> GamingGpuCapabilityReport {
    let platform = match std::env::consts::OS {
        "windows" => GpuHostPlatform::Windows,
        "linux" => GpuHostPlatform::Linux,
        _ => GpuHostPlatform::Unsupported,
    };
    GamingGpuCapabilityReport {
        host: HostGpuCapabilities {
            platform,
            vulkan_loader_available: false,
            vulkan_probe_available: false,
            vulkan_summary: None,
            opengl_probe_available: false,
        },
        hypervisor: HypervisorGpuCapabilities {
            virtio_gpu_2d: false,
            virgl: false,
            venus: false,
            rutabaga: false,
            gfxstream_vulkan: false,
            android_gfxstream_experimental: false,
        },
    }
}

fn gpu_capabilities_summary(
    policy: GamingGpuPolicy,
    report: &GamingGpuCapabilityReport,
    resolved: ResolvedGamingGpuProfile,
    resolution_error: Option<String>,
) -> GpuCapabilitiesDto {
    GpuCapabilitiesDto {
        requested_mode: gpu_preference_name(policy.preference).to_owned(),
        effective_backend: gpu_backend_name(resolved.backend).to_owned(),
        hostmem_mib: resolved.hostmem_mib,
        experimental: resolved.experimental,
        vulkan_loader_available: report.host.vulkan_loader_available,
        vulkan_probe_available: report.host.vulkan_probe_available,
        vulkan_summary: report.host.vulkan_summary.clone(),
        opengl_available: report.host.opengl_probe_available,
        qemu_virtio_2d: report.hypervisor.virtio_gpu_2d,
        qemu_virgl: report.hypervisor.virgl,
        qemu_venus: report.hypervisor.venus,
        qemu_rutabaga: report.hypervisor.rutabaga,
        qemu_gfxstream_vulkan: report.hypervisor.gfxstream_vulkan,
        android_gfxstream_experimental: report.hypervisor.android_gfxstream_experimental,
        resolution_error,
    }
}

const fn gpu_preference_name(value: GpuBackendPreference) -> &'static str {
    match value {
        GpuBackendPreference::Auto => "auto",
        GpuBackendPreference::Software => "software",
        GpuBackendPreference::Virtio2d => "virtio_2d",
        GpuBackendPreference::VirglVenus => "virgl_venus",
        GpuBackendPreference::Gfxstream => "gfxstream",
    }
}

const fn gpu_backend_name(value: GpuBackend) -> &'static str {
    match value {
        GpuBackend::Software => "software",
        GpuBackend::Virtio2d => "virtio_2d",
        GpuBackend::VirglVenus => "virgl_venus",
        GpuBackend::Gfxstream => "gfxstream",
    }
}

const fn disk_bus_name(bus: DiskBus) -> &'static str {
    match bus {
        DiskBus::Virtio => "virtio",
        DiskBus::Ide => "ide",
    }
}

const fn network_mode_name(mode: NetworkMode) -> &'static str {
    match mode {
        NetworkMode::ManagedNat => "managed_nat",
        NetworkMode::Private => "private",
        NetworkMode::UserNat => "user_nat",
        NetworkMode::Bridge => "bridge",
    }
}

const fn network_profile_mode(profile: NetworkProfileDto) -> NetworkMode {
    match profile {
        NetworkProfileDto::ManagedNat => NetworkMode::ManagedNat,
        NetworkProfileDto::Private => NetworkMode::Private,
        NetworkProfileDto::Bridge => NetworkMode::Bridge,
        NetworkProfileDto::UserNat => NetworkMode::UserNat,
    }
}

const fn network_protocol(protocol: NetworkProtocolDto) -> PortProtocol {
    match protocol { NetworkProtocolDto::Tcp => PortProtocol::Tcp, NetworkProtocolDto::Udp => PortProtocol::Udp }
}

const fn port_protocol_name(protocol: PortProtocol) -> &'static str {
    match protocol { PortProtocol::Tcp => "tcp", PortProtocol::Udp => "udp" }
}

fn subnet_prefix(subnet: std::net::Ipv4Addr) -> String {
    let octets = subnet.octets();
    format!("{}.{}.{}", octets[0], octets[1], octets[2])
}

fn deterministic_vm_mac(vm_id: &str, network_id: &str) -> String {
    let mut hash = 0x811c9dc5_u32;
    for byte in vm_id.bytes().chain([b':']).chain(network_id.bytes()) {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(0x01000193);
    }
    format!("02:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}", (hash >> 24) as u8, (hash >> 16) as u8, (hash >> 8) as u8, hash as u8, (hash ^ 0xa5) as u8)
}

const fn disk_format_extension(format: DiskFormat) -> &'static str {
    match format {
        DiskFormat::Qcow2 => "qcow2",
        DiskFormat::Raw => "raw",
    }
}

const fn network_device_model_name(model: NetworkDeviceModel) -> &'static str {
    match model {
        NetworkDeviceModel::VirtioNetPci => "virtio_net_pci",
        NetworkDeviceModel::E1000 => "e1000",
    }
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or_default()
}

const fn clone_mode(value: CloneModeDto) -> CloneMode {
    match value {
        CloneModeDto::Full => CloneMode::Full,
        CloneModeDto::Linked => CloneMode::Linked,
    }
}

fn artifact_cache_entry_summary(record: &ArtifactCacheRecord) -> ArtifactCacheEntryDto {
    ArtifactCacheEntryDto {
        source_key: record.source_key.clone(),
        source_url: record.source_url.clone(),
        sha256: record.sha256.clone(),
        size_bytes: record.size_bytes,
        immutable: record.immutable,
        pinned: record.pinned,
        etag: record.validators.etag.clone(),
        last_modified: record.validators.last_modified.clone(),
        created_at_unix_ms: record.created_at_unix_ms,
        last_access_unix_ms: record.last_access_unix_ms,
        last_revalidated_unix_ms: record.last_revalidated_unix_ms,
    }
}

fn artifact_cache_revalidation_summary(report: &ArtifactCacheRevalidation) -> ArtifactCacheRevalidationDto {
    ArtifactCacheRevalidationDto {
        source_key: report.source_key.clone(),
        state: match report.state {
            ArtifactCacheRevalidationState::Immutable => ArtifactCacheRevalidationStateDto::Immutable,
            ArtifactCacheRevalidationState::NotModified => ArtifactCacheRevalidationStateDto::NotModified,
            ArtifactCacheRevalidationState::RemoteModified => ArtifactCacheRevalidationStateDto::RemoteModified,
            ArtifactCacheRevalidationState::ValidationFailed => ArtifactCacheRevalidationStateDto::ValidationFailed,
        },
        etag: report.validators.etag.clone(),
        last_modified: report.validators.last_modified.clone(),
        detail: report.detail.clone(),
    }
}

fn debug_error<E: std::fmt::Debug>(code: &'static str) -> impl FnOnce(E) -> (String, String) {
    move |error| (code.to_owned(), format!("{error:?}"))
}

fn platform_name(value: &HostPlatform) -> &str {
    match value {
        HostPlatform::Windows => "windows",
        HostPlatform::Linux => "linux",
        HostPlatform::Unsupported(_) => "unsupported",
    }
}

fn android_image_summary(image: &AndroidImage) -> AndroidImageDto {
    let profile = image.build_profile;
    AndroidImageDto {
        id: image.id.as_str().to_owned(),
        name: image.name.clone(),
        architecture: match image.architecture {
            AndroidImageArchitecture::X86_64 => AndroidImageArchitectureDto::X86_64,
            AndroidImageArchitecture::Arm64 => AndroidImageArchitectureDto::Arm64,
        },
        state: match image.state {
            AndroidImageState::Defined => AndroidImageStateDto::Defined,
            AndroidImageState::BuildPlanned => AndroidImageStateDto::BuildPlanned,
            AndroidImageState::Installing => AndroidImageStateDto::Installing,
            AndroidImageState::Ready => AndroidImageStateDto::Ready,
            AndroidImageState::Failed => AndroidImageStateDto::Failed,
        },
        branch: profile.source_track.branch().to_owned(),
        lunch_target: format!("{}-{}-{}", profile.product.code(), profile.release_config.code(), profile.variant.code()),
        capabilities: AndroidImageCapabilitiesDto {
            adb_tcp: image.capabilities.adb_tcp,
            guest_agent_included: image.capabilities.guest_agent_included,
            persistent_multi_touch: image.capabilities.persistent_multi_touch,
            native_x86_64: image.capabilities.native_x86_64,
            arm_translation: image.capabilities.arm_translation,
        },
        requested_release: image.requested_release.clone(),
        source_revision: image.source_revision.clone(),
        android_release: image.android_release.clone(),
        sdk_level: image.sdk_level,
        artifacts: image.artifacts.iter().map(|artifact| AndroidImageArtifactDto {
            role: match artifact.role {
                AndroidImageArtifactRole::Boot => AndroidImageArtifactRoleDto::Boot,
                AndroidImageArtifactRole::InitBoot => AndroidImageArtifactRoleDto::InitBoot,
                AndroidImageArtifactRole::VendorBoot => AndroidImageArtifactRoleDto::VendorBoot,
                AndroidImageArtifactRole::Super => AndroidImageArtifactRoleDto::Super,
                AndroidImageArtifactRole::Userdata => AndroidImageArtifactRoleDto::Userdata,
                AndroidImageArtifactRole::Vbmeta => AndroidImageArtifactRoleDto::Vbmeta,
                AndroidImageArtifactRole::VbmetaSystem => AndroidImageArtifactRoleDto::VbmetaSystem,
                AndroidImageArtifactRole::Metadata => AndroidImageArtifactRoleDto::Metadata,
                AndroidImageArtifactRole::Misc => AndroidImageArtifactRoleDto::Misc,
                AndroidImageArtifactRole::Bootloader => AndroidImageArtifactRoleDto::Bootloader,
                AndroidImageArtifactRole::CompositeDisk => AndroidImageArtifactRoleDto::CompositeDisk,
                AndroidImageArtifactRole::Kernel => AndroidImageArtifactRoleDto::Kernel,
                AndroidImageArtifactRole::Ramdisk | AndroidImageArtifactRole::System => AndroidImageArtifactRoleDto::Other,
                AndroidImageArtifactRole::Other => AndroidImageArtifactRoleDto::Other,
            },
            relative_path: artifact.relative_path.clone(),
            size_bytes: artifact.size_bytes,
            sha256: artifact.sha256.clone(),
        }).collect(),
        last_error: image.last_error.clone(),
        install_progress: None,
    }
}

fn android_image_install_progress_summary(progress: &AndroidImageDistributionProgress) -> AndroidImageInstallProgressDto {
    AndroidImageInstallProgressDto {
        stage: match progress.stage {
            AndroidImageDistributionStage::Discovering => AndroidImageInstallStageDto::Discovering,
            AndroidImageDistributionStage::CheckingDisk => AndroidImageInstallStageDto::CheckingDisk,
            AndroidImageDistributionStage::DownloadingDevice => AndroidImageInstallStageDto::DownloadingDevice,
            AndroidImageDistributionStage::DownloadingHost => AndroidImageInstallStageDto::DownloadingHost,
            AndroidImageDistributionStage::Validating => AndroidImageInstallStageDto::Validating,
            AndroidImageDistributionStage::Extracting => AndroidImageInstallStageDto::Extracting,
            AndroidImageDistributionStage::Assembling => AndroidImageInstallStageDto::Assembling,
            AndroidImageDistributionStage::Finalizing => AndroidImageInstallStageDto::Finalizing,
            AndroidImageDistributionStage::Cancelling => AndroidImageInstallStageDto::Cancelling,
            AndroidImageDistributionStage::Completed => AndroidImageInstallStageDto::Completed,
            AndroidImageDistributionStage::Failed => AndroidImageInstallStageDto::Failed,
        },
        downloaded_bytes: progress.downloaded_bytes,
        total_bytes: progress.total_bytes,
        detail: progress.detail.clone(),
        log_path: progress.log_path.clone(),
        elapsed_seconds: progress.elapsed_seconds,
        bytes_per_second: progress.bytes_per_second,
        eta_seconds: progress.eta_seconds,
    }
}

fn android_image_build_plan_summary(plan: &AndroidImageBuildPlan) -> AndroidImageBuildPlanDto {
    AndroidImageBuildPlanDto {
        supported_host: plan.supported_host,
        source_root: plan.source_root.clone(),
        output_root: plan.output_root.clone(),
        branch: plan.branch.clone(),
        lunch_target: plan.lunch_target.clone(),
        build_script: plan.build_script.clone(),
        minimum_free_disk_gib: plan.minimum_free_disk_gib,
        notes: plan.notes.clone(),
    }
}

fn android_image_assignment_summary(assignment: &AndroidImageAssignment) -> AndroidImageAssignmentDto {
    AndroidImageAssignmentDto {
        vm_id: assignment.vm_id.clone(),
        image_id: assignment.image_id.as_str().to_owned(),
        provisioning_state: match assignment.provisioning_state {
            AndroidImageProvisioningState::PendingFirstBoot => AndroidImageProvisioningStateDto::PendingFirstBoot,
            AndroidImageProvisioningState::Ready => AndroidImageProvisioningStateDto::Ready,
            AndroidImageProvisioningState::Failed => AndroidImageProvisioningStateDto::Failed,
        },
        boot_attempts: assignment.boot_attempts,
        last_error: assignment.last_error.clone(),
    }
}

fn architecture_name(value: &HostArchitecture) -> &str {
    match value {
        HostArchitecture::X86_64 => "x86_64",
        HostArchitecture::Aarch64 => "aarch64",
        HostArchitecture::Unsupported(_) => "unsupported",
    }
}

const fn acceleration_name(value: AccelerationBackend) -> &'static str {
    match value {
        AccelerationBackend::Whpx => "whpx",
        AccelerationBackend::Kvm => "kvm",
        AccelerationBackend::Tcg => "tcg",
    }
}

const fn state_name(value: VmState) -> &'static str {
    match value {
        VmState::Created => "created",
        VmState::Stopped => "stopped",
        VmState::Starting => "starting",
        VmState::Running => "running",
        VmState::Pausing => "pausing",
        VmState::Paused => "paused",
        VmState::Resuming => "resuming",
        VmState::Stopping => "stopping",
        VmState::Error => "error",
    }
}


fn guest_template_summary(template: &GuestTemplate) -> GuestTemplateDto {
    GuestTemplateDto {
        id: template.id.clone(),
        family: template.family.code().to_owned(),
        family_label: template.family_label.clone(),
        product_id: template.product_id.clone(),
        product_label: template.product_label.clone(),
        release_id: template.release_id.clone(),
        release_label: template.release_label.clone(),
        profile_id: template.profile_id.clone(),
        profile_label: template.profile_label.clone(),
        guest_profile: match template.guest_profile {
            GuestProfileHint::Generic => GuestProfileDto::Generic,
            GuestProfileHint::Linux => GuestProfileDto::Linux,
            GuestProfileHint::Windows => GuestProfileDto::Windows,
            GuestProfileHint::Android => GuestProfileDto::Android,
        },
        architecture: template.architecture.clone(),
        source_kind: template.source_kind.code().to_owned(),
        source_label: template.source_label.clone(),
        firmware: template.firmware.code().to_owned(),
        recommended_vcpu_count: template.recommended.vcpu_count,
        recommended_memory_mib: template.recommended.memory_mib,
        recommended_disk_size_gib: template.recommended.disk_size_gib,
        installer_media: template.installer_media.as_ref().map(installer_media_source_summary),
        installer_media_options: template
            .installer_media_options
            .iter()
            .map(installer_media_source_summary)
            .collect(),
        description: template.description.clone(),
    }
}

fn installer_media_source_summary(media: &turkuazvm_guest_catalog::domain::guest_template::InstallerMediaSource) -> InstallerMediaSourceDto {
    InstallerMediaSourceDto {
        id: media.id.clone(),
        mode: media.mode.code().to_owned(),
        managed_download: media.managed_download_supported(),
        provider: media.provider.clone(),
        label: media.label.clone(),
        architecture: media.architecture.clone(),
        recommended: media.recommended,
        url: media.url.clone(),
        filename: media.filename.clone(),
        checksum_url: media.checksum_url.clone(),
        size_bytes: media.size_bytes,
        note: media.note.clone(),
    }
}

fn installer_media_download_summary(status: &InstallerMediaDownloadStatus) -> InstallerMediaDownloadDto {
    InstallerMediaDownloadDto {
        guest_template_id: status.guest_template_id.clone(),
        media_id: status.media_id.clone(),
        state: status.state.code().to_owned(),
        downloaded_bytes: status.downloaded_bytes,
        total_bytes: status.total_bytes,
        local_path: status.local_path.as_ref().map(|path| path.display().to_string()),
        sha256: status.sha256.clone(),
        detail: status.detail.clone(),
    }
}

const fn guest_profile_from_dto(value: GuestProfileDto) -> GuestProfile {
    match value {
        GuestProfileDto::Generic => GuestProfile::Generic,
        GuestProfileDto::Linux => GuestProfile::Linux,
        GuestProfileDto::Windows => GuestProfile::Windows,
        GuestProfileDto::Android => GuestProfile::Android,
    }
}

const fn guest_profile_from_hint(value: GuestProfileHint) -> GuestProfile {
    match value {
        GuestProfileHint::Generic => GuestProfile::Generic,
        GuestProfileHint::Linux => GuestProfile::Linux,
        GuestProfileHint::Windows => GuestProfile::Windows,
        GuestProfileHint::Android => GuestProfile::Android,
    }
}

fn android_input_from_dto(value: AndroidInputDto) -> AndroidInputAction {
    match value {
        AndroidInputDto::Tap { x, y } => AndroidInputAction::Tap { x, y },
        AndroidInputDto::Swipe { from_x, from_y, to_x, to_y, duration_ms } => {
            AndroidInputAction::Swipe { from_x, from_y, to_x, to_y, duration_ms }
        }
        AndroidInputDto::KeyEvent { key_code } => AndroidInputAction::KeyEvent { key_code },
        AndroidInputDto::Text { value } => AndroidInputAction::Text { value },
    }
}

fn gaming_input_profile_from_dto(value: GamingInputProfileConfigDto) -> Result<GameInputProfile, turkuazvm_gaming_input::domain::profile::GamingInputProfileError> {
    GameInputProfile::create(
        GamingVmId::parse(value.vm_id)?,
        value.package_name,
        VirtualJoystickProfile::create(
            value.joystick.enabled,
            normalized_point_from_dto(value.joystick.center)?,
            value.joystick.radius,
            value.joystick.pointer_id,
        )?,
        MouseLookProfile::create(
            value.mouse_look.enabled,
            normalized_point_from_dto(value.mouse_look.anchor)?,
            value.mouse_look.sensitivity_x_milli,
            value.mouse_look.sensitivity_y_milli,
            value.mouse_look.pointer_id,
        )?,
        value.bindings.into_iter().map(gaming_binding_from_dto).collect::<Result<Vec<_>, _>>()?,
    )
}

fn normalized_point_from_dto(value: NormalizedPointDto) -> Result<NormalizedPoint, turkuazvm_gaming_input::domain::profile::GamingInputProfileError> {
    NormalizedPoint::create(value.x, value.y)
}

fn gaming_binding_from_dto(value: GamingInputBindingDto) -> Result<InputBinding, turkuazvm_gaming_input::domain::profile::GamingInputProfileError> {
    let source = match value.source {
        GamingInputSourceDto::Key { key } => InputSource::Key(gaming_key_from_dto(key)),
        GamingInputSourceDto::MouseButton { button } => InputSource::MouseButton(gaming_mouse_button_from_dto(button)),
        GamingInputSourceDto::GamepadButton { button } => InputSource::GamepadButton(button),
    };
    let target = match value.target {
        GamingInputTargetDto::Tap { position } => InputTarget::Tap { position: normalized_point_from_dto(position)? },
        GamingInputTargetDto::HoldTouch { pointer_id, position } => InputTarget::HoldTouch {
            pointer_id,
            position: normalized_point_from_dto(position)?,
        },
        GamingInputTargetDto::AndroidKey { key_code } => InputTarget::AndroidKey { key_code },
    };
    Ok(InputBinding::new(source, target))
}

fn gaming_input_profile_summary(profile: &GameInputProfile) -> GamingInputProfileDto {
    GamingInputProfileDto {
        vm_id: profile.vm_id().as_str().to_owned(),
        package_name: profile.package_name().map(str::to_owned),
        joystick: VirtualJoystickDto {
            enabled: profile.joystick().enabled(),
            center: normalized_point_summary(profile.joystick().center()),
            radius: profile.joystick().radius(),
            pointer_id: profile.joystick().pointer_id(),
        },
        mouse_look: MouseLookDto {
            enabled: profile.mouse_look().enabled(),
            anchor: normalized_point_summary(profile.mouse_look().anchor()),
            sensitivity_x_milli: profile.mouse_look().sensitivity_x_milli(),
            sensitivity_y_milli: profile.mouse_look().sensitivity_y_milli(),
            pointer_id: profile.mouse_look().pointer_id(),
        },
        bindings: profile.bindings().iter().map(gaming_binding_summary).collect(),
    }
}

const fn normalized_point_summary(value: NormalizedPoint) -> NormalizedPointDto {
    NormalizedPointDto { x: value.x(), y: value.y() }
}

fn gaming_binding_summary(value: &InputBinding) -> GamingInputBindingDto {
    let source = match value.source() {
        InputSource::Key(key) => GamingInputSourceDto::Key { key: gaming_key_summary(key) },
        InputSource::MouseButton(button) => GamingInputSourceDto::MouseButton { button: gaming_mouse_button_summary(button) },
        InputSource::GamepadButton(button) => GamingInputSourceDto::GamepadButton { button },
    };
    let target = match value.target() {
        InputTarget::Tap { position } => GamingInputTargetDto::Tap { position: normalized_point_summary(*position) },
        InputTarget::HoldTouch { pointer_id, position } => GamingInputTargetDto::HoldTouch {
            pointer_id: *pointer_id,
            position: normalized_point_summary(*position),
        },
        InputTarget::AndroidKey { key_code } => GamingInputTargetDto::AndroidKey { key_code: *key_code },
    };
    GamingInputBindingDto { source, target }
}

fn guest_agent_status_summary(value: &AndroidGuestAgentReport) -> GuestAgentStatusDto {
    GuestAgentStatusDto {
        available: value.available,
        persistent_multi_touch: value.persistent_multi_touch,
        max_contacts: value.max_contacts,
        continuation_api: value.continuation_api,
        input_backend: value.input_backend.clone(),
        endpoint: value.endpoint.clone(),
        secure_transport_v2: value.secure_transport_v2,
        clipboard: value.clipboard,
    }
}

fn game_catalog_entry_summary(game: &GameDefinition) -> GameCatalogEntryDto {
    GameCatalogEntryDto {
        id: game.id.as_str().to_owned(),
        name: game.name.clone(),
        packages: game.packages.iter().map(|value| value.as_str().to_owned()).collect(),
        maturity: match game.maturity {
            CatalogMaturity::Experimental => GameCatalogMaturityDto::Experimental,
            CatalogMaturity::Playable => GameCatalogMaturityDto::Playable,
            CatalogMaturity::Recommended => GameCatalogMaturityDto::Recommended,
        },
        emulator_disclosure_required: game.emulator_disclosure_required,
        minimum_sdk: game.requirements.minimum_sdk,
        persistent_multi_touch_required: game.requirements.persistent_multi_touch,
        relative_mouse_look_required: game.requirements.relative_mouse_look,
        preferred_gpu_backends: game.requirements.preferred_gpu_backends.iter().copied().map(game_catalog_gpu_backend_summary).collect(),
        width: game.android.width,
        height: game.android.height,
        density_dpi: game.android.density_dpi,
        target_fps: game.android.target_fps,
    }
}

const fn compatibility_status_summary(value: CompatibilityStatus) -> GameCompatibilityStatusDto {
    match value {
        CompatibilityStatus::Blocked => GameCompatibilityStatusDto::Blocked,
        CompatibilityStatus::Experimental => GameCompatibilityStatusDto::Experimental,
        CompatibilityStatus::Playable => GameCompatibilityStatusDto::Playable,
        CompatibilityStatus::Recommended => GameCompatibilityStatusDto::Recommended,
    }
}

const fn catalog_gpu_backend(value: GpuBackend) -> CatalogGpuBackend {
    match value {
        GpuBackend::Software => CatalogGpuBackend::Software,
        GpuBackend::Virtio2d => CatalogGpuBackend::Virtio2d,
        GpuBackend::VirglVenus => CatalogGpuBackend::VirglVenus,
        GpuBackend::Gfxstream => CatalogGpuBackend::Gfxstream,
    }
}

const fn game_catalog_gpu_backend_summary(value: CatalogGpuBackend) -> GameCatalogGpuBackendDto {
    match value {
        CatalogGpuBackend::Software => GameCatalogGpuBackendDto::Software,
        CatalogGpuBackend::Virtio2d => GameCatalogGpuBackendDto::Virtio2d,
        CatalogGpuBackend::VirglVenus => GameCatalogGpuBackendDto::VirglVenus,
        CatalogGpuBackend::Gfxstream => GameCatalogGpuBackendDto::Gfxstream,
    }
}

fn game_input_profile_from_definition(vm_id: &str, game: &GameDefinition) -> Result<GameInputProfile, turkuazvm_gaming_input::domain::profile::GamingInputProfileError> {
    let joystick = &game.input.joystick;
    let mouse = &game.input.mouse_look;
    GameInputProfile::create(
        GamingVmId::parse(vm_id.to_owned())?,
        if game.packages.len() == 1 { game.packages.first().map(|value| value.as_str().to_owned()) } else { None },
        VirtualJoystickProfile::create(joystick.enabled, NormalizedPoint::create(joystick.center.x, joystick.center.y)?, joystick.radius, joystick.pointer_id)?,
        MouseLookProfile::create(mouse.enabled, NormalizedPoint::create(mouse.anchor.x, mouse.anchor.y)?, mouse.sensitivity_x_milli, mouse.sensitivity_y_milli, mouse.pointer_id)?,
        game.input.bindings.iter().map(catalog_binding_to_input).collect::<Result<Vec<_>, _>>()?,
    )
}

fn catalog_binding_to_input(value: &turkuazvm_game_catalog::domain::game::CatalogInputBinding) -> Result<InputBinding, turkuazvm_gaming_input::domain::profile::GamingInputProfileError> {
    let source = match &value.source {
        CatalogInputSource::Key(key) => InputSource::Key(catalog_key(*key)),
        CatalogInputSource::MouseButton(button) => InputSource::MouseButton(catalog_mouse_button(*button)),
        CatalogInputSource::GamepadButton(button) => InputSource::GamepadButton(*button),
    };
    let target = match &value.target {
        CatalogInputTarget::Tap(position) => InputTarget::Tap { position: NormalizedPoint::create(position.x, position.y)? },
        CatalogInputTarget::HoldTouch { pointer_id, position } => InputTarget::HoldTouch { pointer_id: *pointer_id, position: NormalizedPoint::create(position.x, position.y)? },
        CatalogInputTarget::AndroidKey(key_code) => InputTarget::AndroidKey { key_code: *key_code },
    };
    Ok(InputBinding::new(source, target))
}

const fn catalog_key(value: CatalogKey) -> GamingKey {
    match value {
        CatalogKey::W => GamingKey::W, CatalogKey::A => GamingKey::A, CatalogKey::S => GamingKey::S, CatalogKey::D => GamingKey::D,
        CatalogKey::Space => GamingKey::Space, CatalogKey::C => GamingKey::C, CatalogKey::Z => GamingKey::Z, CatalogKey::R => GamingKey::R,
        CatalogKey::F => GamingKey::F, CatalogKey::Q => GamingKey::Q, CatalogKey::E => GamingKey::E, CatalogKey::ShiftLeft => GamingKey::ShiftLeft,
        CatalogKey::ControlLeft => GamingKey::ControlLeft, CatalogKey::Digit1 => GamingKey::Digit1, CatalogKey::Digit2 => GamingKey::Digit2,
        CatalogKey::Digit3 => GamingKey::Digit3, CatalogKey::Digit4 => GamingKey::Digit4, CatalogKey::Tab => GamingKey::Tab, CatalogKey::Escape => GamingKey::Escape,
    }
}

const fn catalog_mouse_button(value: CatalogMouseButton) -> GamingMouseButton {
    match value { CatalogMouseButton::Left => GamingMouseButton::Left, CatalogMouseButton::Right => GamingMouseButton::Right, CatalogMouseButton::Middle => GamingMouseButton::Middle }
}

fn gaming_input_capabilities_summary(value: GamingInputCapabilities) -> GamingInputCapabilitiesDto {
    GamingInputCapabilitiesDto {
        profile_editor: value.profile_editor,
        raw_keyboard: value.raw_keyboard,
        raw_mouse_motion: value.raw_mouse_motion,
        adb_single_touch_fallback: value.adb_single_touch_fallback,
        persistent_multi_touch: value.persistent_multi_touch,
        virtual_joystick: value.virtual_joystick,
        relative_mouse_look: value.relative_mouse_look,
        host_gamepad_observation: value.host_gamepad_observation,
    }
}

fn gaming_input_event_from_dto(value: GamingInputEventDto) -> GamingInputEvent {
    match value {
        GamingInputEventDto::Key { key, pressed } => GamingInputEvent::Key { key: gaming_key_from_dto(key), pressed },
        GamingInputEventDto::MouseButton { button, pressed } => GamingInputEvent::MouseButton {
            button: gaming_mouse_button_from_dto(button), pressed,
        },
        GamingInputEventDto::MouseMotion { delta_x, delta_y } => GamingInputEvent::MouseMotion { delta_x, delta_y },
        GamingInputEventDto::MouseCapture { captured } => GamingInputEvent::MouseCapture { captured },
        GamingInputEventDto::GamepadButton { button, pressed } => GamingInputEvent::GamepadButton { button, pressed },
        GamingInputEventDto::GamepadAxis { axis, value_milli } => GamingInputEvent::GamepadAxis { axis, value_milli },
    }
}

const fn gaming_key_from_dto(value: GamingKeyDto) -> GamingKey {
    match value {
        GamingKeyDto::W => GamingKey::W, GamingKeyDto::A => GamingKey::A, GamingKeyDto::S => GamingKey::S, GamingKeyDto::D => GamingKey::D,
        GamingKeyDto::Space => GamingKey::Space, GamingKeyDto::C => GamingKey::C, GamingKeyDto::Z => GamingKey::Z, GamingKeyDto::R => GamingKey::R,
        GamingKeyDto::F => GamingKey::F, GamingKeyDto::Q => GamingKey::Q, GamingKeyDto::E => GamingKey::E, GamingKeyDto::ShiftLeft => GamingKey::ShiftLeft,
        GamingKeyDto::ControlLeft => GamingKey::ControlLeft, GamingKeyDto::Digit1 => GamingKey::Digit1, GamingKeyDto::Digit2 => GamingKey::Digit2,
        GamingKeyDto::Digit3 => GamingKey::Digit3, GamingKeyDto::Digit4 => GamingKey::Digit4, GamingKeyDto::Tab => GamingKey::Tab,
        GamingKeyDto::Escape => GamingKey::Escape,
    }
}

const fn gaming_key_summary(value: GamingKey) -> GamingKeyDto {
    match value {
        GamingKey::W => GamingKeyDto::W, GamingKey::A => GamingKeyDto::A, GamingKey::S => GamingKeyDto::S, GamingKey::D => GamingKeyDto::D,
        GamingKey::Space => GamingKeyDto::Space, GamingKey::C => GamingKeyDto::C, GamingKey::Z => GamingKeyDto::Z, GamingKey::R => GamingKeyDto::R,
        GamingKey::F => GamingKeyDto::F, GamingKey::Q => GamingKeyDto::Q, GamingKey::E => GamingKeyDto::E, GamingKey::ShiftLeft => GamingKeyDto::ShiftLeft,
        GamingKey::ControlLeft => GamingKeyDto::ControlLeft, GamingKey::Digit1 => GamingKeyDto::Digit1, GamingKey::Digit2 => GamingKeyDto::Digit2,
        GamingKey::Digit3 => GamingKeyDto::Digit3, GamingKey::Digit4 => GamingKeyDto::Digit4, GamingKey::Tab => GamingKeyDto::Tab,
        GamingKey::Escape => GamingKeyDto::Escape,
    }
}

const fn gaming_mouse_button_from_dto(value: GamingMouseButtonDto) -> GamingMouseButton {
    match value {
        GamingMouseButtonDto::Left => GamingMouseButton::Left,
        GamingMouseButtonDto::Right => GamingMouseButton::Right,
        GamingMouseButtonDto::Middle => GamingMouseButton::Middle,
    }
}

const fn gaming_mouse_button_summary(value: GamingMouseButton) -> GamingMouseButtonDto {
    match value {
        GamingMouseButton::Left => GamingMouseButtonDto::Left,
        GamingMouseButton::Right => GamingMouseButtonDto::Right,
        GamingMouseButton::Middle => GamingMouseButtonDto::Middle,
    }
}

const fn android_connection_state_name(value: AndroidConnectionState) -> &'static str {
    match value {
        AndroidConnectionState::Disconnected => "disconnected",
        AndroidConnectionState::Offline => "offline",
        AndroidConnectionState::Unauthorized => "unauthorized",
        AndroidConnectionState::Booting => "booting",
        AndroidConnectionState::Ready => "ready",
    }
}

const fn guest_profile_name(value: GuestProfile) -> &'static str {
    match value {
        GuestProfile::Generic => "generic",
        GuestProfile::Linux => "linux",
        GuestProfile::Windows => "windows",
        GuestProfile::Android => "android",
    }
}
