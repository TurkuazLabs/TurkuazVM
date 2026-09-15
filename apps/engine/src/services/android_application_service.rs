// # 📄 Dosya Yolu: /turkuazvm/apps/engine/src/services/android_application_service.rs
// # 📌 Amac: Android bounded context ile VM/network bounded contextlerini Engine seviyesinde orkestre eder
// # 📌 Modul - Rust
// # Version: 0.40.14
// # Aciklama: Android VM preflight ve ADB port tahsisini merkezi config araligina baglar; profile, package, display ve input use-case'lerini birlestirir
// # Bagimli Oldugu Katman: Service | Repo | Tool

use std::collections::HashSet;
use std::net::{SocketAddrV4, TcpListener};

use turkuazvm_android::commands::android_input_command::AndroidInputCommand;
use turkuazvm_android::commands::configure_android_runtime_command::ConfigureAndroidRuntimeCommand;
use turkuazvm_android::commands::install_apk_command::InstallApkCommand;
use turkuazvm_android::commands::package_action_command::PackageActionCommand;
use turkuazvm_android::domain::device::{AndroidBridgeCapabilities, AndroidDeviceReport};
use turkuazvm_android::domain::guest_agent::{AndroidGuestAgentReport, AndroidTouchContact};
use turkuazvm_android::domain::input::AndroidInputAction;
use turkuazvm_android::domain::package::AndroidPackageInfo;
use turkuazvm_android::domain::runtime_profile::{AndroidRuntimeProfile, AndroidVmId};
use turkuazvm_android::ports::android_guest_agent_port::AndroidGuestAgentPort;
use turkuazvm_android::ports::android_profile_repository_port::{AndroidProfileRepositoryError, AndroidProfileRepositoryPort};
use turkuazvm_android::services::android_input_service::AndroidInputService;
use turkuazvm_android::services::android_package_service::AndroidPackageService;
use turkuazvm_android::services::android_runtime_service::AndroidRuntimeService;
use turkuazvm_core::commands::attach_network_command::{AttachNetworkCommand, PortForwardCommand};
use turkuazvm_core::commands::detach_network_command::DetachNetworkCommand;
use turkuazvm_core::domain::guest_boot::GuestProfile;
use turkuazvm_core::domain::network::{NetworkDeviceModel, NetworkMode, PortProtocol};
use turkuazvm_core::domain::virtual_machine::VirtualMachine;
use turkuazvm_core::domain::vm_state::VmState;
use turkuazvm_core::services::network_service::NetworkService;
use turkuazvm_core::services::vm_query_service::VmQueryService;
use turkuazvm_guest::tools::adb_runtime_tool::{AdbRuntimeSettings, AdbRuntimeTool};
use turkuazvm_guest::tools::android_guest_agent_tool::{
    AndroidGuestAgentSettings, AndroidGuestAgentTool, AndroidGuestAgentUpdateSettings,
};
use turkuazvm_platform::tools::native_network_tool::NativeNetworkTool;
use turkuazvm_repositories::repositories::shared_vm_repository::SharedVmRepository;
use turkuazvm_repositories::repositories::yaml_android_profile_repository::YamlAndroidProfileRepository;
use turkuazvm_repositories::repositories::yaml_vm_repository::YamlVmRepository;

use crate::config::engine_config::AndroidEngineConfig;

const ANDROID_NETWORK_ID: &str = "android-nat";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndroidApplicationError {
    Vm(String),
    VmMustBeStopped(VmState),
    VmMustBeRunning(VmState),
    VmProfileMustBeAndroid,
    AdbPortExhausted,
    AndroidNetworkMismatch,
    Network(String),
    Runtime(String),
    Package(String),
    Input(String),
    GuestAgent(String),
}

type EngineRepository = SharedVmRepository<YamlVmRepository>;
type RuntimeService = AndroidRuntimeService<YamlAndroidProfileRepository, AdbRuntimeTool>;
type PackageService = AndroidPackageService<YamlAndroidProfileRepository, AdbRuntimeTool>;
type InputService = AndroidInputService<YamlAndroidProfileRepository, AdbRuntimeTool>;
type EngineNetworkService = NetworkService<EngineRepository, NativeNetworkTool>;

pub struct AndroidApplicationService {
    query_service: VmQueryService<EngineRepository>,
    network_service: EngineNetworkService,
    profile_repository: YamlAndroidProfileRepository,
    runtime_service: RuntimeService,
    package_service: PackageService,
    input_service: InputService,
    guest_agent: AndroidGuestAgentTool,
    settings: AndroidEngineConfig,
}

impl AndroidApplicationService {
    pub fn new(
        settings: AndroidEngineConfig,
        repository: EngineRepository,
        network_tool: NativeNetworkTool,
        data_root: std::path::PathBuf,
    ) -> Self {
        let profile_repository = YamlAndroidProfileRepository::new(data_root);
        let adb = AdbRuntimeTool::new(AdbRuntimeSettings {
            explicit_binary: settings.adb_binary.clone(),
            command_timeout: settings.command_timeout,
            ready_timeout: settings.ready_timeout,
            ready_poll_interval: settings.ready_poll_interval,
        });
        let guest_agent = AndroidGuestAgentTool::new(AndroidGuestAgentSettings {
            explicit_adb_binary: settings.adb_binary.clone(),
            host_ip: settings.guest_agent.host_ip,
            host_port_min: settings.guest_agent.host_port_min,
            host_port_max: settings.guest_agent.host_port_max,
            guest_port: settings.guest_agent.guest_port,
            command_timeout: settings.command_timeout,
            connect_timeout: settings.guest_agent.connect_timeout,
            io_timeout: settings.guest_agent.io_timeout,
            ready_timeout: settings.guest_agent.ready_timeout,
            ready_poll_interval: settings.guest_agent.ready_poll_interval,
            secret_root: settings.guest_agent.secret_root.clone(),
            require_adb_root_for_provisioning: settings.guest_agent.require_adb_root_for_provisioning,
            update: AndroidGuestAgentUpdateSettings {
                enabled: settings.guest_agent.update.enabled,
                manifest_path: settings.guest_agent.update.manifest_path.clone(),
                trusted_public_key_hex: settings.guest_agent.update.trusted_public_key_hex.clone(),
                trusted_apk_cert_sha256: settings.guest_agent.update.trusted_apk_cert_sha256.clone(),
                apksigner_binary: settings.guest_agent.update.apksigner_binary.clone(),
                rollback_root: settings.guest_agent.update.rollback_root.clone(),
            },
        });
        Self {
            query_service: VmQueryService::new(repository.clone()),
            network_service: NetworkService::new(repository, network_tool),
            profile_repository: profile_repository.clone(),
            runtime_service: AndroidRuntimeService::new(profile_repository.clone(), adb.clone()),
            package_service: AndroidPackageService::new(
                profile_repository.clone(),
                adb.clone(),
                settings.package_root.clone(),
            ),
            input_service: AndroidInputService::new(profile_repository.clone(), adb),
            guest_agent,
            settings,
        }
    }

    pub fn capabilities(&self) -> AndroidBridgeCapabilities {
        self.runtime_service.capabilities()
    }

    pub fn configure_runtime(
        &mut self,
        vm_id: String,
        width: Option<u32>,
        height: Option<u32>,
        density_dpi: Option<u32>,
        target_fps: Option<u16>,
    ) -> Result<AndroidRuntimeProfile, AndroidApplicationError> {
        let machine = self.require_android_vm(&vm_id)?;
        Self::require_stopped(&machine)?;

        let existing_profile = self
            .profile_repository
            .list()
            .map_err(|error| {
                AndroidApplicationError::Runtime(format!("profile_list_failed: {error:?}"))
            })?
            .into_iter()
            .find(|profile| profile.vm_id().as_str() == vm_id);
        let existing_network_port = self.android_network_host_port(&machine)?;
        let host_port = match (existing_profile.as_ref(), existing_network_port) {
            (Some(profile), Some(network_port)) if profile.adb_host_port() == network_port => {
                network_port
            }
            (Some(_), Some(_)) => return Err(AndroidApplicationError::AndroidNetworkMismatch),
            (Some(profile), None) => profile.adb_host_port(),
            (None, Some(network_port)) => network_port,
            (None, None) => self.allocate_adb_port()?,
        };

        let network_was_added = if existing_network_port.is_some() {
            false
        } else {
            self.network_service
                .attach_network(AttachNetworkCommand::new_user_nat(
                    vm_id.clone(),
                    ANDROID_NETWORK_ID,
                    NetworkDeviceModel::VirtioNetPci,
                    None,
                    vec![PortForwardCommand::new(
                        PortProtocol::Tcp,
                        Some(self.settings.adb_host_ip),
                        host_port,
                        None,
                        self.settings.adb_guest_port,
                    )],
                ))
                .map_err(|error| AndroidApplicationError::Network(format!("{error:?}")))?;
            true
        };

        let result = self.runtime_service.configure(ConfigureAndroidRuntimeCommand {
            vm_id: vm_id.clone(),
            adb_host_ip: self.settings.adb_host_ip,
            adb_host_port: host_port,
            adb_guest_port: self.settings.adb_guest_port,
            width: width.unwrap_or(self.settings.default_width),
            height: height.unwrap_or(self.settings.default_height),
            density_dpi: density_dpi.unwrap_or(self.settings.default_density_dpi),
            target_fps: target_fps.unwrap_or(self.settings.default_target_fps),
        });

        match result {
            Ok(profile) => Ok(profile),
            Err(error) => {
                if network_was_added {
                    let _ = self.network_service.detach_network(DetachNetworkCommand::new(
                        vm_id,
                        ANDROID_NETWORK_ID,
                    ));
                }
                Err(AndroidApplicationError::Runtime(format!("{error:?}")))
            }
        }
    }

    pub fn ensure_runtime_profile(&mut self, vm_id: &str) -> Result<AndroidRuntimeProfile, AndroidApplicationError> {
        let android_vm_id = AndroidVmId::parse(vm_id.to_owned())
            .map_err(|error| AndroidApplicationError::Runtime(format!("{error:?}")))?;
        match self.profile_repository.get(&android_vm_id) {
            Ok(profile) if !cfg!(windows) || self.is_windows_emulator_adb_port(profile.adb_host_port()) => Ok(profile),
            Ok(_) => {
                let machine = self.require_android_vm(vm_id)?;
                Self::require_stopped(&machine)?;
                if machine.networks().iter().any(|network| network.id().as_str() == ANDROID_NETWORK_ID) {
                    self.network_service
                        .detach_network(DetachNetworkCommand::new(vm_id.to_owned(), ANDROID_NETWORK_ID))
                        .map_err(|error| AndroidApplicationError::Network(format!("{error:?}")))?;
                }
                self.profile_repository
                    .delete(&android_vm_id)
                    .map_err(|error| AndroidApplicationError::Runtime(format!("{error:?}")))?;
                self.configure_runtime(vm_id.to_owned(), None, None, None, None)
            }
            Err(AndroidProfileRepositoryError::NotFound(_)) => self.configure_runtime(vm_id.to_owned(), None, None, None, None),
            Err(error) => Err(AndroidApplicationError::Runtime(format!("{error:?}"))),
        }
    }

    pub fn profile(&self, vm_id: &str) -> Result<AndroidRuntimeProfile, AndroidApplicationError> {
        self.require_android_vm(vm_id)?;
        self.runtime_service
            .profile(vm_id)
            .map_err(|error| AndroidApplicationError::Runtime(format!("{error:?}")))
    }

    pub fn status(&self, vm_id: &str) -> Result<AndroidDeviceReport, AndroidApplicationError> {
        let machine = self.require_android_vm(vm_id)?;
        Self::require_running(&machine)?;
        self.runtime_service
            .inspect(vm_id)
            .map_err(|error| AndroidApplicationError::Runtime(format!("{error:?}")))
    }

    pub fn wait_until_ready(
        &self,
        vm_id: &str,
    ) -> Result<AndroidDeviceReport, AndroidApplicationError> {
        let machine = self.require_android_vm(vm_id)?;
        Self::require_running(&machine)?;
        self.runtime_service
            .wait_until_ready(vm_id)
            .map_err(|error| AndroidApplicationError::Runtime(format!("{error:?}")))
    }

    pub fn apply_display(&self, vm_id: &str) -> Result<(), AndroidApplicationError> {
        let machine = self.require_android_vm(vm_id)?;
        Self::require_running(&machine)?;
        self.runtime_service
            .apply_display(vm_id)
            .map_err(|error| AndroidApplicationError::Runtime(format!("{error:?}")))
    }

    pub fn list_packages(&self, vm_id: &str) -> Result<Vec<AndroidPackageInfo>, AndroidApplicationError> {
        let machine = self.require_android_vm(vm_id)?;
        Self::require_running(&machine)?;
        self.package_service
            .list_packages(vm_id)
            .map_err(|error| AndroidApplicationError::Package(format!("{error:?}")))
    }

    pub fn install_apk(
        &self,
        vm_id: String,
        relative_apk_path: String,
    ) -> Result<(), AndroidApplicationError> {
        let machine = self.require_android_vm(&vm_id)?;
        Self::require_running(&machine)?;
        self.package_service
            .install_apk(InstallApkCommand {
                vm_id,
                relative_apk_path,
            })
            .map_err(|error| AndroidApplicationError::Package(format!("{error:?}")))
    }

    pub fn uninstall_package(
        &self,
        vm_id: String,
        package_name: String,
    ) -> Result<(), AndroidApplicationError> {
        self.package_action(vm_id, package_name, PackageAction::Uninstall)
    }

    pub fn launch_package(
        &self,
        vm_id: String,
        package_name: String,
    ) -> Result<(), AndroidApplicationError> {
        self.package_action(vm_id, package_name, PackageAction::Launch)
    }

    pub fn stop_package(
        &self,
        vm_id: String,
        package_name: String,
    ) -> Result<(), AndroidApplicationError> {
        self.package_action(vm_id, package_name, PackageAction::Stop)
    }

    pub fn provision_guest_agent(&mut self, vm_id: &str) -> Result<AndroidGuestAgentReport, AndroidApplicationError> {
        let machine = self.require_android_vm(vm_id)?;
        Self::require_running(&machine)?;
        let profile = self.profile(vm_id)?;
        self.guest_agent
            .provision_and_wait_ready(&profile)
            .map_err(|error| AndroidApplicationError::GuestAgent(format!("{error:?}")))
    }

    pub fn guest_agent_status(&mut self, vm_id: &str) -> Result<AndroidGuestAgentReport, AndroidApplicationError> {
        let machine = self.require_android_vm(vm_id)?;
        Self::require_running(&machine)?;
        let profile = self.profile(vm_id)?;
        self.guest_agent.inspect(&profile).map_err(|error| AndroidApplicationError::GuestAgent(format!("{error:?}")))
    }

    pub fn apply_guest_touch_frame(&mut self, vm_id: &str, contacts: &[AndroidTouchContact]) -> Result<(), AndroidApplicationError> {
        let machine = self.require_android_vm(vm_id)?;
        Self::require_running(&machine)?;
        let profile = self.profile(vm_id)?;
        self.guest_agent.apply_touch_frame(&profile, contacts).map_err(|error| AndroidApplicationError::GuestAgent(format!("{error:?}")))
    }

    pub fn reset_guest_input(&mut self, vm_id: &str) -> Result<(), AndroidApplicationError> {
        let machine = self.require_android_vm(vm_id)?;
        Self::require_running(&machine)?;
        let profile = self.profile(vm_id)?;
        self.guest_agent.reset_input(&profile).map_err(|error| AndroidApplicationError::GuestAgent(format!("{error:?}")))
    }


    pub fn read_guest_clipboard(&mut self, vm_id: &str) -> Result<String, AndroidApplicationError> {
        let machine = self.require_android_vm(vm_id)?;
        Self::require_running(&machine)?;
        let profile = self.profile(vm_id)?;
        self.guest_agent
            .read_clipboard(&profile)
            .map_err(|error| AndroidApplicationError::GuestAgent(format!("{error:?}")))
    }

    pub fn write_guest_clipboard(&mut self, vm_id: &str, text: &str) -> Result<(), AndroidApplicationError> {
        let machine = self.require_android_vm(vm_id)?;
        Self::require_running(&machine)?;
        let profile = self.profile(vm_id)?;
        self.guest_agent
            .write_clipboard(&profile, text)
            .map_err(|error| AndroidApplicationError::GuestAgent(format!("{error:?}")))
    }

    pub fn inject_input(
        &self,
        vm_id: String,
        action: AndroidInputAction,
    ) -> Result<(), AndroidApplicationError> {
        let machine = self.require_android_vm(&vm_id)?;
        Self::require_running(&machine)?;
        self.input_service
            .inject(AndroidInputCommand { vm_id, action })
            .map_err(|error| AndroidApplicationError::Input(format!("{error:?}")))
    }

    fn package_action(
        &self,
        vm_id: String,
        package_name: String,
        action: PackageAction,
    ) -> Result<(), AndroidApplicationError> {
        let machine = self.require_android_vm(&vm_id)?;
        Self::require_running(&machine)?;
        let command = PackageActionCommand {
            vm_id,
            package_name,
        };
        let result = match action {
            PackageAction::Uninstall => self.package_service.uninstall(command),
            PackageAction::Launch => self.package_service.launch(command),
            PackageAction::Stop => self.package_service.stop(command),
        };
        result.map_err(|error| AndroidApplicationError::Package(format!("{error:?}")))
    }


    fn android_network_host_port(
        &self,
        machine: &VirtualMachine,
    ) -> Result<Option<u16>, AndroidApplicationError> {
        let Some(network) = machine
            .networks()
            .iter()
            .find(|network| network.id().as_str() == ANDROID_NETWORK_ID)
        else {
            return Ok(None);
        };
        if network.mode() != NetworkMode::UserNat {
            return Err(AndroidApplicationError::AndroidNetworkMismatch);
        }
        let matches = network
            .port_forwards()
            .iter()
            .filter(|rule| {
                rule.protocol() == PortProtocol::Tcp
                    && rule.host_ip() == Some(self.settings.adb_host_ip)
                    && rule.guest_port() == self.settings.adb_guest_port
            })
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return Err(AndroidApplicationError::AndroidNetworkMismatch);
        }
        Ok(Some(matches[0].host_port()))
    }

    fn require_android_vm(&self, vm_id: &str) -> Result<VirtualMachine, AndroidApplicationError> {
        let machine = self
            .query_service
            .get(vm_id)
            .map_err(|error| AndroidApplicationError::Vm(format!("{error:?}")))?;
        if machine.guest_boot().profile() != GuestProfile::Android {
            return Err(AndroidApplicationError::VmProfileMustBeAndroid);
        }
        Ok(machine)
    }

    fn require_stopped(machine: &VirtualMachine) -> Result<(), AndroidApplicationError> {
        if machine.state() != VmState::Stopped {
            return Err(AndroidApplicationError::VmMustBeStopped(machine.state()));
        }
        Ok(())
    }

    fn require_running(machine: &VirtualMachine) -> Result<(), AndroidApplicationError> {
        if machine.state() != VmState::Running {
            return Err(AndroidApplicationError::VmMustBeRunning(machine.state()));
        }
        Ok(())
    }

    fn allocate_adb_port(&self) -> Result<u16, AndroidApplicationError> {
        let reserved: HashSet<u16> = self
            .profile_repository
            .list()
            .map_err(|error| AndroidApplicationError::Runtime(format!("{error:?}")))?
            .into_iter()
            .map(|profile| profile.adb_host_port())
            .collect();
        for port in self.settings.adb_host_port_min..=self.settings.adb_host_port_max {
            if cfg!(windows) && !self.is_windows_emulator_adb_port(port) { continue; }
            if reserved.contains(&port) { continue; }
            if TcpListener::bind(SocketAddrV4::new(self.settings.adb_host_ip, port)).is_err() { continue; }
            if cfg!(windows) {
                let console_port = port.saturating_sub(1);
                if TcpListener::bind(SocketAddrV4::new(self.settings.adb_host_ip, console_port)).is_err() { continue; }
            }
            return Ok(port);
        }
        Err(AndroidApplicationError::AdbPortExhausted)
    }

    fn is_windows_emulator_adb_port(&self, port: u16) -> bool {
        (self.settings.adb_host_port_min..=self.settings.adb_host_port_max).contains(&port) && port % 2 == 1
    }
}

#[derive(Debug, Clone, Copy)]
enum PackageAction {
    Uninstall,
    Launch,
    Stop,
}
