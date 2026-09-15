// # 📄 Dosya Yolu: /turkuazvm/crates/platform/src/tools/native_network_tool.rs
// # 📌 Amac: Windows ve Linux host network backendlerini preflight eder, runtime icin hazirlar ve crash recovery yapar
// # 📌 Modul - Rust
// # Version: 0.32.0
// # Aciklama: QEMU User NAT tabanli portable Turkuaz NAT ile ileri seviye private/TAP/bridge stratejilerini NetworkPort contractina adapte eder
// # Bagimli Oldugu Katman: Service | Tool

use std::collections::HashMap;
use std::fs;
use std::net::{Ipv4Addr, UdpSocket};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use turkuazvm_core::domain::network::{
    HostNetworkName, NetworkAttachment, NetworkMode, NetworkRuntimePlan, PreparedNetworkBackend,
    PreparedNetworkBinding,
};
use turkuazvm_core::domain::virtual_machine::{VirtualMachine, VmId};
use turkuazvm_core::ports::network_port::{
    NetworkCapabilities, NetworkError, NetworkPort, NetworkRecoveryReport,
};

const LEASE_SCHEMA_VERSION: u16 = 1;
const DIR_RUNTIME: &str = "runtime";
const DIR_NETWORK: &str = "network";
const LEASE_EXTENSION: &str = "yml";
const LINUX_SYS_CLASS_NET: &str = "/sys/class/net";
const LINUX_PROC_ROOT: &str = "/proc";
const COMMAND_IP: &str = "ip";
const COMMAND_ID: &str = "id";
const COMMAND_POWERSHELL: &str = "powershell.exe";
const LINUX_ROOT_UID: &str = "0";
const MAX_LINUX_INTERFACE_NAME_LENGTH: usize = 15;
const MAX_WINDOWS_HELPER_DIAGNOSTIC_CHARS: usize = 2400;

#[derive(Debug, Clone)]
pub struct NativeNetworkSettings {
    pub state_root: PathBuf,
    pub linux_bridge_helper: Option<PathBuf>,
    pub linux_allow_managed_tap: bool,
    pub windows_managed_helper: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum LeaseKind {
    UserNat,
    ExistingTap,
    ManagedTap,
    HostBridge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LeaseRecord {
    network_id: String,
    kind: LeaseKind,
    interface_name: Option<String>,
    bridge_name: Option<String>,
    managed_guest_ip: Option<String>,
    owned_by_turkuazvm: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NetworkLeaseFile {
    schema_version: u16,
    vm_id: String,
    process_id: Option<u32>,
    leases: Vec<LeaseRecord>,
}

#[derive(Debug, Clone)]
struct ManagedDhcpLease {
    address: Ipv4Addr,
    prefix_length: u8,
    gateway: Option<Ipv4Addr>,
    dns_servers: Vec<Ipv4Addr>,
}

static MANAGED_DHCP_LEASES: OnceLock<Mutex<HashMap<[u8; 6], ManagedDhcpLease>>> = OnceLock::new();
static MANAGED_DHCP_SERVER_STARTED: OnceLock<Result<(), String>> = OnceLock::new();

pub struct NativeNetworkTool {
    settings: NativeNetworkSettings,
}

impl NativeNetworkTool {
    pub fn new(settings: NativeNetworkSettings) -> Self {
        Self { settings }
    }

    fn lease_directory(&self) -> PathBuf {
        self.settings
            .state_root
            .join(DIR_RUNTIME)
            .join(DIR_NETWORK)
    }

    fn lease_path(&self, vm_id: &VmId) -> PathBuf {
        self.lease_directory()
            .join(format!("{}.{}", vm_id.as_str(), LEASE_EXTENSION))
    }

    fn validate_bridge_attachment(&self, attachment: &NetworkAttachment) -> Result<(), NetworkError> {
        let bridge = attachment.bridge_configuration().ok_or_else(|| {
            NetworkError::InvalidConfiguration(String::from(
                "Bridge network requires bridge configuration",
            ))
        })?;

        match std::env::consts::OS {
            "windows" => {
                if bridge.tap_name().is_none() {
                    return Err(NetworkError::InvalidConfiguration(String::from(
                        "Windows bridge mode requires an existing TAP adapter name",
                    )));
                }
                Ok(())
            }
            "linux" => {
                if bridge.tap_name().is_none() && bridge.bridge_name().is_none() {
                    return Err(NetworkError::InvalidConfiguration(String::from(
                        "Linux bridge mode requires a TAP interface or bridge name",
                    )));
                }
                Ok(())
            }
            _ => Err(NetworkError::UnsupportedMode(NetworkMode::Bridge)),
        }
    }

    fn prepare_windows_bridge(
        &self,
        attachment: &NetworkAttachment,
    ) -> Result<(PreparedNetworkBinding, LeaseRecord), NetworkError> {
        let bridge = attachment.bridge_configuration().ok_or_else(|| {
            NetworkError::InvalidConfiguration(String::from("Missing bridge configuration"))
        })?;
        let tap_name = bridge.tap_name().ok_or_else(|| {
            NetworkError::InvalidConfiguration(String::from(
                "Windows bridge mode requires an existing TAP adapter",
            ))
        })?;

        Self::ensure_windows_adapter_exists(tap_name)?;

        Ok((
            PreparedNetworkBinding::new(
                attachment.id().clone(),
                PreparedNetworkBackend::HostTap {
                    interface_name: tap_name.clone(),
                },
            ),
            LeaseRecord {
                network_id: attachment.id().as_str().to_owned(),
                kind: LeaseKind::ExistingTap,
                interface_name: Some(tap_name.as_str().to_owned()),
                bridge_name: bridge.bridge_name().map(|name| name.as_str().to_owned()),
                managed_guest_ip: None,
                owned_by_turkuazvm: false,
            },
        ))
    }

    fn prepare_linux_bridge(
        &self,
        vm_id: &VmId,
        index: usize,
        attachment: &NetworkAttachment,
    ) -> Result<(PreparedNetworkBinding, LeaseRecord), NetworkError> {
        let bridge = attachment.bridge_configuration().ok_or_else(|| {
            NetworkError::InvalidConfiguration(String::from("Missing bridge configuration"))
        })?;

        if let Some(tap_name) = bridge.tap_name() {
            Self::ensure_linux_interface_exists(tap_name)?;
            return Ok((
                PreparedNetworkBinding::new(
                    attachment.id().clone(),
                    PreparedNetworkBackend::HostTap {
                        interface_name: tap_name.clone(),
                    },
                ),
                LeaseRecord {
                    network_id: attachment.id().as_str().to_owned(),
                    kind: LeaseKind::ExistingTap,
                    interface_name: Some(tap_name.as_str().to_owned()),
                    bridge_name: bridge.bridge_name().map(|name| name.as_str().to_owned()),
                    managed_guest_ip: None,
                    owned_by_turkuazvm: false,
                },
            ));
        }

        let bridge_name = bridge.bridge_name().ok_or_else(|| {
            NetworkError::InvalidConfiguration(String::from("Linux bridge name is required"))
        })?;
        Self::ensure_linux_bridge_exists(bridge_name)?;

        if let Some(helper_path) = self.settings.linux_bridge_helper.as_ref() {
            if helper_path.is_file() {
                return Ok((
                    PreparedNetworkBinding::new(
                        attachment.id().clone(),
                        PreparedNetworkBackend::HostBridge {
                            bridge_name: bridge_name.clone(),
                            helper_path: Some(helper_path.clone()),
                        },
                    ),
                    LeaseRecord {
                        network_id: attachment.id().as_str().to_owned(),
                        kind: LeaseKind::HostBridge,
                        interface_name: None,
                        bridge_name: Some(bridge_name.as_str().to_owned()),
                        managed_guest_ip: None,
                        owned_by_turkuazvm: false,
                    },
                ));
            }
        }

        if !self.settings.linux_allow_managed_tap {
            return Err(NetworkError::PreflightFailed(String::from(
                "No Linux bridge helper is configured and managed TAP creation is disabled",
            )));
        }

        Self::require_linux_root()?;
        Self::ensure_linux_ip_available()?;
        let tap_name = Self::managed_tap_name(vm_id, index)?;
        Self::create_linux_managed_tap(&tap_name, bridge_name)?;

        Ok((
            PreparedNetworkBinding::new(
                attachment.id().clone(),
                PreparedNetworkBackend::HostTap {
                    interface_name: tap_name.clone(),
                },
            ),
            LeaseRecord {
                network_id: attachment.id().as_str().to_owned(),
                kind: LeaseKind::ManagedTap,
                interface_name: Some(tap_name.as_str().to_owned()),
                bridge_name: Some(bridge_name.as_str().to_owned()),
                managed_guest_ip: None,
                owned_by_turkuazvm: true,
            },
        ))
    }

    fn prepare_bridge(
        &self,
        vm_id: &VmId,
        index: usize,
        attachment: &NetworkAttachment,
    ) -> Result<(PreparedNetworkBinding, LeaseRecord), NetworkError> {
        match std::env::consts::OS {
            "windows" => self.prepare_windows_bridge(attachment),
            "linux" => self.prepare_linux_bridge(vm_id, index, attachment),
            _ => Err(NetworkError::UnsupportedMode(NetworkMode::Bridge)),
        }
    }


    fn windows_managed_tap_name(vm_id: &VmId, index: usize) -> Result<HostNetworkName, NetworkError> {
        let hash = Self::fnv1a_32(vm_id.as_str().as_bytes());
        HostNetworkName::parse(format!("TurkuazVM-{hash:08x}-{index:02}"))
            .map_err(|error| NetworkError::PreparationFailed(format!("{error:?}")))
    }

    fn windows_managed_helper(&self) -> Result<&Path, NetworkError> {
        self.settings
            .windows_managed_helper
            .as_deref()
            .filter(|path| path.is_file())
            .ok_or_else(|| NetworkError::PreflightFailed(String::from(
                "Windows managed network helper is not configured or missing",
            )))
    }

    fn invoke_windows_managed_helper(
        &self,
        action: &str,
        tap_name: Option<&HostNetworkName>,
        attachment: &NetworkAttachment,
        forward: Option<&turkuazvm_core::domain::network::PortForwardRule>,
    ) -> Result<(), NetworkError> {
        let helper = self.windows_managed_helper()?;
        let address = attachment.managed_address().ok_or_else(|| {
            NetworkError::InvalidConfiguration(String::from("Managed network address is missing"))
        })?;
        let gateway = address.gateway().ok_or_else(|| {
            NetworkError::InvalidConfiguration(String::from("Managed NAT gateway is missing"))
        })?;
        let subnet = format!("{}/{}", address.subnet_address(), address.prefix_length());
        let mut command = Command::new(COMMAND_POWERSHELL);
        command.args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
        ]);
        command.arg(helper);
        command.arg("-Action").arg(action);
        command.arg("-FabricId").arg(attachment.fabric_id().map(|id| id.as_str()).unwrap_or(attachment.id().as_str()));
        command.arg("-SubnetCidr").arg(subnet);
        command.arg("-GatewayIp").arg(gateway.to_string());
        command.arg("-EnableNat").arg(if attachment.mode() == NetworkMode::ManagedNat { "true" } else { "false" });
        command.arg("-GuestIp").arg(address.ipv4_address().to_string());
        command.arg("-StateRoot").arg(&self.settings.state_root);
        if let Some(tap_name) = tap_name {
            command.arg("-TapName").arg(tap_name.as_str());
        }
        if let Some(forward) = forward {
            command.arg("-Protocol").arg(match forward.protocol() {
                turkuazvm_core::domain::network::PortProtocol::Tcp => "TCP",
                turkuazvm_core::domain::network::PortProtocol::Udp => "UDP",
            });
            command.arg("-HostPort").arg(forward.host_port().to_string());
            command.arg("-GuestPort").arg(forward.guest_port().to_string());
        }
        let output = command
            .stdin(Stdio::null())
            .output()
            .map_err(|error| NetworkError::PreparationFailed(error.to_string()))?;
        if output.status.success() { return Ok(()); }

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        let mut diagnostics = [stderr, stdout]
            .into_iter()
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>()
            .join(" | ");
        if diagnostics.chars().count() > MAX_WINDOWS_HELPER_DIAGNOSTIC_CHARS {
            diagnostics = diagnostics
                .chars()
                .take(MAX_WINDOWS_HELPER_DIAGNOSTIC_CHARS)
                .collect::<String>();
            diagnostics.push_str("...");
        }
        let exit_code = output.status.code().map_or(String::from("terminated"), |value| value.to_string());
        if diagnostics.is_empty() {
            diagnostics = String::from("helper returned no diagnostic output");
        }
        Err(NetworkError::PreparationFailed(format!(
            "Windows managed network helper failed: action={action} network={} exit={exit_code} detail={diagnostics}",
            attachment.id().as_str()
        )))
    }

    fn register_managed_dhcp_lease(attachment: &NetworkAttachment) -> Result<(), NetworkError> {
        let mac = attachment.mac_address().ok_or_else(|| {
            NetworkError::InvalidConfiguration(String::from("Managed network MAC is missing"))
        })?;
        let address = attachment.managed_address().ok_or_else(|| {
            NetworkError::InvalidConfiguration(String::from("Managed network address is missing"))
        })?;
        let leases = MANAGED_DHCP_LEASES.get_or_init(|| Mutex::new(HashMap::new()));
        leases.lock().map_err(|_| NetworkError::PreparationFailed(String::from("DHCP lease lock poisoned")))?
            .insert(mac.octets(), ManagedDhcpLease {
                address: address.ipv4_address(),
                prefix_length: address.prefix_length(),
                gateway: address.gateway(),
                dns_servers: address.dns_servers().to_vec(),
            });
        Self::ensure_managed_dhcp_server()?;
        Ok(())
    }

    fn ensure_managed_dhcp_server() -> Result<(), NetworkError> {
        let state = MANAGED_DHCP_SERVER_STARTED.get_or_init(|| {
            let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 67))
                .map_err(|error| format!("Managed DHCP UDP/67 bind failed: {error}"))?;
            socket
                .set_broadcast(true)
                .map_err(|error| format!("Managed DHCP broadcast setup failed: {error}"))?;
            socket
                .set_read_timeout(Some(Duration::from_millis(500)))
                .map_err(|error| format!("Managed DHCP timeout setup failed: {error}"))?;
            thread::spawn(move || {
                let mut buffer = [0_u8; 1500];
                loop {
                    let Ok((size, _)) = socket.recv_from(&mut buffer) else { continue; };
                    if size < 240 || buffer[0] != 1 || buffer[1] != 1 || buffer[2] < 6 { continue; }
                    let mac = [buffer[28], buffer[29], buffer[30], buffer[31], buffer[32], buffer[33]];
                    let Some(message_type) = Self::dhcp_message_type(&buffer[..size]) else { continue; };
                    if !matches!(message_type, 1 | 3) { continue; }
                    let lease = MANAGED_DHCP_LEASES.get()
                        .and_then(|leases| leases.lock().ok())
                        .and_then(|leases| leases.get(&mac).cloned());
                    let Some(lease) = lease else { continue; };
                    let response_type = if message_type == 1 { 2 } else { 5 };
                    let response = Self::build_dhcp_response(&buffer[..size], &lease, response_type);
                    let _ = socket.send_to(&response, (Ipv4Addr::BROADCAST, 68));
                }
            });
            Ok(())
        });
        state.clone().map_err(NetworkError::PreparationFailed)
    }

    fn dhcp_message_type(packet: &[u8]) -> Option<u8> {
        if packet.len() < 240 || packet[236..240] != [99, 130, 83, 99] { return None; }
        let mut index = 240;
        while index < packet.len() {
            let code = packet[index];
            index += 1;
            if code == 255 { break; }
            if code == 0 { continue; }
            if index >= packet.len() { break; }
            let length = usize::from(packet[index]);
            index += 1;
            if index + length > packet.len() { break; }
            if code == 53 && length == 1 { return Some(packet[index]); }
            index += length;
        }
        None
    }

    fn build_dhcp_response(request: &[u8], lease: &ManagedDhcpLease, message_type: u8) -> Vec<u8> {
        let mut response = vec![0_u8; 240];
        response[0] = 2;
        response[1] = 1;
        response[2] = 6;
        response[3] = 0;
        response[4..8].copy_from_slice(&request[4..8]);
        response[10..12].copy_from_slice(&request[10..12]);
        response[16..20].copy_from_slice(&lease.address.octets());
        let server = lease.gateway.unwrap_or(lease.address);
        response[20..24].copy_from_slice(&server.octets());
        response[28..44].copy_from_slice(&request[28..44]);
        response[236..240].copy_from_slice(&[99, 130, 83, 99]);
        response.extend_from_slice(&[53, 1, message_type]);
        response.extend_from_slice(&[54, 4]);
        response.extend_from_slice(&server.octets());
        response.extend_from_slice(&[1, 4]);
        let mask = if lease.prefix_length == 0 { 0 } else { u32::MAX << (32 - u32::from(lease.prefix_length)) };
        response.extend_from_slice(&Ipv4Addr::from(mask).octets());
        if let Some(gateway) = lease.gateway {
            response.extend_from_slice(&[3, 4]);
            response.extend_from_slice(&gateway.octets());
        }
        if !lease.dns_servers.is_empty() {
            let length = lease.dns_servers.len().saturating_mul(4).min(252) as u8;
            response.extend_from_slice(&[6, length]);
            for dns in lease.dns_servers.iter().take(usize::from(length) / 4) {
                response.extend_from_slice(&dns.octets());
            }
        }
        response.extend_from_slice(&[51, 4, 0, 1, 81, 128]);
        response.push(255);
        response
    }

    fn prepare_windows_managed(
        &self,
        vm_id: &VmId,
        index: usize,
        attachment: &NetworkAttachment,
    ) -> Result<(PreparedNetworkBinding, LeaseRecord), NetworkError> {
        let tap_name = Self::windows_managed_tap_name(vm_id, index)?;
        self.invoke_windows_managed_helper("ensure", Some(&tap_name), attachment, None)?;
        for forward in attachment.port_forwards() {
            self.invoke_windows_managed_helper("publish", Some(&tap_name), attachment, Some(forward))?;
        }
        Self::register_managed_dhcp_lease(attachment)?;
        let guest_ip = attachment.managed_address().map(|address| address.ipv4_address().to_string());
        Ok((
            PreparedNetworkBinding::new(
                attachment.id().clone(),
                PreparedNetworkBackend::HostTap { interface_name: tap_name.clone() },
            ),
            LeaseRecord {
                network_id: attachment.fabric_id().map(|id| id.as_str()).unwrap_or(attachment.id().as_str()).to_owned(),
                kind: LeaseKind::ManagedTap,
                interface_name: Some(tap_name.as_str().to_owned()),
                bridge_name: Some(format!("TurkuazVM-{}", attachment.fabric_id().map(|id| id.as_str()).unwrap_or(attachment.id().as_str()))),
                managed_guest_ip: guest_ip,
                owned_by_turkuazvm: true,
            },
        ))
    }

    fn save_lease_file(&self, lease: &NetworkLeaseFile) -> Result<(), NetworkError> {
        let vm_id = VmId::parse(lease.vm_id.clone())
            .map_err(|error| NetworkError::RuntimeBindingFailed(format!("{error:?}")))?;
        let directory = self.lease_directory();
        fs::create_dir_all(&directory)
            .map_err(|error| NetworkError::RuntimeBindingFailed(error.to_string()))?;
        let path = self.lease_path(&vm_id);
        let yaml = serde_yaml_ng::to_string(lease)
            .map_err(|error| NetworkError::RuntimeBindingFailed(error.to_string()))?;
        let header = format!(
            "# \u{1F4C4} Dosya Yolu: {}\n# \u{1F4CC} Amac: VM host network runtime lease ve cleanup sahipligini saklar\n# \u{1F4CC} Modul - YAML\n# Version: {}\n# Aciklama: Crash recovery icin portable QEMU NAT, Advanced TAP/fabric lease ve QEMU process bilgisini saklar\n# Bagimli Oldugu Katman: Tool\n\n",
            path.display(),
            env!("CARGO_PKG_VERSION")
        );
        fs::write(path, format!("{header}{yaml}"))
            .map_err(|error| NetworkError::RuntimeBindingFailed(error.to_string()))
    }

    fn load_lease_file(&self, vm_id: &VmId) -> Result<Option<NetworkLeaseFile>, NetworkError> {
        let path = self.lease_path(vm_id);
        if !path.is_file() {
            return Ok(None);
        }
        let content = fs::read_to_string(&path)
            .map_err(|error| NetworkError::RecoveryFailed(error.to_string()))?;
        let lease: NetworkLeaseFile = serde_yaml_ng::from_str(&content)
            .map_err(|error| NetworkError::RecoveryFailed(error.to_string()))?;
        if lease.schema_version != LEASE_SCHEMA_VERSION {
            return Err(NetworkError::RecoveryFailed(format!(
                "Unsupported network lease schema version: {}",
                lease.schema_version
            )));
        }
        Ok(Some(lease))
    }

    fn cleanup_records(&self, records: &[LeaseRecord]) -> Result<(), NetworkError> {
        for record in records.iter().rev() {
            if !record.owned_by_turkuazvm || record.kind != LeaseKind::ManagedTap {
                continue;
            }
            let Some(interface_name) = record.interface_name.as_deref() else { continue; };
            match std::env::consts::OS {
                "windows" => {
                    let helper = self.windows_managed_helper()?;
                    let status = Command::new(COMMAND_POWERSHELL)
                        .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-File"])
                        .arg(helper)
                        .arg("-Action").arg("cleanup")
                        .arg("-TapName").arg(interface_name)
                        .arg("-FabricId").arg(&record.network_id)
                        .arg("-GuestIp").arg(record.managed_guest_ip.as_deref().unwrap_or(""))
                        .arg("-StateRoot").arg(&self.settings.state_root)
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status()
                        .map_err(|error| NetworkError::CleanupFailed(error.to_string()))?;
                    if !status.success() {
                        return Err(NetworkError::CleanupFailed(format!("Windows managed TAP cleanup failed: {interface_name}")));
                    }
                }
                "linux" => Self::delete_linux_interface_if_present(interface_name)?,
                _ => {}
            }
        }
        Ok(())
    }

    fn cleanup_lease_file(&self, lease: &NetworkLeaseFile) -> Result<(), NetworkError> {
        self.cleanup_records(&lease.leases)?;
        self.remove_lease_file_only(lease)
    }

    fn remove_lease_file_only(&self, lease: &NetworkLeaseFile) -> Result<(), NetworkError> {
        let vm_id = VmId::parse(lease.vm_id.clone())
            .map_err(|error| NetworkError::CleanupFailed(format!("{error:?}")))?;
        let path = self.lease_path(&vm_id);
        if path.exists() {
            fs::remove_file(path)
                .map_err(|error| NetworkError::CleanupFailed(error.to_string()))?;
        }
        Ok(())
    }

    fn cleanup_previous_lease_for_machine(
        &self,
        machine: &VirtualMachine,
        lease: &NetworkLeaseFile,
    ) -> Result<(), NetworkError> {
        match self.cleanup_lease_file(lease) {
            Ok(()) => Ok(()),
            Err(error) => {
                let portable_only = machine
                    .networks()
                    .iter()
                    .all(|attachment| matches!(attachment.mode(), NetworkMode::ManagedNat | NetworkMode::UserNat));
                let legacy_managed_tap = lease
                    .leases
                    .iter()
                    .any(|record| record.kind == LeaseKind::ManagedTap && record.owned_by_turkuazvm);
                if portable_only && legacy_managed_tap {
                    self.remove_lease_file_only(lease)
                } else {
                    Err(error)
                }
            }
        }
    }

    fn ensure_windows_adapter_exists(name: &HostNetworkName) -> Result<(), NetworkError> {
        let status = Command::new(COMMAND_POWERSHELL)
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "$adapter = Get-NetAdapter -Name $env:TURKUAZVM_TAP_NAME -ErrorAction Stop; if ($adapter.Status -eq 'Disabled') { exit 3 }; exit 0",
            ])
            .env("TURKUAZVM_TAP_NAME", name.as_str())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|error| NetworkError::PreflightFailed(error.to_string()))?;

        if status.success() {
            return Ok(());
        }
        Err(NetworkError::PreflightFailed(format!(
            "Windows TAP adapter is missing or disabled: {}",
            name.as_str()
        )))
    }

    fn ensure_linux_interface_exists(name: &HostNetworkName) -> Result<(), NetworkError> {
        if Path::new(LINUX_SYS_CLASS_NET).join(name.as_str()).exists() {
            return Ok(());
        }
        Err(NetworkError::PreflightFailed(format!(
            "Linux network interface not found: {}",
            name.as_str()
        )))
    }

    fn ensure_linux_bridge_exists(name: &HostNetworkName) -> Result<(), NetworkError> {
        let bridge_path = Path::new(LINUX_SYS_CLASS_NET)
            .join(name.as_str())
            .join("bridge");
        if bridge_path.is_dir() {
            return Ok(());
        }
        Err(NetworkError::PreflightFailed(format!(
            "Linux bridge not found: {}",
            name.as_str()
        )))
    }

    fn require_linux_root() -> Result<(), NetworkError> {
        let output = Command::new(COMMAND_ID)
            .arg("-u")
            .output()
            .map_err(|error| NetworkError::PrivilegeRequired(error.to_string()))?;
        let uid = String::from_utf8_lossy(&output.stdout);
        if output.status.success() && uid.trim() == LINUX_ROOT_UID {
            return Ok(());
        }
        Err(NetworkError::PrivilegeRequired(String::from(
            "Managed TAP creation currently requires root",
        )))
    }

    fn ensure_linux_ip_available() -> Result<(), NetworkError> {
        let status = Command::new(COMMAND_IP)
            .args(["link", "show"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|error| NetworkError::PreflightFailed(error.to_string()))?;
        if status.success() {
            return Ok(());
        }
        Err(NetworkError::PreflightFailed(String::from(
            "Linux ip command is unavailable",
        )))
    }

    fn managed_tap_name(vm_id: &VmId, index: usize) -> Result<HostNetworkName, NetworkError> {
        let hash = Self::fnv1a_32(vm_id.as_str().as_bytes());
        let name = format!("tv{hash:08x}{index:02}");
        let name = name
            .chars()
            .take(MAX_LINUX_INTERFACE_NAME_LENGTH)
            .collect::<String>();
        HostNetworkName::parse(name)
            .map_err(|error| NetworkError::PreparationFailed(format!("{error:?}")))
    }

    fn fnv1a_32(bytes: &[u8]) -> u32 {
        let mut hash = 0x811c_9dc5_u32;
        let mut index = 0;
        while index < bytes.len() {
            hash ^= bytes[index] as u32;
            hash = hash.wrapping_mul(0x0100_0193);
            index += 1;
        }
        hash
    }

    fn create_linux_managed_tap(
        tap_name: &HostNetworkName,
        bridge_name: &HostNetworkName,
    ) -> Result<(), NetworkError> {
        Self::run_ip([
            "tuntap",
            "add",
            "dev",
            tap_name.as_str(),
            "mode",
            "tap",
        ])?;

        if let Err(error) = Self::run_ip([
            "link",
            "set",
            "dev",
            tap_name.as_str(),
            "master",
            bridge_name.as_str(),
        ]) {
            let _ = Self::delete_linux_interface_if_present(tap_name.as_str());
            return Err(error);
        }

        if let Err(error) = Self::run_ip(["link", "set", "dev", tap_name.as_str(), "up"]) {
            let _ = Self::delete_linux_interface_if_present(tap_name.as_str());
            return Err(error);
        }

        Ok(())
    }

    fn run_ip<const N: usize>(args: [&str; N]) -> Result<ExitStatus, NetworkError> {
        let status = Command::new(COMMAND_IP)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|error| NetworkError::PreparationFailed(error.to_string()))?;
        if status.success() {
            return Ok(status);
        }
        Err(NetworkError::PreparationFailed(format!(
            "ip command failed with status {status}"
        )))
    }

    fn delete_linux_interface_if_present(interface_name: &str) -> Result<(), NetworkError> {
        let path = Path::new(LINUX_SYS_CLASS_NET).join(interface_name);
        if !path.exists() {
            return Ok(());
        }
        let status = Command::new(COMMAND_IP)
            .args(["link", "delete", "dev", interface_name])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|error| NetworkError::CleanupFailed(error.to_string()))?;
        if status.success() {
            return Ok(());
        }
        Err(NetworkError::CleanupFailed(format!(
            "Failed to delete Linux TAP interface: {interface_name}"
        )))
    }

    fn process_is_alive(process_id: u32) -> bool {
        match std::env::consts::OS {
            "linux" => Path::new(LINUX_PROC_ROOT).join(process_id.to_string()).exists(),
            "windows" => Command::new(COMMAND_POWERSHELL)
                .args([
                    "-NoProfile",
                    "-NonInteractive",
                    "-Command",
                    "if (Get-Process -Id $env:TURKUAZVM_PID -ErrorAction SilentlyContinue) { exit 0 } else { exit 1 }",
                ])
                .env("TURKUAZVM_PID", process_id.to_string())
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .is_ok_and(|status| status.success()),
            _ => false,
        }
    }

    fn helper_available(&self) -> bool {
        self.settings
            .linux_bridge_helper
            .as_ref()
            .is_some_and(|path| path.is_file())
    }
}

impl NetworkPort for NativeNetworkTool {
    fn capabilities(&self) -> NetworkCapabilities {
        match std::env::consts::OS {
            "windows" => NetworkCapabilities {
                managed_nat: true,
                private_network: self.settings.windows_managed_helper.as_ref().is_some_and(|path| path.is_file()),
                user_nat: true,
                bridge: true,
                existing_tap: true,
                managed_tap: true,
                bridge_helper: false,
            },
            "linux" => NetworkCapabilities {
                managed_nat: true,
                private_network: false,
                user_nat: true,
                bridge: true,
                existing_tap: true,
                managed_tap: self.settings.linux_allow_managed_tap,
                bridge_helper: self.helper_available(),
            },
            _ => NetworkCapabilities {
                managed_nat: true,
                private_network: false,
                user_nat: true,
                bridge: false,
                existing_tap: false,
                managed_tap: false,
                bridge_helper: false,
            },
        }
    }

    fn validate_attachment(&self, attachment: &NetworkAttachment) -> Result<(), NetworkError> {
        match attachment.mode() {
            NetworkMode::ManagedNat => {
                if attachment.managed_address().is_none() || attachment.mac_address().is_none() {
                    return Err(NetworkError::InvalidConfiguration(String::from("Turkuaz NAT requires MAC and IPv4 assignment")));
                }
                Ok(())
            }
            NetworkMode::Private => {
                if std::env::consts::OS != "windows" {
                    return Err(NetworkError::UnsupportedMode(attachment.mode()));
                }
                self.windows_managed_helper()?;
                if attachment.managed_address().is_none() || attachment.mac_address().is_none() {
                    return Err(NetworkError::InvalidConfiguration(String::from("Private network requires MAC and IPv4 assignment")));
                }
                Ok(())
            }
            NetworkMode::UserNat => Ok(()),
            NetworkMode::Bridge => self.validate_bridge_attachment(attachment),
        }
    }

    fn prepare_runtime(
        &mut self,
        machine: &VirtualMachine,
    ) -> Result<NetworkRuntimePlan, NetworkError> {
        if let Some(existing) = self.load_lease_file(machine.id())? {
            if existing.process_id.is_some_and(Self::process_is_alive) {
                return Err(NetworkError::PreparationFailed(format!(
                    "Active network runtime lease already exists for VM {}",
                    machine.id().as_str()
                )));
            }
            self.cleanup_previous_lease_for_machine(machine, &existing)?;
        }

        let mut bindings = Vec::with_capacity(machine.networks().len());
        let mut leases = Vec::with_capacity(machine.networks().len());

        for (index, attachment) in machine.networks().iter().enumerate() {
            if let Err(error) = self.validate_attachment(attachment) {
                let _ = self.cleanup_records(&leases);
                return Err(error);
            }
            let prepared = match attachment.mode() {
                NetworkMode::ManagedNat | NetworkMode::UserNat => Ok((
                    PreparedNetworkBinding::new(
                        attachment.id().clone(),
                        PreparedNetworkBackend::UserNat,
                    ),
                    LeaseRecord {
                        network_id: attachment.id().as_str().to_owned(),
                        kind: LeaseKind::UserNat,
                        interface_name: None,
                        bridge_name: None,
                        managed_guest_ip: attachment.managed_address().map(|address| address.ipv4_address().to_string()),
                        owned_by_turkuazvm: false,
                    },
                )),
                NetworkMode::Private => self.prepare_windows_managed(machine.id(), index, attachment),
                NetworkMode::Bridge => self.prepare_bridge(machine.id(), index, attachment),
            };
            let prepared = match prepared {
                Ok(prepared) => prepared,
                Err(error) => {
                    let _ = self.cleanup_records(&leases);
                    return Err(error);
                }
            };
            bindings.push(prepared.0);
            leases.push(prepared.1);
        }

        let plan = match NetworkRuntimePlan::new(bindings) {
            Ok(plan) => plan,
            Err(error) => {
                let _ = self.cleanup_records(&leases);
                return Err(NetworkError::PreparationFailed(format!("{error:?}")));
            }
        };

        let lease_file = NetworkLeaseFile {
            schema_version: LEASE_SCHEMA_VERSION,
            vm_id: machine.id().as_str().to_owned(),
            process_id: None,
            leases,
        };
        if let Err(error) = self.save_lease_file(&lease_file) {
            let _ = self.cleanup_records(&lease_file.leases);
            return Err(error);
        }

        Ok(plan)
    }

    fn bind_runtime_process(
        &mut self,
        vm_id: &VmId,
        process_id: u32,
    ) -> Result<(), NetworkError> {
        let mut lease = self.load_lease_file(vm_id)?.ok_or_else(|| {
            NetworkError::RuntimeBindingFailed(format!(
                "Network runtime lease not found for VM {}",
                vm_id.as_str()
            ))
        })?;
        lease.process_id = Some(process_id);
        self.save_lease_file(&lease)
    }

    fn cleanup_runtime(&mut self, vm_id: &VmId) -> Result<(), NetworkError> {
        let Some(lease) = self.load_lease_file(vm_id)? else {
            return Ok(());
        };
        self.cleanup_lease_file(&lease)
    }

    fn recover_runtime(&mut self) -> Result<NetworkRecoveryReport, NetworkError> {
        let directory = self.lease_directory();
        if !directory.is_dir() {
            return Ok(NetworkRecoveryReport::default());
        }

        let mut report = NetworkRecoveryReport::default();
        for entry in fs::read_dir(&directory)
            .map_err(|error| NetworkError::RecoveryFailed(error.to_string()))?
        {
            let entry = entry.map_err(|error| NetworkError::RecoveryFailed(error.to_string()))?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some(LEASE_EXTENSION) {
                continue;
            }

            let content = match fs::read_to_string(&path) {
                Ok(content) => content,
                Err(_) => {
                    report.failed_leases += 1;
                    continue;
                }
            };
            let lease: NetworkLeaseFile = match serde_yaml_ng::from_str(&content) {
                Ok(lease) => lease,
                Err(_) => {
                    report.failed_leases += 1;
                    continue;
                }
            };

            if lease.process_id.is_some_and(Self::process_is_alive) {
                report.active_leases += 1;
                continue;
            }

            match self.cleanup_lease_file(&lease) {
                Ok(()) => report.cleaned_leases += 1,
                Err(_) => report.failed_leases += 1,
            }
        }
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    use turkuazvm_core::domain::hypervisor::AccelerationBackend;
    use turkuazvm_core::domain::network::{
        MacAddress, ManagedAddressConfiguration, NetworkDeviceModel, NetworkId,
    };
    use turkuazvm_core::domain::virtual_machine::VmResourceConfig;

    fn test_root() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be valid")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "turkuazvm-network-tool-{}-{nonce}",
            std::process::id()
        ))
    }

    #[test]
    fn user_nat_prepare_creates_recoverable_lease_without_host_resource() {
        let root = test_root();
        let mut machine = VirtualMachine::create(
            VmId::parse("nat-test").expect("vm id must be valid"),
            "NAT Test",
            VmResourceConfig {
                vcpu_count: 2,
                memory_mib: 2048,
            },
            AccelerationBackend::Tcg,
        )
        .expect("vm must be valid");
        machine
            .attach_network(
                NetworkAttachment::create(
                    NetworkId::parse("default").expect("network id must be valid"),
                    NetworkMode::UserNat,
                    NetworkDeviceModel::VirtioNetPci,
                    None,
                    Vec::new(),
                    None,
                )
                .expect("network must be valid"),
            )
            .expect("network attach must succeed");

        let mut tool = NativeNetworkTool::new(NativeNetworkSettings {
            state_root: root.clone(),
            linux_bridge_helper: None,
            linux_allow_managed_tap: false,
            windows_managed_helper: None,
        });
        let plan = tool
            .prepare_runtime(&machine)
            .expect("NAT prepare must succeed");
        assert_eq!(plan.bindings().len(), 1);
        assert!(matches!(
            plan.bindings()[0].backend(),
            PreparedNetworkBackend::UserNat
        ));
        tool.cleanup_runtime(machine.id())
            .expect("cleanup must succeed");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn managed_nat_prepare_uses_qemu_user_backend_without_windows_helper() {
        let root = test_root();
        let mut machine = VirtualMachine::create(
            VmId::parse("portable-managed-nat-test").expect("vm id must be valid"),
            "Portable Managed NAT Test",
            VmResourceConfig {
                vcpu_count: 2,
                memory_mib: 2048,
            },
            AccelerationBackend::Tcg,
        )
        .expect("vm must be valid");
        let address = ManagedAddressConfiguration::create(
            "192.168.240.10".parse().expect("ip must be valid"),
            24,
            Some("192.168.240.1".parse().expect("gateway must be valid")),
            Vec::new(),
        )
        .expect("managed address must be valid");
        machine
            .attach_network(
                NetworkAttachment::create_with_address(
                    NetworkId::parse("turkuaz-net-01").expect("network id must be valid"),
                    NetworkMode::ManagedNat,
                    NetworkDeviceModel::VirtioNetPci,
                    Some(MacAddress::parse("02:54:00:12:34:56").expect("mac must be valid")),
                    Vec::new(),
                    None,
                    Some(address),
                    Some(NetworkId::parse("turkuaz-net-01").expect("fabric id must be valid")),
                )
                .expect("managed network must be valid"),
            )
            .expect("network attach must succeed");

        let mut tool = NativeNetworkTool::new(NativeNetworkSettings {
            state_root: root.clone(),
            linux_bridge_helper: None,
            linux_allow_managed_tap: false,
            windows_managed_helper: None,
        });
        let plan = tool
            .prepare_runtime(&machine)
            .expect("portable managed NAT prepare must succeed without host helper");
        assert_eq!(plan.bindings().len(), 1);
        assert!(matches!(
            plan.bindings()[0].backend(),
            PreparedNetworkBackend::UserNat
        ));
        tool.cleanup_runtime(machine.id())
            .expect("portable NAT cleanup must succeed");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn managed_tap_name_fits_linux_interface_limit() {
        let vm_id = VmId::parse("very-long-virtual-machine-name")
            .expect("vm id must be valid");
        let name = NativeNetworkTool::managed_tap_name(&vm_id, 1)
            .expect("tap name must be valid");
        assert!(name.as_str().len() <= MAX_LINUX_INTERFACE_NAME_LENGTH);
    }
}
