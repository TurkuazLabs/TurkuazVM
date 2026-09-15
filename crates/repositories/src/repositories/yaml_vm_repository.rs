// # 📄 Dosya Yolu: /turkuazvm/crates/repositories/src/repositories/yaml_vm_repository.rs
// # 📌 Amac: VirtualMachine aggregate bilgisini machine.yml manifestlerine kalici olarak yazar ve okur
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Core domain managed IPv4 dahil serde/YAML detaylarindan ayiran filesystem repository adapteridir
// # Bagimli Oldugu Katman: Repo

use std::fs;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use turkuazvm_core::domain::disk::{DiskAttachment, DiskBus, DiskFormat, DiskId, DiskImage};
use turkuazvm_core::domain::guest_boot::{
    BootDevice, BootOrder, FirmwareSelection, GuestBootConfiguration, GuestProfile, IsoAttachment,
    MediaId, UefiFirmware,
};
use turkuazvm_core::domain::hypervisor::AccelerationBackend;
use turkuazvm_core::domain::network::{
    BridgeConfiguration, HostNetworkName, MacAddress, ManagedAddressConfiguration, NetworkAttachment, NetworkDeviceModel,
    NetworkId, NetworkMode, PortForwardRule, PortProtocol,
};
use turkuazvm_core::domain::snapshot::{SnapshotId, SnapshotRecord};
use turkuazvm_core::domain::virtual_machine::{
    VirtualMachine, VmFailure, VmFailureKind, VmId, VmResourceConfig,
};
use turkuazvm_core::domain::vm_state::VmState;
use turkuazvm_core::ports::vm_repository_port::{VmRepositoryError, VmRepositoryPort};

const MANIFEST_SCHEMA_VERSION: u16 = 6;
const MIN_SUPPORTED_MANIFEST_SCHEMA_VERSION: u16 = 1;
const DIR_MACHINES: &str = "machines";
const FILE_MANIFEST: &str = "machine.yml";

#[derive(Debug, Clone)]
pub struct YamlVmRepositorySettings {
    pub data_root: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct VmManifest {
    schema_version: u16,
    vm: VmManifestMachine,
    storage: VmManifestStorage,
    #[serde(default)]
    network: VmManifestNetwork,
    #[serde(default)]
    guest: Option<VmManifestGuest>,
    #[serde(default)]
    snapshots: VmManifestSnapshots,
}

#[derive(Debug, Serialize, Deserialize)]
struct VmManifestMachine {
    id: String,
    name: String,
    vcpu_count: u16,
    memory_mib: u64,
    acceleration: ManifestAcceleration,
    state: ManifestVmState,
    last_failure: Option<ManifestFailure>,
}

#[derive(Debug, Serialize, Deserialize)]
struct VmManifestStorage {
    disks: Vec<ManifestDisk>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ManifestDisk {
    id: String,
    format: ManifestDiskFormat,
    virtual_size_bytes: u64,
    relative_path: String,
    bus: ManifestDiskBus,
    boot_index: Option<u8>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct VmManifestNetwork {
    adapters: Vec<ManifestNetworkAdapter>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ManifestNetworkAdapter {
    id: String,
    mode: ManifestNetworkMode,
    device_model: ManifestNetworkDeviceModel,
    mac_address: Option<String>,
    port_forwards: Vec<ManifestPortForward>,
    #[serde(default)]
    bridge: Option<ManifestBridgeConfiguration>,
    #[serde(default)]
    managed_address: Option<ManifestManagedAddress>,
    #[serde(default)]
    fabric_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ManifestManagedAddress {
    ipv4_address: String,
    prefix_length: u8,
    gateway: Option<String>,
    #[serde(default)]
    dns_servers: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ManifestBridgeConfiguration {
    bridge_name: Option<String>,
    tap_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ManifestPortForward {
    protocol: ManifestPortProtocol,
    host_ip: Option<String>,
    host_port: u16,
    guest_ip: Option<String>,
    guest_port: u16,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct VmManifestSnapshots {
    records: Vec<ManifestSnapshot>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ManifestSnapshot {
    id: String,
    name: String,
    created_at_unix_ms: u64,
    disk_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct VmManifestGuest {
    profile: ManifestGuestProfile,
    firmware: ManifestFirmware,
    boot_order: Vec<ManifestBootDevice>,
    boot_once: bool,
    installer_iso: Option<ManifestIsoAttachment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    catalog_template_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ManifestIsoAttachment {
    id: String,
    relative_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ManifestGuestProfile {
    Generic,
    Linux,
    Windows,
    Android,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ManifestBootDevice {
    Disk,
    Cdrom,
    Network,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
enum ManifestFirmware {
    Bios,
    Uefi {
        code_relative_path: String,
        vars_relative_path: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ManifestNetworkMode {
    ManagedNat,
    Private,
    UserNat,
    Bridge,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ManifestNetworkDeviceModel {
    VirtioNetPci,
    E1000,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ManifestPortProtocol {
    Tcp,
    Udp,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ManifestAcceleration {
    Whpx,
    Kvm,
    Tcg,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ManifestVmState {
    Created,
    Stopped,
    Starting,
    Running,
    Pausing,
    Paused,
    Resuming,
    Stopping,
    Error,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ManifestDiskFormat {
    Qcow2,
    Raw,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ManifestDiskBus {
    Virtio,
    Ide,
}

#[derive(Debug, Serialize, Deserialize)]
struct ManifestFailure {
    kind: ManifestFailureKind,
    detail: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ManifestFailureKind {
    NetworkPrepare,
    NetworkBind,
    NetworkCleanup,
    HypervisorStart,
    HypervisorStop,
    RuntimeExit,
}

pub struct YamlVmRepository {
    settings: YamlVmRepositorySettings,
}

impl YamlVmRepository {
    pub fn new(settings: YamlVmRepositorySettings) -> Self {
        Self { settings }
    }

    fn machine_directory(&self, vm_id: &VmId) -> PathBuf {
        self.settings
            .data_root
            .join(DIR_MACHINES)
            .join(vm_id.as_str())
    }

    fn manifest_path(&self, vm_id: &VmId) -> PathBuf {
        self.machine_directory(vm_id).join(FILE_MANIFEST)
    }

    fn quarantine_manifest(path: &Path) -> Result<(), VmRepositoryError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| VmRepositoryError::StorageFailed(error.to_string()))?
            .as_millis();
        let quarantine = PathBuf::from(format!("{}.bad.{timestamp}", path.display()));
        fs::rename(path, quarantine)
            .map_err(|error| VmRepositoryError::StorageFailed(error.to_string()))
    }

    fn classify_manifest_schema(path: &Path) -> Result<Option<u16>, VmRepositoryError> {
        let content = fs::read_to_string(path)
            .map_err(|error| VmRepositoryError::StorageFailed(error.to_string()))?;
        let value: serde_yaml_ng::Value = match serde_yaml_ng::from_str(&content) {
            Ok(value) => value,
            Err(_) => return Ok(None),
        };
        Ok(value
            .get("schema_version")
            .and_then(serde_yaml_ng::Value::as_u64)
            .and_then(|value| u16::try_from(value).ok()))
    }

    fn write_machine(&self, machine: &VirtualMachine) -> Result<(), VmRepositoryError> {
        let directory = self.machine_directory(machine.id());
        fs::create_dir_all(&directory)
            .map_err(|error| VmRepositoryError::StorageFailed(error.to_string()))?;

        let manifest = Self::to_manifest(machine);
        let yaml = serde_yaml_ng::to_string(&manifest)
            .map_err(|error| VmRepositoryError::StorageFailed(error.to_string()))?;
        let path = self.manifest_path(machine.id());
        let header = Self::manifest_header(&path);
        fs::write(&path, format!("{header}{yaml}"))
            .map_err(|error| VmRepositoryError::StorageFailed(error.to_string()))
    }

    fn read_machine(&self, vm_id: &VmId) -> Result<VirtualMachine, VmRepositoryError> {
        let path = self.manifest_path(vm_id);
        if !path.is_file() {
            return Err(VmRepositoryError::NotFound(vm_id.clone()));
        }

        let content = fs::read_to_string(&path)
            .map_err(|error| VmRepositoryError::StorageFailed(error.to_string()))?;
        let manifest: VmManifest = serde_yaml_ng::from_str(&content)
            .map_err(|error| VmRepositoryError::StorageFailed(error.to_string()))?;

        if manifest.schema_version < MIN_SUPPORTED_MANIFEST_SCHEMA_VERSION
            || manifest.schema_version > MANIFEST_SCHEMA_VERSION
        {
            return Err(VmRepositoryError::StorageFailed(format!(
                "Unsupported manifest schema version: {}",
                manifest.schema_version
            )));
        }

        Self::from_manifest(manifest)
    }

    fn manifest_header(path: &Path) -> String {
        format!(
            "# \u{1f4c4} Dosya Yolu: {}\n# \u{1f4cc} Amac: Bu VM icin kalici machine manifestini saklar\n# \u{1f4cc} Modul - YAML\n# Version: {}\n# Aciklama: VM kaynak, state, guest boot, disk, network ve snapshot metadata bilgisini tasinabilir YAML olarak saklar\n# Bagimli Oldugu Katman: Repo\n\n",
            path.display(),
            env!("CARGO_PKG_VERSION")
        )
    }

    fn to_manifest(machine: &VirtualMachine) -> VmManifest {
        VmManifest {
            schema_version: MANIFEST_SCHEMA_VERSION,
            vm: VmManifestMachine {
                id: machine.id().as_str().to_owned(),
                name: machine.name().to_owned(),
                vcpu_count: machine.resources().vcpu_count,
                memory_mib: machine.resources().memory_mib,
                acceleration: machine.acceleration().into(),
                state: machine.state().into(),
                last_failure: machine.last_failure().cloned().map(Into::into),
            },
            storage: VmManifestStorage {
                disks: machine
                    .disks()
                    .iter()
                    .map(|attachment| ManifestDisk {
                        id: attachment.image().id().as_str().to_owned(),
                        format: attachment.image().format().into(),
                        virtual_size_bytes: attachment.image().virtual_size_bytes(),
                        relative_path: attachment.image().relative_path().to_owned(),
                        bus: attachment.bus().into(),
                        boot_index: attachment.boot_index(),
                    })
                    .collect(),
            },
            network: VmManifestNetwork {
                adapters: machine
                    .networks()
                    .iter()
                    .map(|attachment| ManifestNetworkAdapter {
                        id: attachment.id().as_str().to_owned(),
                        mode: attachment.mode().into(),
                        device_model: attachment.device_model().into(),
                        mac_address: attachment
                            .mac_address()
                            .map(MacAddress::to_canonical_string),
                        port_forwards: attachment
                            .port_forwards()
                            .iter()
                            .map(|rule| ManifestPortForward {
                                protocol: rule.protocol().into(),
                                host_ip: rule.host_ip().map(|ip| ip.to_string()),
                                host_port: rule.host_port(),
                                guest_ip: rule.guest_ip().map(|ip| ip.to_string()),
                                guest_port: rule.guest_port(),
                            })
                            .collect(),
                        bridge: attachment.bridge_configuration().map(|bridge| {
                            ManifestBridgeConfiguration {
                                bridge_name: bridge
                                    .bridge_name()
                                    .map(|name| name.as_str().to_owned()),
                                tap_name: bridge
                                    .tap_name()
                                    .map(|name| name.as_str().to_owned()),
                            }
                        }),
                        managed_address: attachment.managed_address().map(|address| ManifestManagedAddress {
                            ipv4_address: address.ipv4_address().to_string(),
                            prefix_length: address.prefix_length(),
                            gateway: address.gateway().map(|value| value.to_string()),
                            dns_servers: address.dns_servers().iter().map(ToString::to_string).collect(),
                        }),
                        fabric_id: attachment.fabric_id().map(|value| value.as_str().to_owned()),
                    })
                    .collect(),
            },
            guest: Some(Self::guest_to_manifest(machine.guest_boot())),
            snapshots: VmManifestSnapshots {
                records: machine
                    .snapshots()
                    .iter()
                    .map(|snapshot| ManifestSnapshot {
                        id: snapshot.id().as_str().to_owned(),
                        name: snapshot.name().to_owned(),
                        created_at_unix_ms: snapshot.created_at_unix_ms(),
                        disk_ids: snapshot
                            .disk_ids()
                            .iter()
                            .map(|disk_id| disk_id.as_str().to_owned())
                            .collect(),
                    })
                    .collect(),
            },
        }
    }

    fn guest_to_manifest(guest: &GuestBootConfiguration) -> VmManifestGuest {
        let firmware = match guest.firmware() {
            FirmwareSelection::Bios => ManifestFirmware::Bios,
            FirmwareSelection::Uefi(uefi) => ManifestFirmware::Uefi {
                code_relative_path: uefi.code_relative_path().to_owned(),
                vars_relative_path: uefi.vars_relative_path().to_owned(),
            },
        };
        VmManifestGuest {
            profile: guest.profile().into(),
            firmware,
            boot_order: guest
                .boot_order()
                .devices()
                .iter()
                .copied()
                .map(Into::into)
                .collect(),
            boot_once: guest.boot_order().apply_once(),
            installer_iso: guest.installer_iso().map(|iso| ManifestIsoAttachment {
                id: iso.id().as_str().to_owned(),
                relative_path: iso.relative_path().to_owned(),
            }),
            catalog_template_id: guest.catalog_template_id().map(str::to_owned),
        }
    }

    fn guest_from_manifest(
        guest: Option<VmManifestGuest>,
    ) -> Result<GuestBootConfiguration, VmRepositoryError> {
        let Some(guest) = guest else {
            return Ok(GuestBootConfiguration::default());
        };
        let firmware = match guest.firmware {
            ManifestFirmware::Bios => FirmwareSelection::Bios,
            ManifestFirmware::Uefi {
                code_relative_path,
                vars_relative_path,
            } => FirmwareSelection::Uefi(
                UefiFirmware::create(code_relative_path, vars_relative_path)
                    .map_err(|error| VmRepositoryError::StorageFailed(format!("{error:?}")))?,
            ),
        };
        let boot_order = BootOrder::create(
            guest.boot_order.into_iter().map(Into::into).collect(),
            guest.boot_once,
        )
        .map_err(|error| VmRepositoryError::StorageFailed(format!("{error:?}")))?;
        let installer_iso = guest
            .installer_iso
            .map(|iso| {
                let id = MediaId::parse(iso.id)
                    .map_err(|error| VmRepositoryError::StorageFailed(format!("{error:?}")))?;
                IsoAttachment::create(id, iso.relative_path)
                    .map_err(|error| VmRepositoryError::StorageFailed(format!("{error:?}")))
            })
            .transpose()?;
        GuestBootConfiguration::create(
            guest.profile.into(),
            firmware,
            boot_order,
            installer_iso,
            guest.catalog_template_id,
        )
            .map_err(|error| VmRepositoryError::StorageFailed(format!("{error:?}")))
    }

    fn from_manifest(manifest: VmManifest) -> Result<VirtualMachine, VmRepositoryError> {
        let id = VmId::parse(manifest.vm.id)
            .map_err(|error| VmRepositoryError::StorageFailed(format!("{error:?}")))?;

        let mut disks = Vec::with_capacity(manifest.storage.disks.len());
        for disk in manifest.storage.disks {
            let disk_id = DiskId::parse(disk.id)
                .map_err(|error| VmRepositoryError::StorageFailed(format!("{error:?}")))?;
            let image = DiskImage::create(
                disk_id,
                disk.format.into(),
                disk.virtual_size_bytes,
                disk.relative_path,
            )
            .map_err(|error| VmRepositoryError::StorageFailed(format!("{error:?}")))?;
            disks.push(DiskAttachment::new(image, disk.bus.into(), disk.boot_index));
        }

        let mut networks = Vec::with_capacity(manifest.network.adapters.len());
        for adapter in manifest.network.adapters {
            let network_id = NetworkId::parse(adapter.id)
                .map_err(|error| VmRepositoryError::StorageFailed(format!("{error:?}")))?;
            let mac_address = adapter
                .mac_address
                .as_deref()
                .map(MacAddress::parse)
                .transpose()
                .map_err(|error| VmRepositoryError::StorageFailed(format!("{error:?}")))?;
            let mut forwards = Vec::with_capacity(adapter.port_forwards.len());
            for forward in adapter.port_forwards {
                let host_ip = forward
                    .host_ip
                    .map(|value| value.parse::<Ipv4Addr>())
                    .transpose()
                    .map_err(|error| VmRepositoryError::StorageFailed(error.to_string()))?;
                let guest_ip = forward
                    .guest_ip
                    .map(|value| value.parse::<Ipv4Addr>())
                    .transpose()
                    .map_err(|error| VmRepositoryError::StorageFailed(error.to_string()))?;
                forwards.push(
                    PortForwardRule::create(
                        forward.protocol.into(),
                        host_ip,
                        forward.host_port,
                        guest_ip,
                        forward.guest_port,
                    )
                    .map_err(|error| VmRepositoryError::StorageFailed(format!("{error:?}")))?,
                );
            }
            let bridge = adapter
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
                .map_err(|error| VmRepositoryError::StorageFailed(format!("{error:?}")))?;
            let managed_address = adapter.managed_address
                .map(|address| {
                    let ipv4_address = address.ipv4_address.parse::<Ipv4Addr>()
                        .map_err(|error| VmRepositoryError::StorageFailed(error.to_string()))?;
                    let gateway = address.gateway
                        .map(|value| value.parse::<Ipv4Addr>())
                        .transpose()
                        .map_err(|error| VmRepositoryError::StorageFailed(error.to_string()))?;
                    let dns_servers = address.dns_servers
                        .into_iter()
                        .map(|value| value.parse::<Ipv4Addr>())
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|error| VmRepositoryError::StorageFailed(error.to_string()))?;
                    ManagedAddressConfiguration::create(ipv4_address, address.prefix_length, gateway, dns_servers)
                        .map_err(|error| VmRepositoryError::StorageFailed(format!("{error:?}")))
                })
                .transpose()?;
            let fabric_id = adapter.fabric_id
                .map(NetworkId::parse)
                .transpose()
                .map_err(|error| VmRepositoryError::StorageFailed(format!("{error:?}")))?;
            networks.push(
                NetworkAttachment::create_with_address(
                    network_id,
                    adapter.mode.into(),
                    adapter.device_model.into(),
                    mac_address,
                    forwards,
                    bridge,
                    managed_address,
                    fabric_id,
                )
                .map_err(|error| VmRepositoryError::StorageFailed(format!("{error:?}")))?,
            );
        }

        let mut snapshots = Vec::with_capacity(manifest.snapshots.records.len());
        for snapshot in manifest.snapshots.records {
            let snapshot_id = SnapshotId::parse(snapshot.id)
                .map_err(|error| VmRepositoryError::StorageFailed(format!("{error:?}")))?;
            let mut disk_ids = Vec::with_capacity(snapshot.disk_ids.len());
            for disk_id in snapshot.disk_ids {
                disk_ids.push(
                    DiskId::parse(disk_id)
                        .map_err(|error| VmRepositoryError::StorageFailed(format!("{error:?}")))?,
                );
            }
            snapshots.push(
                SnapshotRecord::create(
                    snapshot_id,
                    snapshot.name,
                    snapshot.created_at_unix_ms,
                    disk_ids,
                )
                .map_err(|error| VmRepositoryError::StorageFailed(format!("{error:?}")))?,
            );
        }

        let guest_boot = Self::guest_from_manifest(manifest.guest)?;

        VirtualMachine::reconstitute(
            id,
            manifest.vm.name,
            VmResourceConfig {
                vcpu_count: manifest.vm.vcpu_count,
                memory_mib: manifest.vm.memory_mib,
            },
            manifest.vm.acceleration.into(),
            manifest.vm.state.into(),
            manifest.vm.last_failure.map(Into::into),
            guest_boot,
            disks,
            networks,
            snapshots,
        )
        .map_err(|error| VmRepositoryError::StorageFailed(format!("{error:?}")))
    }
}

impl VmRepositoryPort for YamlVmRepository {
    fn insert(&mut self, machine: VirtualMachine) -> Result<(), VmRepositoryError> {
        let path = self.manifest_path(machine.id());
        if path.exists() {
            return Err(VmRepositoryError::AlreadyExists(machine.id().clone()));
        }
        self.write_machine(&machine)
    }

    fn get(&self, vm_id: &VmId) -> Result<VirtualMachine, VmRepositoryError> {
        self.read_machine(vm_id)
    }

    fn list(&self) -> Result<Vec<VirtualMachine>, VmRepositoryError> {
        let machines_root = self.settings.data_root.join(DIR_MACHINES);
        if !machines_root.exists() {
            return Ok(Vec::new());
        }

        let mut machines = Vec::new();
        let entries = fs::read_dir(&machines_root)
            .map_err(|error| VmRepositoryError::StorageFailed(error.to_string()))?;

        for entry in entries {
            let entry = entry
                .map_err(|error| VmRepositoryError::StorageFailed(error.to_string()))?;
            if !entry.path().is_dir() {
                continue;
            }

            let manifest_path = entry.path().join(FILE_MANIFEST);
            if !manifest_path.is_file() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            let vm_id = match VmId::parse(name) {
                Ok(vm_id) => vm_id,
                Err(_) => {
                    Self::quarantine_manifest(&manifest_path)?;
                    continue;
                }
            };
            match self.read_machine(&vm_id) {
                Ok(machine) => machines.push(machine),
                Err(error) => {
                    if matches!(Self::classify_manifest_schema(&manifest_path)?, Some(version) if version > MANIFEST_SCHEMA_VERSION) {
                        return Err(error);
                    }
                    Self::quarantine_manifest(&manifest_path)?;
                }
            }
        }

        machines.sort_by(|left, right| left.name().cmp(right.name()));
        Ok(machines)
    }

    fn save(&mut self, machine: VirtualMachine) -> Result<(), VmRepositoryError> {
        let path = self.manifest_path(machine.id());
        if !path.exists() {
            return Err(VmRepositoryError::NotFound(machine.id().clone()));
        }
        self.write_machine(&machine)
    }

    fn delete(&mut self, vm_id: &VmId) -> Result<(), VmRepositoryError> {
        let directory = self.machine_directory(vm_id);
        if !directory.exists() {
            return Err(VmRepositoryError::NotFound(vm_id.clone()));
        }
        fs::remove_dir_all(directory)
            .map_err(|error| VmRepositoryError::StorageFailed(error.to_string()))
    }
}

impl From<AccelerationBackend> for ManifestAcceleration {
    fn from(value: AccelerationBackend) -> Self {
        match value {
            AccelerationBackend::Whpx => Self::Whpx,
            AccelerationBackend::Kvm => Self::Kvm,
            AccelerationBackend::Tcg => Self::Tcg,
        }
    }
}

impl From<ManifestAcceleration> for AccelerationBackend {
    fn from(value: ManifestAcceleration) -> Self {
        match value {
            ManifestAcceleration::Whpx => Self::Whpx,
            ManifestAcceleration::Kvm => Self::Kvm,
            ManifestAcceleration::Tcg => Self::Tcg,
        }
    }
}

impl From<VmState> for ManifestVmState {
    fn from(value: VmState) -> Self {
        match value {
            VmState::Created => Self::Created,
            VmState::Stopped => Self::Stopped,
            VmState::Starting => Self::Starting,
            VmState::Running => Self::Running,
            VmState::Pausing => Self::Pausing,
            VmState::Paused => Self::Paused,
            VmState::Resuming => Self::Resuming,
            VmState::Stopping => Self::Stopping,
            VmState::Error => Self::Error,
        }
    }
}

impl From<ManifestVmState> for VmState {
    fn from(value: ManifestVmState) -> Self {
        match value {
            ManifestVmState::Created => Self::Created,
            ManifestVmState::Stopped => Self::Stopped,
            ManifestVmState::Starting => Self::Starting,
            ManifestVmState::Running => Self::Running,
            ManifestVmState::Pausing => Self::Pausing,
            ManifestVmState::Paused => Self::Paused,
            ManifestVmState::Resuming => Self::Resuming,
            ManifestVmState::Stopping => Self::Stopping,
            ManifestVmState::Error => Self::Error,
        }
    }
}

impl From<DiskFormat> for ManifestDiskFormat {
    fn from(value: DiskFormat) -> Self {
        match value {
            DiskFormat::Qcow2 => Self::Qcow2,
            DiskFormat::Raw => Self::Raw,
        }
    }
}

impl From<ManifestDiskFormat> for DiskFormat {
    fn from(value: ManifestDiskFormat) -> Self {
        match value {
            ManifestDiskFormat::Qcow2 => Self::Qcow2,
            ManifestDiskFormat::Raw => Self::Raw,
        }
    }
}

impl From<DiskBus> for ManifestDiskBus {
    fn from(value: DiskBus) -> Self {
        match value {
            DiskBus::Virtio => Self::Virtio,
            DiskBus::Ide => Self::Ide,
        }
    }
}

impl From<ManifestDiskBus> for DiskBus {
    fn from(value: ManifestDiskBus) -> Self {
        match value {
            ManifestDiskBus::Virtio => Self::Virtio,
            ManifestDiskBus::Ide => Self::Ide,
        }
    }
}

impl From<GuestProfile> for ManifestGuestProfile {
    fn from(value: GuestProfile) -> Self {
        match value {
            GuestProfile::Generic => Self::Generic,
            GuestProfile::Linux => Self::Linux,
            GuestProfile::Windows => Self::Windows,
            GuestProfile::Android => Self::Android,
        }
    }
}

impl From<ManifestGuestProfile> for GuestProfile {
    fn from(value: ManifestGuestProfile) -> Self {
        match value {
            ManifestGuestProfile::Generic => Self::Generic,
            ManifestGuestProfile::Linux => Self::Linux,
            ManifestGuestProfile::Windows => Self::Windows,
            ManifestGuestProfile::Android => Self::Android,
        }
    }
}

impl From<BootDevice> for ManifestBootDevice {
    fn from(value: BootDevice) -> Self {
        match value {
            BootDevice::Disk => Self::Disk,
            BootDevice::Cdrom => Self::Cdrom,
            BootDevice::Network => Self::Network,
        }
    }
}

impl From<ManifestBootDevice> for BootDevice {
    fn from(value: ManifestBootDevice) -> Self {
        match value {
            ManifestBootDevice::Disk => Self::Disk,
            ManifestBootDevice::Cdrom => Self::Cdrom,
            ManifestBootDevice::Network => Self::Network,
        }
    }
}

impl From<NetworkMode> for ManifestNetworkMode {
    fn from(value: NetworkMode) -> Self {
        match value {
            NetworkMode::ManagedNat => Self::ManagedNat,
            NetworkMode::Private => Self::Private,
            NetworkMode::UserNat => Self::UserNat,
            NetworkMode::Bridge => Self::Bridge,
        }
    }
}

impl From<ManifestNetworkMode> for NetworkMode {
    fn from(value: ManifestNetworkMode) -> Self {
        match value {
            ManifestNetworkMode::ManagedNat => Self::ManagedNat,
            ManifestNetworkMode::Private => Self::Private,
            ManifestNetworkMode::UserNat => Self::UserNat,
            ManifestNetworkMode::Bridge => Self::Bridge,
        }
    }
}

impl From<NetworkDeviceModel> for ManifestNetworkDeviceModel {
    fn from(value: NetworkDeviceModel) -> Self {
        match value {
            NetworkDeviceModel::VirtioNetPci => Self::VirtioNetPci,
            NetworkDeviceModel::E1000 => Self::E1000,
        }
    }
}

impl From<ManifestNetworkDeviceModel> for NetworkDeviceModel {
    fn from(value: ManifestNetworkDeviceModel) -> Self {
        match value {
            ManifestNetworkDeviceModel::VirtioNetPci => Self::VirtioNetPci,
            ManifestNetworkDeviceModel::E1000 => Self::E1000,
        }
    }
}

impl From<PortProtocol> for ManifestPortProtocol {
    fn from(value: PortProtocol) -> Self {
        match value {
            PortProtocol::Tcp => Self::Tcp,
            PortProtocol::Udp => Self::Udp,
        }
    }
}

impl From<ManifestPortProtocol> for PortProtocol {
    fn from(value: ManifestPortProtocol) -> Self {
        match value {
            ManifestPortProtocol::Tcp => Self::Tcp,
            ManifestPortProtocol::Udp => Self::Udp,
        }
    }
}

impl From<VmFailure> for ManifestFailure {
    fn from(value: VmFailure) -> Self {
        Self {
            kind: match value.kind {
                VmFailureKind::NetworkPrepare => ManifestFailureKind::NetworkPrepare,
                VmFailureKind::NetworkBind => ManifestFailureKind::NetworkBind,
                VmFailureKind::NetworkCleanup => ManifestFailureKind::NetworkCleanup,
                VmFailureKind::HypervisorStart => ManifestFailureKind::HypervisorStart,
                VmFailureKind::HypervisorStop => ManifestFailureKind::HypervisorStop,
                VmFailureKind::RuntimeExit => ManifestFailureKind::RuntimeExit,
            },
            detail: value.detail,
        }
    }
}

impl From<ManifestFailure> for VmFailure {
    fn from(value: ManifestFailure) -> Self {
        Self {
            kind: match value.kind {
                ManifestFailureKind::NetworkPrepare => VmFailureKind::NetworkPrepare,
                ManifestFailureKind::NetworkBind => VmFailureKind::NetworkBind,
                ManifestFailureKind::NetworkCleanup => VmFailureKind::NetworkCleanup,
                ManifestFailureKind::HypervisorStart => VmFailureKind::HypervisorStart,
                ManifestFailureKind::HypervisorStop => VmFailureKind::HypervisorStop,
                ManifestFailureKind::RuntimeExit => VmFailureKind::RuntimeExit,
            },
            detail: value.detail,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    use turkuazvm_core::domain::disk::{DiskAttachment, DiskBus, DiskFormat, DiskId, DiskImage};

    fn test_root() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be valid")
            .as_nanos();
        std::env::temp_dir().join(format!("turkuazvm-yaml-repo-{}-{nonce}", std::process::id()))
    }

    fn machine() -> VirtualMachine {
        let id = VmId::parse("yaml-test").expect("vm id must be valid");
        let disk_id = DiskId::parse("system").expect("disk id must be valid");
        let snapshot_disk_id = disk_id.clone();
        let disk = DiskImage::create(
            disk_id,
            DiskFormat::Qcow2,
            64 * 1024 * 1024,
            "disks/system.qcow2",
        )
        .expect("disk must be valid");
        let forward = PortForwardRule::create(PortProtocol::Tcp, None, 2222, None, 22)
            .expect("forward must be valid");
        let network = NetworkAttachment::create(
            NetworkId::parse("default").expect("network id must be valid"),
            NetworkMode::UserNat,
            NetworkDeviceModel::VirtioNetPci,
            None,
            vec![forward],
            None,
        )
        .expect("network must be valid");
        VirtualMachine::reconstitute(
            id,
            "YAML Test",
            VmResourceConfig {
                vcpu_count: 4,
                memory_mib: 4096,
            },
            AccelerationBackend::Tcg,
            VmState::Stopped,
            None,
            GuestBootConfiguration::create(
                GuestProfile::Linux,
                FirmwareSelection::Uefi(
                    UefiFirmware::create("firmware/OVMF_CODE.fd", "firmware/OVMF_VARS.fd")
                        .expect("uefi firmware must be valid"),
                ),
                BootOrder::create(vec![BootDevice::Cdrom, BootDevice::Disk], true)
                    .expect("boot order must be valid"),
                Some(
                    IsoAttachment::create(
                        MediaId::parse("installer").expect("media id must be valid"),
                        "media/installer.iso",
                    )
                    .expect("iso must be valid"),
                ),
                None,
            )
            .expect("guest boot must be valid"),
            vec![DiskAttachment::new(disk, DiskBus::Virtio, Some(1))],
            vec![network],
            vec![SnapshotRecord::create(
                SnapshotId::parse("before-update").expect("snapshot id must be valid"),
                "Before Update",
                1234,
                vec![snapshot_disk_id],
            )
            .expect("snapshot must be valid")],
        )
        .expect("vm must be valid")
    }

    #[test]
    fn manifest_round_trip_preserves_disk_network_and_snapshot_metadata() {
        let root = test_root();
        let settings = YamlVmRepositorySettings {
            data_root: root.clone(),
        };
        let mut repository = YamlVmRepository::new(settings);
        let machine = machine();
        let id = machine.id().clone();
        repository.insert(machine).expect("insert must succeed");

        let loaded = repository.get(&id).expect("manifest must load");
        assert_eq!(loaded.disks().len(), 1);
        assert_eq!(loaded.disks()[0].image().id().as_str(), "system");
        assert_eq!(loaded.networks().len(), 1);
        assert_eq!(loaded.networks()[0].id().as_str(), "default");
        assert_eq!(loaded.networks()[0].port_forwards()[0].host_port(), 2222);
        assert_eq!(loaded.guest_boot().profile(), GuestProfile::Linux);
        assert!(matches!(loaded.guest_boot().firmware(), FirmwareSelection::Uefi(_)));
        assert!(loaded.guest_boot().installer_iso().is_some());
        assert!(loaded.guest_boot().boot_order().apply_once());
        assert_eq!(loaded.snapshots().len(), 1);
        assert_eq!(loaded.snapshots()[0].id().as_str(), "before-update");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn schema_version_one_manifest_loads_with_empty_networks() {
        let root = test_root();
        let machine_dir = root.join("machines").join("legacy");
        fs::create_dir_all(&machine_dir).expect("machine directory must be created");
        let manifest = r#"schema_version: 1
vm:
  id: legacy
  name: Legacy VM
  vcpu_count: 2
  memory_mib: 2048
  acceleration: tcg
  state: stopped
  last_failure: null
storage:
  disks: []
"#;
        fs::write(machine_dir.join("machine.yml"), manifest).expect("manifest must be written");

        let repository = YamlVmRepository::new(YamlVmRepositorySettings {
            data_root: root.clone(),
        });
        let id = VmId::parse("legacy").expect("vm id must be valid");
        let loaded = repository.get(&id).expect("legacy manifest must load");

        assert!(loaded.networks().is_empty());
        assert!(loaded.snapshots().is_empty());
        assert_eq!(loaded.guest_boot(), &GuestBootConfiguration::default());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn schema_version_five_round_trip_preserves_bridge_target() {
        let root = test_root();
        let settings = YamlVmRepositorySettings {
            data_root: root.clone(),
        };
        let mut repository = YamlVmRepository::new(settings);
        let mut machine = machine();
        let bridge_name = HostNetworkName::parse("br0").expect("bridge name must be valid");
        let bridge_network = NetworkAttachment::create(
            NetworkId::parse("lan").expect("network id must be valid"),
            NetworkMode::Bridge,
            NetworkDeviceModel::VirtioNetPci,
            None,
            Vec::new(),
            Some(
                BridgeConfiguration::create(Some(bridge_name), None)
                    .expect("bridge config must be valid"),
            ),
        )
        .expect("bridge network must be valid");
        machine
            .attach_network(bridge_network)
            .expect("bridge attach must succeed");
        let id = machine.id().clone();
        repository.insert(machine).expect("insert must succeed");

        let loaded = repository.get(&id).expect("manifest must load");
        let bridge = loaded
            .networks()
            .iter()
            .find(|network| network.id().as_str() == "lan")
            .expect("bridge network must exist")
            .bridge_configuration()
            .expect("bridge config must exist");
        assert_eq!(
            bridge.bridge_name().map(HostNetworkName::as_str),
            Some("br0")
        );
        let _ = fs::remove_dir_all(root);
    }

}
