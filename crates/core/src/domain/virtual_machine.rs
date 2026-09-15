// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/domain/virtual_machine.rs
// # 📌 Amac: TurkuazVM VirtualMachine aggregate ve domain invariantlarini tanimlar
// # 📌 Modul - Rust
// # Version: 0.40.3
// # Aciklama: VM kimligi, kaynaklari, guest boot oturum yasam dongusu, disk/network attachmentlari, acceleration, state ve failure bilgisini modeller
// # Bagimli Oldugu Katman: Service | Repo

use crate::domain::disk::{DiskAttachment, DiskDomainError, DiskId};
use crate::domain::guest_boot::GuestBootConfiguration;
use crate::domain::hypervisor::AccelerationBackend;
use crate::domain::network::{NetworkAttachment, NetworkDomainError, NetworkId};
use crate::domain::snapshot::{SnapshotDomainError, SnapshotId, SnapshotRecord};
use crate::domain::vm_state::VmState;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VmId(String);

impl VmId {
    pub fn parse(value: impl Into<String>) -> Result<Self, VmDomainError> {
        let value = value.into();
        let valid = !value.is_empty()
            && value.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
            });

        if !valid {
            return Err(VmDomainError::InvalidId);
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmResourceConfig {
    pub vcpu_count: u16,
    pub memory_mib: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmFailureKind {
    NetworkPrepare,
    NetworkBind,
    NetworkCleanup,
    HypervisorStart,
    HypervisorStop,
    RuntimeExit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmFailure {
    pub kind: VmFailureKind,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmDomainError {
    InvalidId,
    InvalidName,
    InvalidVcpuCount,
    InvalidMemory,
    InvalidStateTransition { from: VmState, to: VmState },
    Disk(DiskDomainError),
    Network(NetworkDomainError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualMachine {
    id: VmId,
    name: String,
    resources: VmResourceConfig,
    acceleration: AccelerationBackend,
    state: VmState,
    last_failure: Option<VmFailure>,
    guest_boot: GuestBootConfiguration,
    disks: Vec<DiskAttachment>,
    networks: Vec<NetworkAttachment>,
    snapshots: Vec<SnapshotRecord>,
}

impl VirtualMachine {
    pub fn create(
        id: VmId,
        name: impl Into<String>,
        resources: VmResourceConfig,
        acceleration: AccelerationBackend,
    ) -> Result<Self, VmDomainError> {
        let name = name.into();
        Self::validate_common(&name, &resources)?;
        Ok(Self {
            id,
            name,
            resources,
            acceleration,
            state: VmState::Created,
            last_failure: None,
            guest_boot: GuestBootConfiguration::default(),
            disks: Vec::new(),
            networks: Vec::new(),
            snapshots: Vec::new(),
        })
    }

    pub fn reconstitute(
        id: VmId,
        name: impl Into<String>,
        resources: VmResourceConfig,
        acceleration: AccelerationBackend,
        state: VmState,
        last_failure: Option<VmFailure>,
        guest_boot: GuestBootConfiguration,
        disks: Vec<DiskAttachment>,
        networks: Vec<NetworkAttachment>,
        snapshots: Vec<SnapshotRecord>,
    ) -> Result<Self, VmDomainError> {
        let name = name.into();
        Self::validate_common(&name, &resources)?;

        for (index, attachment) in disks.iter().enumerate() {
            if disks[..index]
                .iter()
                .any(|existing| existing.image().id() == attachment.image().id())
            {
                return Err(VmDomainError::Disk(DiskDomainError::DiskAlreadyAttached(
                    attachment.image().id().clone(),
                )));
            }
        }

        for (index, attachment) in networks.iter().enumerate() {
            for existing in &networks[..index] {
                Self::validate_network_pair(existing, attachment)?;
            }
        }

        Ok(Self {
            id,
            name,
            resources,
            acceleration,
            state,
            last_failure,
            guest_boot,
            disks,
            networks,
            snapshots,
        })
    }

    fn validate_common(name: &str, resources: &VmResourceConfig) -> Result<(), VmDomainError> {
        if name.trim().is_empty() {
            return Err(VmDomainError::InvalidName);
        }
        if resources.vcpu_count == 0 {
            return Err(VmDomainError::InvalidVcpuCount);
        }
        if resources.memory_mib == 0 {
            return Err(VmDomainError::InvalidMemory);
        }
        Ok(())
    }

    pub fn id(&self) -> &VmId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn resources(&self) -> &VmResourceConfig {
        &self.resources
    }

    pub const fn acceleration(&self) -> AccelerationBackend {
        self.acceleration
    }

    pub const fn state(&self) -> VmState {
        self.state
    }

    pub const fn last_failure(&self) -> Option<&VmFailure> {
        self.last_failure.as_ref()
    }

    pub const fn guest_boot(&self) -> &GuestBootConfiguration {
        &self.guest_boot
    }

    pub fn guest_boot_mut(&mut self) -> &mut GuestBootConfiguration {
        &mut self.guest_boot
    }

    pub fn disks(&self) -> &[DiskAttachment] {
        &self.disks
    }

    pub fn networks(&self) -> &[NetworkAttachment] {
        &self.networks
    }

    pub fn snapshots(&self) -> &[SnapshotRecord] {
        &self.snapshots
    }

    pub fn snapshot(&self, snapshot_id: &SnapshotId) -> Result<&SnapshotRecord, SnapshotDomainError> {
        self.snapshots
            .iter()
            .find(|snapshot| snapshot.id() == snapshot_id)
            .ok_or_else(|| SnapshotDomainError::SnapshotNotFound(snapshot_id.clone()))
    }

    pub fn add_snapshot(&mut self, snapshot: SnapshotRecord) -> Result<(), SnapshotDomainError> {
        if self.snapshots.iter().any(|existing| existing.id() == snapshot.id()) {
            return Err(SnapshotDomainError::SnapshotAlreadyExists(snapshot.id().clone()));
        }
        self.snapshots.push(snapshot);
        Ok(())
    }

    pub fn remove_snapshot(&mut self, snapshot_id: &SnapshotId) -> Result<SnapshotRecord, SnapshotDomainError> {
        let index = self
            .snapshots
            .iter()
            .position(|snapshot| snapshot.id() == snapshot_id)
            .ok_or_else(|| SnapshotDomainError::SnapshotNotFound(snapshot_id.clone()))?;
        Ok(self.snapshots.remove(index))
    }

    pub fn network(&self, network_id: &NetworkId) -> Result<&NetworkAttachment, VmDomainError> {
        self.networks
            .iter()
            .find(|attachment| attachment.id() == network_id)
            .ok_or_else(|| {
                VmDomainError::Network(NetworkDomainError::NetworkNotAttached(
                    network_id.clone(),
                ))
            })
    }

    pub fn network_mut(&mut self, network_id: &NetworkId) -> Result<&mut NetworkAttachment, VmDomainError> {
        self.networks
            .iter_mut()
            .find(|attachment| attachment.id() == network_id)
            .ok_or_else(|| VmDomainError::Network(NetworkDomainError::NetworkNotAttached(network_id.clone())))
    }

    pub fn disk(&self, disk_id: &DiskId) -> Result<&DiskAttachment, VmDomainError> {
        self.disks
            .iter()
            .find(|attachment| attachment.image().id() == disk_id)
            .ok_or_else(|| VmDomainError::Disk(DiskDomainError::DiskNotAttached(disk_id.clone())))
    }

    pub fn configure_guest_boot(&mut self, configuration: GuestBootConfiguration) {
        self.guest_boot = configuration;
    }

    pub fn reconfigure(
        &mut self,
        name: impl Into<String>,
        resources: VmResourceConfig,
    ) -> Result<(), VmDomainError> {
        let name = name.into();
        Self::validate_common(&name, &resources)?;
        self.name = name;
        self.resources = resources;
        Ok(())
    }

    pub fn detach_disk(&mut self, disk_id: &DiskId) -> Result<DiskAttachment, VmDomainError> {
        let index = self
            .disks
            .iter()
            .position(|attachment| attachment.image().id() == disk_id)
            .ok_or_else(|| VmDomainError::Disk(DiskDomainError::DiskNotAttached(disk_id.clone())))?;
        Ok(self.disks.remove(index))
    }

    pub fn attach_disk(&mut self, attachment: DiskAttachment) -> Result<(), VmDomainError> {
        if self
            .disks
            .iter()
            .any(|existing| existing.image().id() == attachment.image().id())
        {
            return Err(VmDomainError::Disk(DiskDomainError::DiskAlreadyAttached(
                attachment.image().id().clone(),
            )));
        }

        self.disks.push(attachment);
        Ok(())
    }

    pub fn attach_network(&mut self, attachment: NetworkAttachment) -> Result<(), VmDomainError> {
        for existing in &self.networks {
            Self::validate_network_pair(existing, &attachment)?;
        }

        self.networks.push(attachment);
        Ok(())
    }

    pub fn detach_network(&mut self, network_id: &NetworkId) -> Result<NetworkAttachment, VmDomainError> {
        let index = self
            .networks
            .iter()
            .position(|attachment| attachment.id() == network_id)
            .ok_or_else(|| {
                VmDomainError::Network(NetworkDomainError::NetworkNotAttached(network_id.clone()))
            })?;
        Ok(self.networks.remove(index))
    }

    fn validate_network_pair(
        existing: &NetworkAttachment,
        candidate: &NetworkAttachment,
    ) -> Result<(), VmDomainError> {
        if existing.id() == candidate.id() {
            return Err(VmDomainError::Network(
                NetworkDomainError::NetworkAlreadyAttached(candidate.id().clone()),
            ));
        }

        if let (Some(existing_mac), Some(candidate_mac)) =
            (existing.mac_address(), candidate.mac_address())
        {
            if existing_mac == candidate_mac {
                return Err(VmDomainError::Network(
                    NetworkDomainError::DuplicateMacAddress(
                        candidate_mac.to_canonical_string(),
                    ),
                ));
            }
        }

        for existing_rule in existing.port_forwards() {
            for candidate_rule in candidate.port_forwards() {
                if existing_rule.conflicts_with(candidate_rule) {
                    return Err(VmDomainError::Network(
                        NetworkDomainError::DuplicateHostBinding {
                            protocol: candidate_rule.protocol(),
                            host_ip: candidate_rule.host_ip(),
                            host_port: candidate_rule.host_port(),
                        },
                    ));
                }
            }
        }

        Ok(())
    }

    pub fn grow_disk(
        &mut self,
        disk_id: &DiskId,
        new_virtual_size_bytes: u64,
    ) -> Result<(), VmDomainError> {
        let attachment = self
            .disks
            .iter_mut()
            .find(|attachment| attachment.image().id() == disk_id)
            .ok_or_else(|| VmDomainError::Disk(DiskDomainError::DiskNotAttached(disk_id.clone())))?;

        attachment
            .image_mut()
            .grow_to(new_virtual_size_bytes)
            .map_err(VmDomainError::Disk)
    }

    pub fn transition_to(&mut self, target: VmState) -> Result<(), VmDomainError> {
        if !self.state.can_transition_to(target) {
            return Err(VmDomainError::InvalidStateTransition {
                from: self.state,
                to: target,
            });
        }

        self.state = target;
        if target != VmState::Error {
            self.last_failure = None;
        }
        Ok(())
    }

    pub fn mark_failure(&mut self, failure: VmFailure) -> Result<(), VmDomainError> {
        self.transition_to(VmState::Error)?;
        self.last_failure = Some(failure);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::disk::{DiskBus, DiskFormat, DiskImage};
    use crate::domain::network::{
        MacAddress, NetworkDeviceModel, NetworkMode, PortForwardRule, PortProtocol,
    };

    fn machine() -> VirtualMachine {
        let id = VmId::parse("test-vm").expect("test id must be valid");
        VirtualMachine::create(
            id,
            "Test VM",
            VmResourceConfig {
                vcpu_count: 2,
                memory_mib: 2048,
            },
            AccelerationBackend::Tcg,
        )
        .expect("test machine must be valid")
    }

    #[test]
    fn machine_starts_in_created_state() {
        assert_eq!(machine().state(), VmState::Created);
    }

    #[test]
    fn invalid_transition_is_rejected() {
        let mut machine = machine();
        let error = machine
            .transition_to(VmState::Running)
            .expect_err("created machine cannot jump to running");

        assert_eq!(
            error,
            VmDomainError::InvalidStateTransition {
                from: VmState::Created,
                to: VmState::Running,
            }
        );
    }

    #[test]
    fn invalid_id_is_rejected() {
        assert_eq!(VmId::parse("bad id"), Err(VmDomainError::InvalidId));
    }

    #[test]
    fn duplicate_disk_attachment_is_rejected() {
        let mut machine = machine();
        let disk_id = DiskId::parse("system").expect("disk id must be valid");
        let image = DiskImage::create(
            disk_id,
            DiskFormat::Qcow2,
            1024,
            "disks/system.qcow2",
        )
        .expect("disk must be valid");
        let attachment = DiskAttachment::new(image, DiskBus::Virtio, Some(1));
        machine
            .attach_disk(attachment.clone())
            .expect("first attach must succeed");
        assert!(matches!(
            machine.attach_disk(attachment),
            Err(VmDomainError::Disk(DiskDomainError::DiskAlreadyAttached(_)))
        ));
    }

    #[test]
    fn duplicate_host_binding_across_adapters_is_rejected() {
        let mut machine = machine();
        let first_rule = PortForwardRule::create(PortProtocol::Tcp, None, 2222, None, 22)
            .expect("first rule must be valid");
        let second_rule = PortForwardRule::create(
            PortProtocol::Tcp,
            Some("127.0.0.1".parse().expect("ip must be valid")),
            2222,
            None,
            2223,
        )
        .expect("second rule must be valid");
        let first = NetworkAttachment::create(
            NetworkId::parse("net-a").expect("network id must be valid"),
            NetworkMode::UserNat,
            NetworkDeviceModel::VirtioNetPci,
            None,
            vec![first_rule],
            None,
        )
        .expect("first network must be valid");
        let second = NetworkAttachment::create(
            NetworkId::parse("net-b").expect("network id must be valid"),
            NetworkMode::UserNat,
            NetworkDeviceModel::VirtioNetPci,
            None,
            vec![second_rule],
            None,
        )
        .expect("second network must be valid");

        machine
            .attach_network(first)
            .expect("first network attach must succeed");
        assert!(matches!(
            machine.attach_network(second),
            Err(VmDomainError::Network(
                NetworkDomainError::DuplicateHostBinding { .. }
            ))
        ));
    }

    #[test]
    fn duplicate_explicit_mac_is_rejected() {
        let mut machine = machine();
        let mac = MacAddress::parse("52:54:00:12:34:56").expect("mac must be valid");
        let first = NetworkAttachment::create(
            NetworkId::parse("net-a").expect("network id must be valid"),
            NetworkMode::UserNat,
            NetworkDeviceModel::VirtioNetPci,
            Some(mac.clone()),
            Vec::new(),
            None,
        )
        .expect("first network must be valid");
        let second = NetworkAttachment::create(
            NetworkId::parse("net-b").expect("network id must be valid"),
            NetworkMode::UserNat,
            NetworkDeviceModel::E1000,
            Some(mac),
            Vec::new(),
            None,
        )
        .expect("second network must be valid");

        machine
            .attach_network(first)
            .expect("first network attach must succeed");
        assert!(matches!(
            machine.attach_network(second),
            Err(VmDomainError::Network(
                NetworkDomainError::DuplicateMacAddress(_)
            ))
        ));
    }

}
