// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/services/network_service.rs
// # 📌 Amac: VM network attachment ve network runtime recovery use-case is kurallarini yonetir
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: VM state, repository, managed IPv4, host network validation ve runtime recovery islemlerini orkestre eder
// # Bagimli Oldugu Katman: Repo | Tool

use crate::commands::attach_network_command::AttachNetworkCommand;
use crate::commands::detach_network_command::DetachNetworkCommand;
use crate::commands::update_network_service_command::{PublishNetworkServiceCommand, UnpublishNetworkServiceCommand};
use crate::domain::network::{
    BridgeConfiguration, HostNetworkName, MacAddress, ManagedAddressConfiguration, NetworkAttachment, NetworkDomainError,
    NetworkId, PortForwardRule,
};
use crate::domain::virtual_machine::{VirtualMachine, VmDomainError, VmId};
use crate::domain::vm_state::VmState;
use crate::ports::network_port::{NetworkError, NetworkPort, NetworkRecoveryReport};
use crate::ports::vm_repository_port::{VmRepositoryError, VmRepositoryPort};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkServiceError {
    VmDomain(VmDomainError),
    NetworkDomain(NetworkDomainError),
    Repository(VmRepositoryError),
    Network(NetworkError),
    VmMustBeStopped(VmState),
}

pub struct NetworkService<R, N>
where
    R: VmRepositoryPort,
    N: NetworkPort,
{
    repository: R,
    network: N,
}

impl<R, N> NetworkService<R, N>
where
    R: VmRepositoryPort,
    N: NetworkPort,
{
    pub const fn new(repository: R, network: N) -> Self {
        Self { repository, network }
    }

    pub fn attach_network(
        &mut self,
        command: AttachNetworkCommand,
    ) -> Result<VirtualMachine, NetworkServiceError> {
        let vm_id = VmId::parse(command.vm_id).map_err(NetworkServiceError::VmDomain)?;
        let mut machine = self
            .repository
            .get(&vm_id)
            .map_err(NetworkServiceError::Repository)?;
        Self::require_stopped(&machine)?;

        let network_id = NetworkId::parse(command.network_id)
            .map_err(NetworkServiceError::NetworkDomain)?;
        let mac_address = command
            .mac_address
            .as_deref()
            .map(MacAddress::parse)
            .transpose()
            .map_err(NetworkServiceError::NetworkDomain)?;

        let mut forwards = Vec::with_capacity(command.port_forwards.len());
        for forward in command.port_forwards {
            forwards.push(
                PortForwardRule::create(
                    forward.protocol,
                    forward.host_ip,
                    forward.host_port,
                    forward.guest_ip,
                    forward.guest_port,
                )
                .map_err(NetworkServiceError::NetworkDomain)?,
            );
        }

        let bridge = command
            .bridge
            .map(|bridge| {
                let bridge_name = bridge
                    .bridge_name
                    .map(HostNetworkName::parse)
                    .transpose()?;
                let tap_name = bridge.tap_name.map(HostNetworkName::parse).transpose()?;
                BridgeConfiguration::create(bridge_name, tap_name)
            })
            .transpose()
            .map_err(NetworkServiceError::NetworkDomain)?;

        let managed_address = command
            .managed_address
            .map(|address| {
                ManagedAddressConfiguration::create(
                    address.ipv4_address,
                    address.prefix_length,
                    address.gateway,
                    address.dns_servers,
                )
            })
            .transpose()
            .map_err(NetworkServiceError::NetworkDomain)?;

        let fabric_id = command.fabric_id
            .map(NetworkId::parse)
            .transpose()
            .map_err(NetworkServiceError::NetworkDomain)?;

        let attachment = NetworkAttachment::create_with_address(
            network_id,
            command.mode,
            command.device_model,
            mac_address,
            forwards,
            bridge,
            managed_address,
            fabric_id,
        )
        .map_err(NetworkServiceError::NetworkDomain)?;

        self.network
            .validate_attachment(&attachment)
            .map_err(NetworkServiceError::Network)?;
        machine
            .attach_network(attachment)
            .map_err(NetworkServiceError::VmDomain)?;
        self.repository
            .save(machine.clone())
            .map_err(NetworkServiceError::Repository)?;

        Ok(machine)
    }

    pub fn publish_service(
        &mut self,
        command: PublishNetworkServiceCommand,
    ) -> Result<VirtualMachine, NetworkServiceError> {
        let vm_id = VmId::parse(command.vm_id).map_err(NetworkServiceError::VmDomain)?;
        let network_id = NetworkId::parse(command.network_id).map_err(NetworkServiceError::NetworkDomain)?;
        let mut machine = self.repository.get(&vm_id).map_err(NetworkServiceError::Repository)?;
        Self::require_stopped(&machine)?;
        let guest_ip = machine
            .network(&network_id)
            .map_err(NetworkServiceError::VmDomain)?
            .managed_address()
            .map(|address| address.ipv4_address());
        let rule = PortForwardRule::create(
            command.protocol,
            command.host_ip,
            command.host_port,
            guest_ip,
            command.guest_port,
        ).map_err(NetworkServiceError::NetworkDomain)?;
        machine.network_mut(&network_id)
            .map_err(NetworkServiceError::VmDomain)?
            .add_port_forward(rule)
            .map_err(NetworkServiceError::NetworkDomain)?;
        self.repository.save(machine.clone()).map_err(NetworkServiceError::Repository)?;
        Ok(machine)
    }

    pub fn unpublish_service(
        &mut self,
        command: UnpublishNetworkServiceCommand,
    ) -> Result<VirtualMachine, NetworkServiceError> {
        let vm_id = VmId::parse(command.vm_id).map_err(NetworkServiceError::VmDomain)?;
        let network_id = NetworkId::parse(command.network_id).map_err(NetworkServiceError::NetworkDomain)?;
        let mut machine = self.repository.get(&vm_id).map_err(NetworkServiceError::Repository)?;
        Self::require_stopped(&machine)?;
        machine.network_mut(&network_id)
            .map_err(NetworkServiceError::VmDomain)?
            .remove_port_forward(command.protocol, command.host_port)
            .map_err(NetworkServiceError::NetworkDomain)?;
        self.repository.save(machine.clone()).map_err(NetworkServiceError::Repository)?;
        Ok(machine)
    }

    pub fn detach_network(
        &mut self,
        command: DetachNetworkCommand,
    ) -> Result<VirtualMachine, NetworkServiceError> {
        let vm_id = VmId::parse(command.vm_id).map_err(NetworkServiceError::VmDomain)?;
        let network_id = NetworkId::parse(command.network_id)
            .map_err(NetworkServiceError::NetworkDomain)?;
        let mut machine = self
            .repository
            .get(&vm_id)
            .map_err(NetworkServiceError::Repository)?;
        Self::require_stopped(&machine)?;
        machine
            .detach_network(&network_id)
            .map_err(NetworkServiceError::VmDomain)?;
        self.repository
            .save(machine.clone())
            .map_err(NetworkServiceError::Repository)?;
        Ok(machine)
    }

    pub fn recover_runtime(&mut self) -> Result<NetworkRecoveryReport, NetworkServiceError> {
        self.network
            .recover_runtime()
            .map_err(NetworkServiceError::Network)
    }

    pub fn capabilities(&self) -> crate::ports::network_port::NetworkCapabilities {
        self.network.capabilities()
    }

    pub fn into_parts(self) -> (R, N) {
        (self.repository, self.network)
    }

    fn require_stopped(machine: &VirtualMachine) -> Result<(), NetworkServiceError> {
        if machine.state() != VmState::Stopped {
            return Err(NetworkServiceError::VmMustBeStopped(machine.state()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::commands::attach_network_command::PortForwardCommand;
    use crate::domain::hypervisor::AccelerationBackend;
    use crate::domain::network::{
        NetworkDeviceModel, NetworkMode, NetworkRuntimePlan, PortProtocol,
    };
    use crate::domain::virtual_machine::VmResourceConfig;
    use crate::ports::network_port::{NetworkCapabilities, NetworkRecoveryReport};

    #[derive(Default)]
    struct FakeRepository {
        machines: HashMap<VmId, VirtualMachine>,
    }

    impl VmRepositoryPort for FakeRepository {
        fn insert(&mut self, machine: VirtualMachine) -> Result<(), VmRepositoryError> {
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
    struct FakeNetwork;

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
            _machine: &VirtualMachine,
        ) -> Result<NetworkRuntimePlan, NetworkError> {
            Ok(NetworkRuntimePlan::default())
        }

        fn bind_runtime_process(
            &mut self,
            _vm_id: &VmId,
            _process_id: u32,
        ) -> Result<(), NetworkError> {
            Ok(())
        }

        fn cleanup_runtime(&mut self, _vm_id: &VmId) -> Result<(), NetworkError> {
            Ok(())
        }

        fn recover_runtime(&mut self) -> Result<NetworkRecoveryReport, NetworkError> {
            Ok(NetworkRecoveryReport::default())
        }
    }

    fn stopped_machine() -> VirtualMachine {
        let mut machine = VirtualMachine::create(
            VmId::parse("network-test").expect("vm id must be valid"),
            "Network Test",
            VmResourceConfig {
                vcpu_count: 2,
                memory_mib: 2048,
            },
            AccelerationBackend::Tcg,
        )
        .expect("vm must be valid");
        machine
            .transition_to(VmState::Stopped)
            .expect("created vm must stop");
        machine
    }

    #[test]
    fn user_nat_is_attached_to_stopped_vm() {
        let mut repository = FakeRepository::default();
        repository
            .insert(stopped_machine())
            .expect("insert must succeed");
        let mut service = NetworkService::new(repository, FakeNetwork);

        let machine = service
            .attach_network(AttachNetworkCommand::new_user_nat(
                "network-test",
                "default",
                NetworkDeviceModel::VirtioNetPci,
                None,
                vec![PortForwardCommand::new(
                    PortProtocol::Tcp,
                    None,
                    2222,
                    None,
                    22,
                )],
            ))
            .expect("network attach must succeed");

        assert_eq!(machine.networks().len(), 1);
        assert_eq!(machine.networks()[0].port_forwards().len(), 1);
        assert_eq!(machine.networks()[0].mode(), NetworkMode::UserNat);
    }

    #[test]
    fn bridge_requires_explicit_host_target() {
        let mut repository = FakeRepository::default();
        repository
            .insert(stopped_machine())
            .expect("insert must succeed");
        let mut service = NetworkService::new(repository, FakeNetwork);

        let result = service.attach_network(AttachNetworkCommand::new_bridge(
            "network-test",
            "bridge0",
            NetworkDeviceModel::VirtioNetPci,
            None,
            None,
            None,
        ));

        assert!(matches!(
            result,
            Err(NetworkServiceError::NetworkDomain(
                NetworkDomainError::BridgeTargetRequired
            ))
        ));
    }
}
