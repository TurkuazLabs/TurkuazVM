// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/domain/network.rs
// # 📌 Amac: TurkuazVM network attachment, managed IPv4, bridge hedefi, runtime binding ve servis yayinlama kurallarini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Managed NAT, private fabric, legacy User NAT ve host bridge senaryolari icin typed network modelini saglar
// # Bagimli Oldugu Katman: Service | Repo | Tool

use std::net::Ipv4Addr;
use std::path::PathBuf;

const MAX_HOST_NETWORK_NAME_LENGTH: usize = 128;
const MIN_MANAGED_PREFIX_LENGTH: u8 = 16;
const MAX_MANAGED_PREFIX_LENGTH: u8 = 30;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NetworkId(String);

impl NetworkId {
    pub fn parse(value: impl Into<String>) -> Result<Self, NetworkDomainError> {
        let value = value.into();
        let valid = !value.is_empty()
            && value.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
            });
        if !valid {
            return Err(NetworkDomainError::InvalidNetworkId);
        }
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str { &self.0 }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HostNetworkName(String);

impl HostNetworkName {
    pub fn parse(value: impl Into<String>) -> Result<Self, NetworkDomainError> {
        let value = value.into();
        let trimmed = value.trim();
        let valid = !trimmed.is_empty()
            && trimmed.len() <= MAX_HOST_NETWORK_NAME_LENGTH
            && !trimmed.contains(',')
            && !trimmed.chars().any(char::is_control);
        if !valid { return Err(NetworkDomainError::InvalidHostNetworkName); }
        Ok(Self(trimmed.to_owned()))
    }
    pub fn as_str(&self) -> &str { &self.0 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkMode {
    ManagedNat,
    Private,
    UserNat,
    Bridge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkDeviceModel { VirtioNetPci, E1000 }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortProtocol { Tcp, Udp }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedAddressConfiguration {
    ipv4_address: Ipv4Addr,
    prefix_length: u8,
    gateway: Option<Ipv4Addr>,
    dns_servers: Vec<Ipv4Addr>,
}

impl ManagedAddressConfiguration {
    pub fn create(
        ipv4_address: Ipv4Addr,
        prefix_length: u8,
        gateway: Option<Ipv4Addr>,
        dns_servers: Vec<Ipv4Addr>,
    ) -> Result<Self, NetworkDomainError> {
        if !ipv4_address.is_private() || ipv4_address.is_unspecified() || ipv4_address.is_broadcast() {
            return Err(NetworkDomainError::InvalidManagedIpv4Address);
        }
        if !(MIN_MANAGED_PREFIX_LENGTH..=MAX_MANAGED_PREFIX_LENGTH).contains(&prefix_length) {
            return Err(NetworkDomainError::InvalidManagedPrefixLength(prefix_length));
        }
        if let Some(gateway) = gateway {
            if !same_subnet(ipv4_address, gateway, prefix_length) || gateway == ipv4_address {
                return Err(NetworkDomainError::InvalidManagedGateway);
            }
        }
        if dns_servers.iter().any(|server| server.is_unspecified() || server.is_broadcast()) {
            return Err(NetworkDomainError::InvalidManagedDnsServer);
        }
        Ok(Self { ipv4_address, prefix_length, gateway, dns_servers })
    }

    pub const fn ipv4_address(&self) -> Ipv4Addr { self.ipv4_address }
    pub const fn prefix_length(&self) -> u8 { self.prefix_length }
    pub const fn gateway(&self) -> Option<Ipv4Addr> { self.gateway }
    pub fn dns_servers(&self) -> &[Ipv4Addr] { &self.dns_servers }
    pub fn subnet_address(&self) -> Ipv4Addr {
        Ipv4Addr::from(u32::from(self.ipv4_address) & prefix_mask(self.prefix_length))
    }
}

fn prefix_mask(prefix_length: u8) -> u32 {
    if prefix_length == 0 { 0 } else { u32::MAX << (32 - u32::from(prefix_length)) }
}

fn same_subnet(left: Ipv4Addr, right: Ipv4Addr, prefix_length: u8) -> bool {
    let mask = prefix_mask(prefix_length);
    (u32::from(left) & mask) == (u32::from(right) & mask)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeConfiguration {
    bridge_name: Option<HostNetworkName>,
    tap_name: Option<HostNetworkName>,
}

impl BridgeConfiguration {
    pub fn create(bridge_name: Option<HostNetworkName>, tap_name: Option<HostNetworkName>) -> Result<Self, NetworkDomainError> {
        if bridge_name.is_none() && tap_name.is_none() { return Err(NetworkDomainError::BridgeTargetRequired); }
        Ok(Self { bridge_name, tap_name })
    }
    pub fn bridge_name(&self) -> Option<&HostNetworkName> { self.bridge_name.as_ref() }
    pub fn tap_name(&self) -> Option<&HostNetworkName> { self.tap_name.as_ref() }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacAddress([u8; 6]);

impl MacAddress {
    pub fn parse(value: &str) -> Result<Self, NetworkDomainError> {
        let parts: Vec<&str> = value.split(':').collect();
        if parts.len() != 6 { return Err(NetworkDomainError::InvalidMacAddress); }
        let mut bytes = [0_u8; 6];
        for (index, part) in parts.iter().enumerate() {
            if part.len() != 2 { return Err(NetworkDomainError::InvalidMacAddress); }
            bytes[index] = u8::from_str_radix(part, 16).map_err(|_| NetworkDomainError::InvalidMacAddress)?;
        }
        if bytes == [0; 6] || bytes == [0xff; 6] || bytes[0] & 1 == 1 { return Err(NetworkDomainError::InvalidMacAddress); }
        Ok(Self(bytes))
    }
    pub fn to_canonical_string(&self) -> String {
        self.0.iter().map(|byte| format!("{byte:02x}")).collect::<Vec<_>>().join(":")
    }
    pub const fn octets(&self) -> [u8; 6] { self.0 }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortForwardRule {
    protocol: PortProtocol,
    host_ip: Option<Ipv4Addr>,
    host_port: u16,
    guest_ip: Option<Ipv4Addr>,
    guest_port: u16,
}

impl PortForwardRule {
    pub fn create(protocol: PortProtocol, host_ip: Option<Ipv4Addr>, host_port: u16, guest_ip: Option<Ipv4Addr>, guest_port: u16) -> Result<Self, NetworkDomainError> {
        if host_port == 0 { return Err(NetworkDomainError::InvalidHostPort); }
        if guest_port == 0 { return Err(NetworkDomainError::InvalidGuestPort); }
        Ok(Self { protocol, host_ip: host_ip.filter(|ip| !ip.is_unspecified()), host_port, guest_ip, guest_port })
    }
    pub const fn protocol(&self) -> PortProtocol { self.protocol }
    pub const fn host_ip(&self) -> Option<Ipv4Addr> { self.host_ip }
    pub const fn host_port(&self) -> u16 { self.host_port }
    pub const fn guest_ip(&self) -> Option<Ipv4Addr> { self.guest_ip }
    pub const fn guest_port(&self) -> u16 { self.guest_port }
    pub fn conflicts_with(&self, other: &Self) -> bool {
        let address_overlap = self.host_ip.is_none() || other.host_ip.is_none() || self.host_ip == other.host_ip;
        self.protocol == other.protocol && address_overlap && self.host_port == other.host_port
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkAttachment {
    id: NetworkId,
    mode: NetworkMode,
    device_model: NetworkDeviceModel,
    mac_address: Option<MacAddress>,
    port_forwards: Vec<PortForwardRule>,
    bridge: Option<BridgeConfiguration>,
    managed_address: Option<ManagedAddressConfiguration>,
    fabric_id: Option<NetworkId>,
}

impl NetworkAttachment {
    pub fn create(
        id: NetworkId,
        mode: NetworkMode,
        device_model: NetworkDeviceModel,
        mac_address: Option<MacAddress>,
        port_forwards: Vec<PortForwardRule>,
        bridge: Option<BridgeConfiguration>,
    ) -> Result<Self, NetworkDomainError> {
        Self::create_with_address(id, mode, device_model, mac_address, port_forwards, bridge, None, None)
    }

    pub fn create_with_address(
        id: NetworkId,
        mode: NetworkMode,
        device_model: NetworkDeviceModel,
        mac_address: Option<MacAddress>,
        port_forwards: Vec<PortForwardRule>,
        bridge: Option<BridgeConfiguration>,
        managed_address: Option<ManagedAddressConfiguration>,
        fabric_id: Option<NetworkId>,
    ) -> Result<Self, NetworkDomainError> {
        match mode {
            NetworkMode::ManagedNat => {
                if bridge.is_some() { return Err(NetworkDomainError::BridgeConfigurationNotAllowed); }
                if managed_address.is_none() { return Err(NetworkDomainError::ManagedAddressRequired); }
                if mac_address.is_none() { return Err(NetworkDomainError::ManagedMacRequired); }
                if fabric_id.is_none() { return Err(NetworkDomainError::ManagedFabricRequired); }
            }
            NetworkMode::Private => {
                if bridge.is_some() { return Err(NetworkDomainError::BridgeConfigurationNotAllowed); }
                if managed_address.is_none() { return Err(NetworkDomainError::ManagedAddressRequired); }
                if mac_address.is_none() { return Err(NetworkDomainError::ManagedMacRequired); }
                if fabric_id.is_none() { return Err(NetworkDomainError::ManagedFabricRequired); }
                if !port_forwards.is_empty() { return Err(NetworkDomainError::PortForwardNotAllowed); }
            }
            NetworkMode::UserNat => {
                if bridge.is_some() { return Err(NetworkDomainError::BridgeConfigurationNotAllowed); }
                if managed_address.is_some() { return Err(NetworkDomainError::ManagedAddressNotAllowed); }
            }
            NetworkMode::Bridge => {
                if !port_forwards.is_empty() { return Err(NetworkDomainError::PortForwardNotAllowed); }
                if bridge.is_none() { return Err(NetworkDomainError::BridgeConfigurationRequired); }
                if managed_address.is_some() { return Err(NetworkDomainError::ManagedAddressNotAllowed); }
            }
        }
        for (index, rule) in port_forwards.iter().enumerate() {
            if port_forwards[..index].iter().any(|existing| existing.conflicts_with(rule)) {
                return Err(NetworkDomainError::DuplicateHostBinding { protocol: rule.protocol(), host_ip: rule.host_ip(), host_port: rule.host_port() });
            }
        }
        Ok(Self { id, mode, device_model, mac_address, port_forwards, bridge, managed_address, fabric_id })
    }

    pub fn id(&self) -> &NetworkId { &self.id }
    pub const fn mode(&self) -> NetworkMode { self.mode }
    pub const fn device_model(&self) -> NetworkDeviceModel { self.device_model }
    pub const fn mac_address(&self) -> Option<&MacAddress> { self.mac_address.as_ref() }
    pub fn port_forwards(&self) -> &[PortForwardRule] { &self.port_forwards }
    pub fn bridge_configuration(&self) -> Option<&BridgeConfiguration> { self.bridge.as_ref() }
    pub fn managed_address(&self) -> Option<&ManagedAddressConfiguration> { self.managed_address.as_ref() }
    pub fn fabric_id(&self) -> Option<&NetworkId> { self.fabric_id.as_ref() }

    pub fn add_port_forward(&mut self, rule: PortForwardRule) -> Result<(), NetworkDomainError> {
        if !matches!(self.mode, NetworkMode::ManagedNat | NetworkMode::UserNat) {
            return Err(NetworkDomainError::PortForwardNotAllowed);
        }
        if self.port_forwards.iter().any(|existing| existing.conflicts_with(&rule)) {
            return Err(NetworkDomainError::DuplicateHostBinding { protocol: rule.protocol(), host_ip: rule.host_ip(), host_port: rule.host_port() });
        }
        self.port_forwards.push(rule);
        Ok(())
    }

    pub fn remove_port_forward(&mut self, protocol: PortProtocol, host_port: u16) -> Result<PortForwardRule, NetworkDomainError> {
        let index = self.port_forwards.iter().position(|rule| rule.protocol() == protocol && rule.host_port() == host_port)
            .ok_or(NetworkDomainError::PortForwardNotFound { protocol, host_port })?;
        Ok(self.port_forwards.remove(index))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreparedNetworkBackend {
    UserNat,
    HostTap { interface_name: HostNetworkName },
    HostBridge { bridge_name: HostNetworkName, helper_path: Option<PathBuf> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedNetworkBinding { network_id: NetworkId, backend: PreparedNetworkBackend }
impl PreparedNetworkBinding {
    pub fn new(network_id: NetworkId, backend: PreparedNetworkBackend) -> Self { Self { network_id, backend } }
    pub fn network_id(&self) -> &NetworkId { &self.network_id }
    pub fn backend(&self) -> &PreparedNetworkBackend { &self.backend }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NetworkRuntimePlan { bindings: Vec<PreparedNetworkBinding> }
impl NetworkRuntimePlan {
    pub fn new(bindings: Vec<PreparedNetworkBinding>) -> Result<Self, NetworkDomainError> {
        for (index, binding) in bindings.iter().enumerate() {
            if bindings[..index].iter().any(|existing| existing.network_id() == binding.network_id()) {
                return Err(NetworkDomainError::DuplicatePreparedBinding(binding.network_id().clone()));
            }
        }
        Ok(Self { bindings })
    }
    pub fn bindings(&self) -> &[PreparedNetworkBinding] { &self.bindings }
    pub fn binding(&self, network_id: &NetworkId) -> Option<&PreparedNetworkBinding> {
        self.bindings.iter().find(|binding| binding.network_id() == network_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkDomainError {
    InvalidNetworkId,
    InvalidHostNetworkName,
    InvalidMacAddress,
    InvalidHostPort,
    InvalidGuestPort,
    InvalidManagedIpv4Address,
    InvalidManagedPrefixLength(u8),
    InvalidManagedGateway,
    InvalidManagedDnsServer,
    ManagedAddressRequired,
    ManagedAddressNotAllowed,
    ManagedMacRequired,
    ManagedFabricRequired,
    BridgeTargetRequired,
    BridgeConfigurationRequired,
    BridgeConfigurationNotAllowed,
    PortForwardNotAllowed,
    DuplicateHostBinding { protocol: PortProtocol, host_ip: Option<Ipv4Addr>, host_port: u16 },
    PortForwardNotFound { protocol: PortProtocol, host_port: u16 },
    NetworkAlreadyAttached(NetworkId),
    NetworkNotAttached(NetworkId),
    DuplicateMacAddress(String),
    DuplicatePreparedBinding(NetworkId),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_nat_requires_private_address_and_mac() {
        let address = ManagedAddressConfiguration::create(
            "192.168.240.10".parse().expect("ip"), 24,
            Some("192.168.240.1".parse().expect("gateway")),
            vec!["192.168.240.1".parse().expect("dns")],
        ).expect("address");
        let attachment = NetworkAttachment::create_with_address(
            NetworkId::parse("turkuaz-net-01").expect("id"), NetworkMode::ManagedNat,
            NetworkDeviceModel::VirtioNetPci,
            Some(MacAddress::parse("02:54:00:12:34:56").expect("mac")), Vec::new(), None, Some(address),
            Some(NetworkId::parse("turkuaz-net-01").expect("fabric")),
        );
        assert!(attachment.is_ok());
    }

    #[test]
    fn duplicate_host_binding_is_rejected() {
        let first = PortForwardRule::create(PortProtocol::Tcp, None, 2222, None, 22).expect("rule");
        let second = PortForwardRule::create(PortProtocol::Tcp, None, 2222, None, 2223).expect("rule");
        let result = NetworkAttachment::create(
            NetworkId::parse("default").expect("id"), NetworkMode::UserNat,
            NetworkDeviceModel::VirtioNetPci, None, vec![first, second], None,
        );
        assert!(matches!(result, Err(NetworkDomainError::DuplicateHostBinding { .. })));
    }

    #[test]
    fn bridge_requires_explicit_target() {
        let result = NetworkAttachment::create(
            NetworkId::parse("bridge0").expect("id"), NetworkMode::Bridge,
            NetworkDeviceModel::VirtioNetPci, None, Vec::new(), None,
        );
        assert_eq!(result, Err(NetworkDomainError::BridgeConfigurationRequired));
    }
}
