// # 📄 Dosya Yolu: /turkuazvm/apps/engine/src/services/android_image_application_service.rs
// # 📌 Amac: Android Image bounded context ile VM bounded contextini Engine seviyesinde orkestre eder
// # 📌 Modul - Rust
// # Version: 0.41.6
// # Aciklama: Android SDK provider configini distribution ve runtime media Tool katmanlarina tasir; resolver/cache ve VM assignment akislarini compose eder
// # Bagimli Oldugu Katman: Service | Repo | Tool

use turkuazvm_android::domain::runtime_profile::AndroidRuntimeProfile;
use turkuazvm_android_image::commands::android_image_commands::{
    AssignAndroidImageCommand, CancelAndroidImageDistributionCommand, CleanupAndroidImageDistributionCommand,
    DefineAndroidImageCommand, InstallAndroidImageDistributionCommand, PrepareAndroidImageBuildCommand,
    RegisterAndroidImageBuildCommand,
};
use turkuazvm_android_image::domain::distribution_source::{
    AndroidDistributionChannelPolicy, AndroidDistributionResolverPolicy,
};
use turkuazvm_android_image::domain::image::{AndroidImage, AndroidImageAssignment};
use turkuazvm_android_image::ports::android_image_builder_port::AndroidImageBuildPlan;
use turkuazvm_android_image::ports::android_image_distribution_port::AndroidImageDistributionProgress;
use turkuazvm_android_image::services::android_distribution_source_resolver_service::AndroidDistributionSourceResolverService;
use turkuazvm_android_image::services::android_image_service::AndroidImageService;
use turkuazvm_core::domain::guest_boot::GuestProfile;
use turkuazvm_core::domain::runtime_media::VmRuntimeMediaPlan;
use turkuazvm_core::domain::vm_state::VmState;
use turkuazvm_core::services::vm_query_service::VmQueryService;
use turkuazvm_guest::tools::aosp_android_image_tool::{
    AospAndroidImageSettings, AospAndroidImageTool,
};
use turkuazvm_guest::tools::android_ci_distribution_tool::{
    AndroidCiDistributionSettings, AndroidCiDistributionTool, SharedAndroidDistributionSourceResolver,
};
use turkuazvm_guest::tools::android_ci_source_provider_tool::{
    AndroidCiSourceProviderSettings, AndroidCiSourceProviderTool,
};
use turkuazvm_guest::tools::android_distribution_router_tool::{
    AndroidDistributionProvider, AndroidDistributionRouterTool,
};
use turkuazvm_guest::tools::android_runtime_media_tool::{
    AndroidRuntimeMediaError, AndroidRuntimeMediaSettings, AndroidRuntimeMediaTool,
};
use turkuazvm_guest::tools::android_sdk_distribution_tool::{
    AndroidSdkDistributionSettings, AndroidSdkDistributionTool,
};
use turkuazvm_repositories::repositories::shared_vm_repository::SharedVmRepository;
use turkuazvm_repositories::repositories::yaml_android_distribution_source_cache_repository::YamlAndroidDistributionSourceCacheRepository;
use turkuazvm_repositories::repositories::yaml_android_image_repository::YamlAndroidImageRepository;
use turkuazvm_repositories::repositories::yaml_vm_repository::YamlVmRepository;

use crate::config::engine_config::{
    AndroidDistributionProviderEngineConfig, AndroidImageEngineConfig, ArtifactCacheEngineConfig,
    DownloadHttpEngineConfig,
};
use crate::services::artifact_cache_application_service::SharedArtifactCacheClient;

type EngineRepository = SharedVmRepository<YamlVmRepository>;
type ImageService = AndroidImageService<YamlAndroidImageRepository, AospAndroidImageTool, AndroidDistributionRouterTool>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndroidImageApplicationError {
    Image(String),
    Vm(String),
    VmProfileMustBeAndroid,
    VmMustBeStopped(VmState),
    AssignmentRequired,
    RuntimeMedia(AndroidRuntimeMediaError),
}

pub struct AndroidImageApplicationService {
    image_service: ImageService,
    query_service: VmQueryService<EngineRepository>,
    runtime_media_tool: AndroidRuntimeMediaTool,
}

impl AndroidImageApplicationService {
    pub fn new(
        settings: AndroidImageEngineConfig,
        download_http: DownloadHttpEngineConfig,
        artifact_cache: ArtifactCacheEngineConfig,
        artifact_cache_client: Option<SharedArtifactCacheClient>,
        repository: EngineRepository,
        data_root: std::path::PathBuf,
        vm_root: std::path::PathBuf,
        qemu_img_binary: Option<std::path::PathBuf>,
    ) -> Self {
        let image_repository = YamlAndroidImageRepository::new(data_root.clone());
        let image_output_root = settings.output_root.clone();
        let android_sdk_tool_root = settings.sdk.tool_root.clone();
        let emulator_console_port_min = settings.sdk.emulator_console_port_min;
        let emulator_console_port_max = settings.sdk.emulator_console_port_max;
        let android_sdk = AndroidSdkDistributionTool::new(AndroidSdkDistributionSettings {
            output_root: settings.output_root.clone(),
            tool_root: settings.sdk.tool_root.clone(),
            curl_binary: download_http.curl_binary.clone(),
            minimum_free_disk_gib: settings.distribution_minimum_free_disk_gib,
            download_retry_count: download_http.retry_count,
            download_retry_delay_seconds: download_http.retry_delay_seconds,
            download_connect_timeout_seconds: download_http.connect_timeout_seconds,
            repository_base_url: settings.sdk.repository_base_url.clone(),
            package_index_url: settings.sdk.package_index_url.clone(),
            emulator_package_path: settings.sdk.emulator_package_path.clone(),
            architecture: settings.sdk.architecture.clone(),
            variant_priority: settings.sdk.variant_priority.clone(),
            system_image_indexes: settings.sdk.system_image_indexes.clone(),
            api_levels: settings.sdk.api_levels.clone(),
        });
        let builder = AospAndroidImageTool::new(AospAndroidImageSettings {
            source_root: settings.source_root,
            output_root: settings.output_root.clone(),
            build_script: settings.build_script,
        });

        let resolver_policy = AndroidDistributionResolverPolicy {
            primary_base_url: settings.distribution_base_url,
            official_base_url: settings.distribution_official_base_url,
            use_official_fallback: settings.distribution_official_fallback,
            branch_templates: settings.distribution_branch_templates,
            target_candidates: settings.distribution_target_candidates,
            default_branch: settings.distribution_branch,
            default_target: settings.distribution_target,
            channels: settings
                .distribution_channels
                .into_iter()
                .map(|(release, channel)| {
                    (
                        release,
                        AndroidDistributionChannelPolicy {
                            expected_sdk: channel.expected_sdk,
                            allow_device_bootloader_fallback: channel.allow_device_bootloader_fallback,
                            branch_hints: channel.branch_hints,
                        },
                    )
                })
                .collect(),
        };
        let source_provider = AndroidCiSourceProviderTool::new(AndroidCiSourceProviderSettings {
            curl_binary: download_http.curl_binary.clone(),
            connect_timeout_seconds: download_http.connect_timeout_seconds,
            retry_count: download_http.retry_count,
            retry_delay_seconds: download_http.retry_delay_seconds,
        });
        let source_cache = YamlAndroidDistributionSourceCacheRepository::new(settings.source_cache_path);
        let source_resolver: SharedAndroidDistributionSourceResolver = std::sync::Arc::new(
            std::sync::Mutex::new(Box::new(AndroidDistributionSourceResolverService::new(
                resolver_policy,
                source_provider,
                source_cache,
            ))),
        );

        let distribution_settings = AndroidCiDistributionSettings {
            output_root: settings.output_root.clone(),
            curl_binary: download_http.curl_binary.clone(),
            minimum_free_disk_gib: settings.distribution_minimum_free_disk_gib,
            download_retry_count: download_http.retry_count,
            download_retry_delay_seconds: download_http.retry_delay_seconds,
            download_connect_timeout_seconds: download_http.connect_timeout_seconds,
        };
        let android_ci = match artifact_cache_client {
            Some(cache_client) if artifact_cache.enabled => AndroidCiDistributionTool::with_artifact_cache(
                distribution_settings,
                std::sync::Arc::clone(&source_resolver),
                cache_client,
            ),
            _ => AndroidCiDistributionTool::new(
                distribution_settings,
                std::sync::Arc::clone(&source_resolver),
            ),
        };
        let distribution_providers = settings
            .distribution_providers
            .into_iter()
            .map(|(release, provider)| {
                let provider = match provider {
                    AndroidDistributionProviderEngineConfig::AndroidSdk => AndroidDistributionProvider::AndroidSdk,
                    AndroidDistributionProviderEngineConfig::AndroidCi => AndroidDistributionProvider::AndroidCi,
                    AndroidDistributionProviderEngineConfig::SourceBuild => AndroidDistributionProvider::SourceBuild,
                };
                (release, provider)
            })
            .collect();
        let distribution = AndroidDistributionRouterTool::new(android_ci, android_sdk, distribution_providers);
        let image_service = AndroidImageService::new(image_repository, builder, distribution);
        let _ = image_service.recover_interrupted_distribution_installs();
        Self {
            image_service,
            query_service: VmQueryService::new(repository),
            runtime_media_tool: AndroidRuntimeMediaTool::new(AndroidRuntimeMediaSettings {
                data_root,
                vm_root,
                image_output_root,
                qemu_img_binary,
                android_sdk_tool_root,
                emulator_console_port_min,
                emulator_console_port_max,
            }),
        }
    }

    pub fn list(&self) -> Result<Vec<AndroidImage>, AndroidImageApplicationError> {
        self.image_service
            .list()
            .map_err(|error| AndroidImageApplicationError::Image(format!("{error:?}")))
    }

    pub fn install_progress(
        &self,
        image_id: &str,
    ) -> Result<Option<AndroidImageDistributionProgress>, AndroidImageApplicationError> {
        self.image_service
            .distribution_progress(image_id)
            .map_err(|error| AndroidImageApplicationError::Image(format!("{error:?}")))
    }

    pub fn define(
        &self,
        image_id: String,
        name: String,
        requested_release: Option<String>,
    ) -> Result<AndroidImage, AndroidImageApplicationError> {
        self.image_service
            .define(DefineAndroidImageCommand { image_id, name, requested_release })
            .map_err(|error| AndroidImageApplicationError::Image(format!("{error:?}")))
    }

    pub fn prepare_build(
        &self,
        image_id: String,
    ) -> Result<AndroidImageBuildPlan, AndroidImageApplicationError> {
        self.image_service
            .prepare_build(PrepareAndroidImageBuildCommand { image_id })
            .map_err(|error| AndroidImageApplicationError::Image(format!("{error:?}")))
    }

    pub fn register_build(
        &self,
        image_id: String,
    ) -> Result<AndroidImage, AndroidImageApplicationError> {
        self.image_service
            .register_build(RegisterAndroidImageBuildCommand { image_id })
            .map_err(|error| AndroidImageApplicationError::Image(format!("{error:?}")))
    }

    pub fn install_distribution(
        &self,
        image_id: String,
    ) -> Result<AndroidImage, AndroidImageApplicationError> {
        let image = self.image_service
            .begin_distribution_install(InstallAndroidImageDistributionCommand { image_id: image_id.clone() })
            .map_err(|error| AndroidImageApplicationError::Image(format!("{error:?}")))?;
        let service = self.image_service.clone();
        std::thread::spawn(move || {
            let _ = service.run_distribution_install(image_id);
        });
        Ok(image)
    }

    pub fn cancel_distribution(
        &self,
        image_id: String,
    ) -> Result<AndroidImage, AndroidImageApplicationError> {
        self.image_service
            .cancel_distribution_install(CancelAndroidImageDistributionCommand { image_id })
            .map_err(|error| AndroidImageApplicationError::Image(format!("{error:?}")))
    }

    pub fn cleanup_distribution(
        &self,
        image_id: String,
    ) -> Result<AndroidImage, AndroidImageApplicationError> {
        self.image_service
            .cleanup_distribution_install(CleanupAndroidImageDistributionCommand { image_id })
            .map_err(|error| AndroidImageApplicationError::Image(format!("{error:?}")))
    }

    pub fn assign(
        &self,
        vm_id: String,
        image_id: String,
    ) -> Result<AndroidImageAssignment, AndroidImageApplicationError> {
        let machine = self
            .query_service
            .get(&vm_id)
            .map_err(|error| AndroidImageApplicationError::Vm(format!("{error:?}")))?;
        if machine.guest_boot().profile() != GuestProfile::Android {
            return Err(AndroidImageApplicationError::VmProfileMustBeAndroid);
        }
        if machine.state() != VmState::Stopped {
            return Err(AndroidImageApplicationError::VmMustBeStopped(machine.state()));
        }
        self.image_service
            .assign(AssignAndroidImageCommand { vm_id, image_id })
            .map_err(|error| AndroidImageApplicationError::Image(format!("{error:?}")))
    }

    pub fn prepare_runtime_media(&self, vm_id: &str, profile: &AndroidRuntimeProfile) -> Result<VmRuntimeMediaPlan, AndroidImageApplicationError> {
        let assignment = self
            .image_service
            .assignment(vm_id)
            .map_err(|error| AndroidImageApplicationError::Image(format!("{error:?}")))?
            .ok_or(AndroidImageApplicationError::AssignmentRequired)?;
        let image = self
            .image_service
            .get(assignment.image_id.as_str())
            .map_err(|error| AndroidImageApplicationError::Image(format!("{error:?}")))?;
        self.runtime_media_tool
            .prepare(&image, vm_id, profile)
            .map_err(AndroidImageApplicationError::RuntimeMedia)
    }

    pub fn begin_boot_attempt(&self, vm_id: &str) -> Result<AndroidImageAssignment, AndroidImageApplicationError> {
        self.image_service
            .begin_boot_attempt(vm_id)
            .map_err(|error| AndroidImageApplicationError::Image(format!("{error:?}")))
    }

    pub fn mark_ready(&self, vm_id: &str) -> Result<AndroidImageAssignment, AndroidImageApplicationError> {
        self.image_service
            .mark_assignment_ready(vm_id)
            .map_err(|error| AndroidImageApplicationError::Image(format!("{error:?}")))
    }

    pub fn mark_failed(&self, vm_id: &str, error: String) -> Result<AndroidImageAssignment, AndroidImageApplicationError> {
        self.image_service
            .mark_assignment_failed(vm_id, error)
            .map_err(|error| AndroidImageApplicationError::Image(format!("{error:?}")))
    }

    pub fn assignment_requires_guest_agent(&self, vm_id: &str) -> Result<bool, AndroidImageApplicationError> {
        let assignment = self.image_service
            .assignment(vm_id)
            .map_err(|error| AndroidImageApplicationError::Image(format!("{error:?}")))?
            .ok_or(AndroidImageApplicationError::AssignmentRequired)?;
        let image = self.image_service
            .get(assignment.image_id.as_str())
            .map_err(|error| AndroidImageApplicationError::Image(format!("{error:?}")))?;
        Ok(image.capabilities.guest_agent_included)
    }

    pub fn assignment(
        &self,
        vm_id: &str,
    ) -> Result<Option<AndroidImageAssignment>, AndroidImageApplicationError> {
        self.image_service
            .assignment(vm_id)
            .map_err(|error| AndroidImageApplicationError::Image(format!("{error:?}")))
    }
}
