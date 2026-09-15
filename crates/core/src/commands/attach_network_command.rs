// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/commands/attach_network_command.rs
// # 📌 Amac: VM network attachment use-case girdisini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Managed NAT, private fabric, legacy User NAT ve bridge ayarlarini Controller katmanindan Service katmanina tasir
// # Bagimli Oldugu Katman: Controller | Service

use std::net::Ipv4Addr;

use crate::domain::network::{NetworkDeviceModel, NetworkMode, PortProtocol};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortForwardCommand {
    pub protocol: PortProtocol,
    pub host_ip: Option<Ipv4Addr>,
    pub host_port: u16,
    pub guest_ip: Option<Ipv4Addr>,
    pub guest_port: u16,
}

impl PortForwardCommand {
    pub const fn new(protocol: PortProtocol, host_ip: Option<Ipv4Addr>, host_port: u16, guest_ip: Option<Ipv4Addr>, guest_port: u16) -> Self {
        Self { protocol, host_ip, host_port, guest_ip, guest_port }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeConfigurationCommand {
    pub bridge_name: Option<String>,
    pub tap_name: Option<String>,
}
impl BridgeConfigurationCommand {
    pub fn new(bridge_name: Option<String>, tap_name: Option<String>) -> Self { Self { bridge_name, tap_name } }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedAddressCommand {
    pub ipv4_address: Ipv4Addr,
    pub prefix_length: u8,
    pub gateway: Option<Ipv4Addr>,
    pub dns_servers: Vec<Ipv4Addr>,
}
impl ManagedAddressCommand {
    pub fn new(ipv4_address: Ipv4Addr, prefix_length: u8, gateway: Option<Ipv4Addr>, dns_servers: Vec<Ipv4Addr>) -> Self {
        Self { ipv4_address, prefix_length, gateway, dns_servers }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachNetworkCommand {
    pub vm_id: String,
    pub network_id: String,
    pub mode: NetworkMode,
    pub device_model: NetworkDeviceModel,
    pub mac_address: Option<String>,
    pub port_forwards: Vec<PortForwardCommand>,
    pub bridge: Option<BridgeConfigurationCommand>,
    pub managed_address: Option<ManagedAddressCommand>,
    pub fabric_id: Option<String>,
}

impl AttachNetworkCommand {
    pub fn new_managed(
        vm_id: impl Into<String>, network_id: impl Into<String>, mode: NetworkMode,
        device_model: NetworkDeviceModel, mac_address: String, managed_address: ManagedAddressCommand, fabric_id: String,
    ) -> Self {
        debug_assert!(matches!(mode, NetworkMode::ManagedNat | NetworkMode::Private));
        Self { vm_id: vm_id.into(), network_id: network_id.into(), mode, device_model, mac_address: Some(mac_address), port_forwards: Vec::new(), bridge: None, managed_address: Some(managed_address), fabric_id: Some(fabric_id) }
    }

    pub fn new_user_nat(
        vm_id: impl Into<String>, network_id: impl Into<String>, device_model: NetworkDeviceModel,
        mac_address: Option<String>, port_forwards: Vec<PortForwardCommand>,
    ) -> Self {
        Self { vm_id: vm_id.into(), network_id: network_id.into(), mode: NetworkMode::UserNat, device_model, mac_address, port_forwards, bridge: None, managed_address: None, fabric_id: None }
    }

    pub fn new_bridge(
        vm_id: impl Into<String>, network_id: impl Into<String>, device_model: NetworkDeviceModel,
        mac_address: Option<String>, bridge_name: Option<String>, tap_name: Option<String>,
    ) -> Self {
        Self { vm_id: vm_id.into(), network_id: network_id.into(), mode: NetworkMode::Bridge, device_model, mac_address, port_forwards: Vec::new(), bridge: Some(BridgeConfigurationCommand::new(bridge_name, tap_name)), managed_address: None, fabric_id: None }
    }
}
