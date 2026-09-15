// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/commands/update_network_service_command.rs
// # 📌 Amac: Managed NAT veya legacy User NAT uzerinde servis yayinlama degisikligini Service katmanina tasir
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: TCP/UDP host portunun guest porta eklenmesi veya kaldirilmasi icin typed command modellerini tanimlar
// # Bagimli Oldugu Katman: Controller | Service

use std::net::Ipv4Addr;

use crate::domain::network::PortProtocol;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishNetworkServiceCommand {
    pub vm_id: String,
    pub network_id: String,
    pub protocol: PortProtocol,
    pub host_ip: Option<Ipv4Addr>,
    pub host_port: u16,
    pub guest_port: u16,
}

impl PublishNetworkServiceCommand {
    pub fn new(
        vm_id: impl Into<String>,
        network_id: impl Into<String>,
        protocol: PortProtocol,
        host_ip: Option<Ipv4Addr>,
        host_port: u16,
        guest_port: u16,
    ) -> Self {
        Self { vm_id: vm_id.into(), network_id: network_id.into(), protocol, host_ip, host_port, guest_port }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnpublishNetworkServiceCommand {
    pub vm_id: String,
    pub network_id: String,
    pub protocol: PortProtocol,
    pub host_port: u16,
}

impl UnpublishNetworkServiceCommand {
    pub fn new(vm_id: impl Into<String>, network_id: impl Into<String>, protocol: PortProtocol, host_port: u16) -> Self {
        Self { vm_id: vm_id.into(), network_id: network_id.into(), protocol, host_port }
    }
}
