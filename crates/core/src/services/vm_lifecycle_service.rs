// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/services/vm_lifecycle_service.rs
// # 📌 Amac: VM create, start ve stop use-case is kurallarini yonetir
// # 📌 Modul - Rust
// # Version: 0.40.13
// # Aciklama: Repository, host network runtime, display session, HypervisorRuntime, guest-reset live ISO eject ve tek-oturumluk installer expiry kurallarini state machine ile orkestre eder
// # Bagimli Oldugu Katman: Repo | Tool

use crate::commands::create_vm_command::CreateVmCommand;
use crate::commands::recover_vm_command::RecoverVmCommand;
use crate::commands::start_vm_command::StartVmCommand;
use crate::commands::stop_vm_command::StopVmCommand;
use crate::domain::guest_boot::{GuestBootConfiguration, GuestBootDomainError};
use crate::domain::hypervisor_runtime_event::{HypervisorRuntimeEvent, HypervisorRuntimeEventKind};
use crate::domain::network::NetworkRuntimePlan;
use crate::domain::runtime_media::VmRuntimeMediaPlan;
use crate::domain::virtual_machine::{
    VirtualMachine, VmDomainError, VmFailure, VmFailureKind, VmId, VmResourceConfig,
};
use crate::domain::vm_state::VmState;
use crate::ports::hypervisor_runtime_port::{HypervisorRuntimeError, HypervisorRuntimePort};
use crate::ports::network_port::{NetworkError, NetworkPort};
use crate::ports::vm_repository_port::{VmRepositoryError, VmRepositoryPort};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmLifecycleError {
    Domain(VmDomainError),
    GuestDomain(GuestBootDomainError),
    Repository(VmRepositoryError),
    Network(NetworkError),
    Hypervisor(HypervisorRuntimeError),
    RecoveryRequiresErrorState(VmState),
    RecoveryFailureMissing,
    RecoveryNotSafe(VmFailureKind),
}

pub struct VmLifecycleService<R, H, N>
where
    R: VmRepositoryPort,
    H: HypervisorRuntimePort,
    N: NetworkPort,
{
    repository: R,
    hypervisor: H,
    network: N,
}

impl<R, H, N> VmLifecycleService<R, H, N>
where
    R: VmRepositoryPort,
    H: HypervisorRuntimePort,
    N: NetworkPort,
{
    pub const fn new(repository: R, hypervisor: H, network: N) -> Self {
        Self {
            repository,
            hypervisor,
            network,
        }
    }

    pub fn create_vm(
        &mut self,
        command: CreateVmCommand,
    ) -> Result<VirtualMachine, VmLifecycleError> {
        let id = VmId::parse(command.vm_id).map_err(VmLifecycleError::Domain)?;
        let mut machine = VirtualMachine::create(
            id,
            command.name,
            VmResourceConfig {
                vcpu_count: command.vcpu_count,
                memory_mib: command.memory_mib,
            },
            command.acceleration,
        )
        .map_err(VmLifecycleError::Domain)?;
        machine.configure_guest_boot(
            GuestBootConfiguration::default_for_profile(
                command.guest_profile,
                command.guest_template_id,
            )
            .map_err(VmLifecycleError::GuestDomain)?,
        );

        machine
            .transition_to(VmState::Stopped)
            .map_err(VmLifecycleError::Domain)?;
        self.repository
            .insert(machine.clone())
            .map_err(VmLifecycleError::Repository)?;

        Ok(machine)
    }

    pub fn recover_vm(
        &mut self,
        command: RecoverVmCommand,
    ) -> Result<VirtualMachine, VmLifecycleError> {
        let id = VmId::parse(command.vm_id).map_err(VmLifecycleError::Domain)?;
        let mut machine = self
            .repository
            .get(&id)
            .map_err(VmLifecycleError::Repository)?;

        if machine.state() == VmState::Stopped {
            return Ok(machine);
        }
        if machine.state() != VmState::Error {
            return Err(VmLifecycleError::RecoveryRequiresErrorState(machine.state()));
        }
        let failure_kind = machine
            .last_failure()
            .map(|failure| failure.kind)
            .ok_or(VmLifecycleError::RecoveryFailureMissing)?;
        if !matches!(
            failure_kind,
            VmFailureKind::NetworkPrepare | VmFailureKind::NetworkBind | VmFailureKind::HypervisorStart | VmFailureKind::RuntimeExit
        ) {
            return Err(VmLifecycleError::RecoveryNotSafe(failure_kind));
        }

        self.network
            .cleanup_runtime(&id)
            .map_err(VmLifecycleError::Network)?;
        machine
            .transition_to(VmState::Stopped)
            .map_err(VmLifecycleError::Domain)?;
        self.repository
            .save(machine.clone())
            .map_err(VmLifecycleError::Repository)?;
        Ok(machine)
    }

    pub fn start_vm(
        &mut self,
        command: StartVmCommand,
    ) -> Result<VirtualMachine, VmLifecycleError> {
        let id = VmId::parse(command.vm_id).map_err(VmLifecycleError::Domain)?;
        let mut machine = self
            .repository
            .get(&id)
            .map_err(VmLifecycleError::Repository)?;

        machine
            .transition_to(VmState::Starting)
            .map_err(VmLifecycleError::Domain)?;
        self.repository
            .save(machine.clone())
            .map_err(VmLifecycleError::Repository)?;

        let uses_host_network_runtime = command
            .runtime_media
            .as_ref()
            .map(VmRuntimeMediaPlan::uses_host_network_runtime)
            .unwrap_or(true);
        let network_plan = if uses_host_network_runtime {
            match self.network.prepare_runtime(&machine) {
                Ok(plan) => plan,
                Err(error) => {
                    self.persist_runtime_failure(
                        &mut machine,
                        VmFailureKind::NetworkPrepare,
                        format!("{error:?}"),
                    );
                    return Err(VmLifecycleError::Network(error));
                }
            }
        } else {
            NetworkRuntimePlan::default()
        };

        let runtime = match self.hypervisor.start(&machine, &network_plan, command.runtime_media.as_ref()) {
            Ok(runtime) => runtime,
            Err(error) => {
                let _ = self.network.cleanup_runtime(machine.id());
                self.persist_runtime_failure(
                    &mut machine,
                    VmFailureKind::HypervisorStart,
                    format!("{error:?}"),
                );
                return Err(VmLifecycleError::Hypervisor(error));
            }
        };

        if uses_host_network_runtime {
            if let Err(error) = self
                .network
                .bind_runtime_process(machine.id(), runtime.process_id)
            {
                let _ = self.hypervisor.stop(machine.id());
                let _ = self.network.cleanup_runtime(machine.id());
                self.persist_runtime_failure(
                    &mut machine,
                    VmFailureKind::NetworkBind,
                    format!("{error:?}"),
                );
                return Err(VmLifecycleError::Network(error));
            }
        }

        machine
            .transition_to(VmState::Running)
            .map_err(VmLifecycleError::Domain)?;
        if let Err(repository_error) = self.repository.save(machine.clone()) {
            let _ = self.hypervisor.stop(machine.id());
            let _ = self.network.cleanup_runtime(machine.id());
            return Err(VmLifecycleError::Repository(repository_error));
        }

        Ok(machine)
    }

    pub fn stop_vm(
        &mut self,
        command: StopVmCommand,
    ) -> Result<VirtualMachine, VmLifecycleError> {
        let id = VmId::parse(command.vm_id).map_err(VmLifecycleError::Domain)?;
        let mut machine = self
            .repository
            .get(&id)
            .map_err(VmLifecycleError::Repository)?;

        machine
            .transition_to(VmState::Stopping)
            .map_err(VmLifecycleError::Domain)?;
        self.repository
            .save(machine.clone())
            .map_err(VmLifecycleError::Repository)?;

        if let Err(error) = self.hypervisor.stop(&id) {
            self.persist_runtime_failure(
                &mut machine,
                VmFailureKind::HypervisorStop,
                format!("{error:?}"),
            );
            return Err(VmLifecycleError::Hypervisor(error));
        }

        machine
            .guest_boot_mut()
            .expire_one_shot_installer_media()
            .map_err(VmLifecycleError::GuestDomain)?;

        if let Err(error) = self.network.cleanup_runtime(&id) {
            self.persist_runtime_failure(
                &mut machine,
                VmFailureKind::NetworkCleanup,
                format!("{error:?}"),
            );
            return Err(VmLifecycleError::Network(error));
        }

        machine
            .transition_to(VmState::Stopped)
            .map_err(VmLifecycleError::Domain)?;
        self.repository
            .save(machine.clone())
            .map_err(VmLifecycleError::Repository)?;

        Ok(machine)
    }


    pub fn maintenance(&mut self) -> Result<Vec<HypervisorRuntimeEvent>, VmLifecycleError> {
        let events = self.hypervisor.maintenance().map_err(VmLifecycleError::Hypervisor)?;
        for event in &events {
            let mut machine = match self.repository.get(&event.vm_id) {
                Ok(machine) => machine,
                Err(VmRepositoryError::NotFound(_)) => continue,
                Err(error) => return Err(VmLifecycleError::Repository(error)),
            };
            if !matches!(machine.state(), VmState::Running | VmState::Starting | VmState::Pausing | VmState::Paused | VmState::Resuming) {
                continue;
            }

            if matches!(event.kind, HypervisorRuntimeEventKind::GuestReset) {
                let one_shot_media_id = machine
                    .guest_boot()
                    .installer_iso()
                    .filter(|_| machine.guest_boot().boot_order().apply_once())
                    .map(|iso| iso.id().as_str().to_owned());
                if let Some(media_id) = one_shot_media_id {
                    let eject_result = self
                        .hypervisor
                        .eject_installer_media(&event.vm_id, &media_id);
                    machine
                        .guest_boot_mut()
                        .expire_one_shot_installer_media()
                        .map_err(VmLifecycleError::GuestDomain)?;
                    self.repository
                        .save(machine.clone())
                        .map_err(VmLifecycleError::Repository)?;
                    eject_result.map_err(VmLifecycleError::Hypervisor)?;
                }
                continue;
            }

            if matches!(event.kind, HypervisorRuntimeEventKind::ProcessExited) {
                let _ = self.network.cleanup_runtime(&event.vm_id);
                machine
                    .guest_boot_mut()
                    .expire_one_shot_installer_media()
                    .map_err(VmLifecycleError::GuestDomain)?;
                self.persist_runtime_failure(
                    &mut machine,
                    VmFailureKind::RuntimeExit,
                    String::from("hypervisor runtime exited unexpectedly"),
                );
            }
        }
        Ok(events)
    }

    pub fn runtime_info(
        &self,
        vm_id: impl Into<String>,
    ) -> Result<crate::ports::hypervisor_runtime_port::HypervisorRuntimeInfo, VmLifecycleError> {
        let id = VmId::parse(vm_id.into()).map_err(VmLifecycleError::Domain)?;
        self.hypervisor
            .runtime_info(&id)
            .map_err(VmLifecycleError::Hypervisor)
    }

    pub fn into_parts(self) -> (R, H, N) {
        (self.repository, self.hypervisor, self.network)
    }

    fn persist_runtime_failure(
        &mut self,
        machine: &mut VirtualMachine,
        kind: VmFailureKind,
        detail: String,
    ) {
        let failure = VmFailure { kind, detail };
        if machine.mark_failure(failure).is_ok() {
            let _ = self.repository.save(machine.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::domain::guest_boot::{
        BootDevice, BootOrder, FirmwareSelection, GuestBootConfiguration, GuestProfile,
        IsoAttachment, MediaId,
    };
    use crate::domain::hypervisor::AccelerationBackend;
    use crate::domain::network::{
        NetworkAttachment, NetworkRuntimePlan, PreparedNetworkBackend, PreparedNetworkBinding,
    };
    use crate::ports::hypervisor_runtime_port::{
        HypervisorRuntimeInfo, HypervisorStopReport, HypervisorTerminationKind,
    };
    use crate::ports::network_port::{NetworkCapabilities, NetworkRecoveryReport};

    #[derive(Default)]
    struct FakeRepository {
        machines: HashMap<VmId, VirtualMachine>,
    }

    impl VmRepositoryPort for FakeRepository {
        fn insert(&mut self, machine: VirtualMachine) -> Result<(), VmRepositoryError> {
            if self.machines.contains_key(machine.id()) {
                return Err(VmRepositoryError::AlreadyExists(machine.id().clone()));
            }
            self.machines.insert(machine.id().clone(), machine);
            Ok(())
        }

        fn get(&self, vm_id: &VmId) -> Result<VirtualMachine, VmRepositoryError> {
            self.machines
                .get(vm_id)
                .cloned()
                .ok_or_else(|| VmRepositoryError::NotFound(vm_id.clone()))
        }

        fn list(&self) -> Result<Vec<VirtualMachine>, VmRepositoryError> {
            Ok(self.machines.values().cloned().collect())
        }

        fn save(&mut self, machine: VirtualMachine) -> Result<(), VmRepositoryError> {
            if !self.machines.contains_key(machine.id()) {
                return Err(VmRepositoryError::NotFound(machine.id().clone()));
            }
            self.machines.insert(machine.id().clone(), machine);
            Ok(())
        }

        fn delete(&mut self, vm_id: &VmId) -> Result<(), VmRepositoryError> {
            self.machines
                .remove(vm_id)
                .map(|_| ())
                .ok_or_else(|| VmRepositoryError::NotFound(vm_id.clone()))
        }
    }

    #[derive(Default)]
    struct FakeHypervisor {
        running: Vec<VmId>,
        fail_start: bool,
        maintenance_events: Vec<HypervisorRuntimeEvent>,
        ejected_media: Vec<(VmId, String)>,
    }

    impl HypervisorRuntimePort for FakeHypervisor {
        fn start(
            &mut self,
            machine: &VirtualMachine,
            _network_plan: &NetworkRuntimePlan,
            _runtime_media: Option<&crate::domain::runtime_media::VmRuntimeMediaPlan>,
        ) -> Result<HypervisorRuntimeInfo, HypervisorRuntimeError> {
            if self.fail_start {
                return Err(HypervisorRuntimeError::LaunchFailed(String::from(
                    "fake start failure",
                )));
            }
            self.running.push(machine.id().clone());
            Ok(HypervisorRuntimeInfo { process_id: 42, display: None })
        }

        fn stop(
            &mut self,
            vm_id: &VmId,
        ) -> Result<HypervisorStopReport, HypervisorRuntimeError> {
            if let Some(index) = self.running.iter().position(|id| id == vm_id) {
                self.running.remove(index);
                return Ok(HypervisorStopReport {
                    termination: HypervisorTerminationKind::Graceful,
                });
            }
            Err(HypervisorRuntimeError::NotRunning(vm_id.clone()))
        }

        fn eject_installer_media(
            &mut self,
            vm_id: &VmId,
            media_id: &str,
        ) -> Result<(), HypervisorRuntimeError> {
            self.ejected_media
                .push((vm_id.clone(), media_id.to_owned()));
            Ok(())
        }

        fn maintenance(&mut self) -> Result<Vec<HypervisorRuntimeEvent>, HypervisorRuntimeError> {
            Ok(std::mem::take(&mut self.maintenance_events))
        }
    }

    #[derive(Default)]
    struct FakeNetwork {
        prepared: Vec<VmId>,
        bound: Vec<(VmId, u32)>,
    }

    impl NetworkPort for FakeNetwork {
        fn capabilities(&self) -> NetworkCapabilities {
            NetworkCapabilities {
                managed_nat: true,
                private_network: true,
                user_nat: true,
                bridge: true,
                existing_tap: true,
                managed_tap: true,
                bridge_helper: true,
            }
        }

        fn validate_attachment(&self, _attachment: &NetworkAttachment) -> Result<(), NetworkError> {
            Ok(())
        }

        fn prepare_runtime(
            &mut self,
            machine: &VirtualMachine,
        ) -> Result<NetworkRuntimePlan, NetworkError> {
            self.prepared.push(machine.id().clone());
            let bindings = machine
                .networks()
                .iter()
                .map(|network| {
                    PreparedNetworkBinding::new(
                        network.id().clone(),
                        PreparedNetworkBackend::UserNat,
                    )
                })
                .collect();
            NetworkRuntimePlan::new(bindings)
                .map_err(|error| NetworkError::PreparationFailed(format!("{error:?}")))
        }

        fn bind_runtime_process(
            &mut self,
            vm_id: &VmId,
            process_id: u32,
        ) -> Result<(), NetworkError> {
            self.bound.push((vm_id.clone(), process_id));
            Ok(())
        }

        fn cleanup_runtime(&mut self, vm_id: &VmId) -> Result<(), NetworkError> {
            self.prepared.retain(|id| id != vm_id);
            Ok(())
        }

        fn recover_runtime(&mut self) -> Result<NetworkRecoveryReport, NetworkError> {
            Ok(NetworkRecoveryReport::default())
        }
    }

    fn create_command() -> CreateVmCommand {
        CreateVmCommand::new(
            "dev-vm",
            "Dev VM",
            4,
            4096,
            AccelerationBackend::Tcg,
            crate::domain::guest_boot::GuestProfile::Generic,
            None,
        )
    }

    #[test]
    fn create_start_stop_follows_state_machine_and_network_lifecycle() {
        let mut service = VmLifecycleService::new(
            FakeRepository::default(),
            FakeHypervisor::default(),
            FakeNetwork::default(),
        );

        let created = service.create_vm(create_command()).expect("create must succeed");
        assert_eq!(created.state(), VmState::Stopped);

        let running = service
            .start_vm(StartVmCommand::new("dev-vm"))
            .expect("start must succeed");
        assert_eq!(running.state(), VmState::Running);

        let stopped = service
            .stop_vm(StopVmCommand::new("dev-vm"))
            .expect("stop must succeed");
        assert_eq!(stopped.state(), VmState::Stopped);
    }


    fn configure_one_shot_installer(machine: &mut VirtualMachine) {
        let boot_order = BootOrder::create(vec![BootDevice::Cdrom, BootDevice::Disk], true)
            .expect("one-shot boot order must be valid");
        let installer = IsoAttachment::create(
            MediaId::parse("installer").expect("media id must be valid"),
            "media/installer.iso",
        )
        .expect("installer attachment must be valid");
        let configuration = GuestBootConfiguration::create(
            GuestProfile::Linux,
            FirmwareSelection::Bios,
            boot_order,
            Some(installer),
            Some(String::from("fedora-44-server")),
        )
        .expect("guest boot configuration must be valid");
        machine.configure_guest_boot(configuration);
    }

    #[test]
    fn one_shot_installer_stays_attached_while_running_and_expires_on_stop() {
        let mut service = VmLifecycleService::new(
            FakeRepository::default(),
            FakeHypervisor::default(),
            FakeNetwork::default(),
        );
        let mut machine = service.create_vm(create_command()).expect("create must succeed");
        configure_one_shot_installer(&mut machine);
        service
            .repository
            .save(machine)
            .expect("configured machine must persist");

        let running = service
            .start_vm(StartVmCommand::new("dev-vm"))
            .expect("start must succeed");
        assert!(running.guest_boot().installer_iso().is_some());
        assert!(running.guest_boot().boot_order().apply_once());

        let stopped = service
            .stop_vm(StopVmCommand::new("dev-vm"))
            .expect("stop must succeed");
        assert!(stopped.guest_boot().installer_iso().is_none());
        assert_eq!(stopped.guest_boot().boot_order().devices(), &[BootDevice::Disk]);
        assert!(!stopped.guest_boot().boot_order().apply_once());
        assert_eq!(stopped.guest_boot().catalog_template_id(), Some("fedora-44-server"));
    }

    #[test]
    fn guest_reset_live_ejects_one_shot_installer_and_keeps_vm_running() {
        let mut service = VmLifecycleService::new(
            FakeRepository::default(),
            FakeHypervisor::default(),
            FakeNetwork::default(),
        );
        let mut machine = service.create_vm(create_command()).expect("create must succeed");
        configure_one_shot_installer(&mut machine);
        service
            .repository
            .save(machine)
            .expect("configured machine must persist");
        service
            .start_vm(StartVmCommand::new("dev-vm"))
            .expect("start must succeed");

        let vm_id = VmId::parse("dev-vm").expect("id must be valid");
        service.hypervisor.maintenance_events.push(HypervisorRuntimeEvent {
            vm_id: vm_id.clone(),
            kind: HypervisorRuntimeEventKind::GuestReset,
        });
        service.maintenance().expect("guest reset maintenance must succeed");

        let machine = service.repository.get(&vm_id).expect("machine must exist");
        assert_eq!(machine.state(), VmState::Running);
        assert!(machine.guest_boot().installer_iso().is_none());
        assert_eq!(machine.guest_boot().boot_order().devices(), &[BootDevice::Disk]);
        assert_eq!(
            service.hypervisor.ejected_media,
            vec![(vm_id, String::from("installer"))]
        );
    }

    #[test]
    fn runtime_exit_expires_one_shot_installer_before_recovery_restart() {
        let hypervisor = FakeHypervisor::default();
        let mut service = VmLifecycleService::new(
            FakeRepository::default(),
            hypervisor,
            FakeNetwork::default(),
        );
        let mut machine = service.create_vm(create_command()).expect("create must succeed");
        configure_one_shot_installer(&mut machine);
        service
            .repository
            .save(machine)
            .expect("configured machine must persist");
        service
            .start_vm(StartVmCommand::new("dev-vm"))
            .expect("start must succeed");

        let vm_id = VmId::parse("dev-vm").expect("id must be valid");
        service.hypervisor.maintenance_events.push(HypervisorRuntimeEvent {
            vm_id: vm_id.clone(),
            kind: HypervisorRuntimeEventKind::ProcessExited,
        });
        service.maintenance().expect("maintenance must succeed");

        let machine = service.repository.get(&vm_id).expect("machine must exist");
        assert_eq!(machine.state(), VmState::Error);
        assert!(machine.guest_boot().installer_iso().is_none());
        assert_eq!(machine.guest_boot().boot_order().devices(), &[BootDevice::Disk]);
    }

    #[test]
    fn invalid_guest_template_id_is_reported_as_guest_domain_error() {
        let mut service = VmLifecycleService::new(
            FakeRepository::default(),
            FakeHypervisor::default(),
            FakeNetwork::default(),
        );
        let command = CreateVmCommand::new(
            "catalog-vm",
            "Catalog VM",
            2,
            2048,
            AccelerationBackend::Tcg,
            crate::domain::guest_boot::GuestProfile::Linux,
            Some(String::from("invalid template id")),
        );

        let error = service
            .create_vm(command)
            .expect_err("invalid guest template id must fail");

        assert_eq!(
            error,
            VmLifecycleError::GuestDomain(
                crate::domain::guest_boot::GuestBootDomainError::InvalidCatalogTemplateId
            )
        );
    }


    #[test]
    fn safe_start_failure_can_recover_to_stopped() {
        let hypervisor = FakeHypervisor {
            fail_start: true,
            ..FakeHypervisor::default()
        };
        let mut service = VmLifecycleService::new(
            FakeRepository::default(),
            hypervisor,
            FakeNetwork::default(),
        );
        service.create_vm(create_command()).expect("create must succeed");
        service
            .start_vm(StartVmCommand::new("dev-vm"))
            .expect_err("start must fail");

        let recovered = service
            .recover_vm(RecoverVmCommand::new("dev-vm"))
            .expect("safe start failure recovery must succeed");
        assert_eq!(recovered.state(), VmState::Stopped);
        assert!(recovered.last_failure().is_none());
    }

    #[test]
    fn hypervisor_start_failure_moves_machine_to_error_and_cleans_network() {
        let hypervisor = FakeHypervisor {
            fail_start: true,
            ..FakeHypervisor::default()
        };
        let mut service = VmLifecycleService::new(
            FakeRepository::default(),
            hypervisor,
            FakeNetwork::default(),
        );
        service.create_vm(create_command()).expect("create must succeed");

        let error = service
            .start_vm(StartVmCommand::new("dev-vm"))
            .expect_err("start must fail");
        assert!(matches!(error, VmLifecycleError::Hypervisor(_)));

        let (repository, _, network) = service.into_parts();
        let id = VmId::parse("dev-vm").expect("id must be valid");
        let machine = repository.get(&id).expect("machine must exist");
        assert_eq!(machine.state(), VmState::Error);
        assert!(network.prepared.is_empty());
        assert_eq!(
            machine.last_failure().map(|failure| failure.kind),
            Some(VmFailureKind::HypervisorStart)
        );
    }
}
