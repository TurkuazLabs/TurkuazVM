// # 📄 Dosya Yolu: /turkuazvm/crates/qemu/src/tools/qemu_command_builder.rs
// # 📌 Amac: VirtualMachine aggregate bilgisinden QEMU launch argumanlari uretir
// # 📌 Modul - Rust
// # Version: 0.41.6
// # Aciklama: Metadata data_root ve buyuk disk image_root koklerini ayirir; legacy data_root disklerini boot sirasinda fallback olarak destekler
// # Bagimli Oldugu Katman: Tool

use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use turkuazvm_core::domain::disk::{DiskBus, DiskFormat};
use turkuazvm_core::domain::guest_boot::{BootDevice, FirmwareSelection};
use turkuazvm_core::domain::hypervisor::AccelerationBackend;
use turkuazvm_core::domain::runtime_media::{AndroidRuntimeMediaPlan, VmRuntimeMediaPlan};
    use turkuazvm_core::domain::network::{
    NetworkDeviceModel, NetworkMode, NetworkRuntimePlan, PortProtocol, PreparedNetworkBackend,
};
use turkuazvm_core::domain::virtual_machine::VirtualMachine;
use turkuazvm_gpu::domain::profile::GpuBackend;

use crate::domain::qemu_runtime::{
    QemuDisplayMode, QemuDisplayRuntimePlan, QemuGpuRuntimeSettings,
};

const ARG_NAME: &str = "-name";
const ARG_MEMORY: &str = "-m";
const ARG_CPU_COUNT: &str = "-smp";
const ARG_ACCELERATION: &str = "-accel";
const ARG_QMP: &str = "-qmp";
const ARG_DISPLAY: &str = "-display";
const ARG_VNC: &str = "-vnc";
const ARG_VGA: &str = "-vga";
const ARG_DRIVE: &str = "-drive";
const ARG_BOOT: &str = "-boot";
const ARG_NETDEV: &str = "-netdev";
const ARG_DEVICE: &str = "-device";
const DIR_MACHINES: &str = "machines";
const ACCEL_WHPX: &str = "whpx";
const ACCEL_KVM: &str = "kvm";
const ACCEL_TCG: &str = "tcg";
const DISPLAY_NONE: &str = "none";
const DISPLAY_EGL_HEADLESS_GL: &str = "egl-headless,gl=on";
const VGA_NONE: &str = "none";
const DEVICE_VIRTIO_GPU: &str = "virtio-vga";
const DEVICE_VIRTIO_GPU_GL: &str = "virtio-vga-gl";
const DEVICE_VIRTIO_GPU_RUTABAGA: &str = "virtio-gpu-rutabaga";
const RFB_LOOPBACK_HOST: &str = "127.0.0.1";
const FORMAT_QCOW2: &str = "qcow2";
const FORMAT_RAW: &str = "raw";
const BUS_VIRTIO: &str = "virtio";
const BUS_IDE: &str = "ide";
const NETDEV_USER: &str = "user";
const DEVICE_VIRTIO_NET_PCI: &str = "virtio-net-pci";
const DEVICE_E1000: &str = "e1000";
const DEVICE_VIRTIO_BLK_PCI: &str = "virtio-blk-pci-non-transitional";
const ANDROID_OS_DRIVE_ID: &str = "turkuaz-android-os";
const ANDROID_OS_DEVICE_ID: &str = "turkuaz-android-os-device";
const PROTOCOL_TCP: &str = "tcp";
const PROTOCOL_UDP: &str = "udp";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QemuCommandBuildError {
    MissingPreparedNetworkBinding(String),
    PreparedNetworkModeMismatch(String),
    MissingInstallerIso(PathBuf),
    MissingUefiCode(PathBuf),
    MissingUefiVars(PathBuf),
    MissingAndroidBootloader(PathBuf),
    MissingAndroidPflash(PathBuf),
    MissingAndroidOsDisk(PathBuf),
    DiskBootRequiresDisk,
    NetworkBootRequiresAdapter,
    ExperimentalGpuBackendNotEnabled,
}

pub struct QemuCommandBuilder;

impl QemuCommandBuilder {
    pub fn build_arguments(
        machine: &VirtualMachine,
        qmp_endpoint: SocketAddr,
        display_plan: QemuDisplayRuntimePlan,
        data_root: &Path,
        network_plan: &NetworkRuntimePlan,
    ) -> Result<Vec<String>, QemuCommandBuildError> {
        Self::build_arguments_with_gpu(
            machine,
            qmp_endpoint,
            display_plan,
            QemuGpuRuntimeSettings {
                backend: GpuBackend::Software,
                hostmem_mib: 1024,
                experimental: false,
            },
            data_root,
            network_plan,
            None,
        )
    }

    pub fn build_arguments_with_gpu(
        machine: &VirtualMachine,
        qmp_endpoint: SocketAddr,
        display_plan: QemuDisplayRuntimePlan,
        gpu: QemuGpuRuntimeSettings,
        data_root: &Path,
        network_plan: &NetworkRuntimePlan,
        runtime_media: Option<&VmRuntimeMediaPlan>,
    ) -> Result<Vec<String>, QemuCommandBuildError> {
        let image_root = data_root.join(DIR_MACHINES);
        Self::build_arguments_with_gpu_and_image_root(
            machine,
            qmp_endpoint,
            display_plan,
            gpu,
            data_root,
            &image_root,
            network_plan,
            runtime_media,
        )
    }

    pub fn build_arguments_with_gpu_and_image_root(
        machine: &VirtualMachine,
        qmp_endpoint: SocketAddr,
        display_plan: QemuDisplayRuntimePlan,
        gpu: QemuGpuRuntimeSettings,
        data_root: &Path,
        image_root: &Path,
        network_plan: &NetworkRuntimePlan,
        runtime_media: Option<&VmRuntimeMediaPlan>,
    ) -> Result<Vec<String>, QemuCommandBuildError> {
        let mut arguments = vec![
            String::from(ARG_NAME),
            machine.name().to_owned(),
            String::from(ARG_MEMORY),
            machine.resources().memory_mib.to_string(),
            String::from(ARG_CPU_COUNT),
            machine.resources().vcpu_count.to_string(),
            String::from(ARG_ACCELERATION),
            Self::acceleration_name(machine.acceleration()).to_owned(),
            String::from(ARG_QMP),
            format!("tcp:{qmp_endpoint},server=on,wait=off"),
        ];

        Self::append_gpu(gpu, &mut arguments)?;
        Self::append_display(display_plan, gpu, &mut arguments);

        Self::append_firmware(machine, data_root, runtime_media, &mut arguments)?;
        Self::append_installer_iso(machine, data_root, &mut arguments)?;
        Self::append_boot_order(machine, runtime_media, &mut arguments)?;
        Self::append_runtime_disks(runtime_media, &mut arguments)?;
        Self::append_disks(machine, data_root, image_root, &mut arguments);
        Self::append_networks(machine, network_plan, &mut arguments)?;

        Ok(arguments)
    }

    fn append_gpu(
        gpu: QemuGpuRuntimeSettings,
        arguments: &mut Vec<String>,
    ) -> Result<(), QemuCommandBuildError> {
        match gpu.backend {
            GpuBackend::Software => {}
            GpuBackend::Virtio2d => {
                arguments.push(String::from(ARG_VGA));
                arguments.push(String::from(VGA_NONE));
                arguments.push(String::from(ARG_DEVICE));
                arguments.push(String::from(DEVICE_VIRTIO_GPU));
            }
            GpuBackend::VirglVenus => {
                arguments.push(String::from(ARG_VGA));
                arguments.push(String::from(VGA_NONE));
                arguments.push(String::from(ARG_DEVICE));
                arguments.push(format!(
                    "{DEVICE_VIRTIO_GPU_GL},hostmem={}M,blob=true,venus=true",
                    gpu.hostmem_mib
                ));
            }
            GpuBackend::Gfxstream => {
                if !gpu.experimental {
                    return Err(QemuCommandBuildError::ExperimentalGpuBackendNotEnabled);
                }
                arguments.push(String::from(ARG_VGA));
                arguments.push(String::from(VGA_NONE));
                arguments.push(String::from(ARG_DEVICE));
                arguments.push(format!(
                    "{DEVICE_VIRTIO_GPU_RUTABAGA},hostmem={}M,gfxstream-vulkan=on,x-gfxstream-gles=on,x-gfxstream-composer=on,wsi=surfaceless",
                    gpu.hostmem_mib
                ));
            }
        }
        Ok(())
    }

    fn append_display(
        plan: QemuDisplayRuntimePlan,
        gpu: QemuGpuRuntimeSettings,
        arguments: &mut Vec<String>,
    ) {
        match plan.mode {
            QemuDisplayMode::NativeRfb => {
                arguments.push(String::from(ARG_DISPLAY));
                arguments.push(String::from(if gpu.backend == GpuBackend::VirglVenus {
                    DISPLAY_EGL_HEADLESS_GL
                } else {
                    DISPLAY_NONE
                }));
                if let Some(display_number) = plan.rfb_display_number {
                    arguments.push(String::from(ARG_VNC));
                    arguments.push(format!("{RFB_LOOPBACK_HOST}:{display_number},share=force-shared"));
                }
            }
            QemuDisplayMode::None => {
                arguments.push(String::from(ARG_DISPLAY));
                arguments.push(String::from(DISPLAY_NONE));
            }
            QemuDisplayMode::Default => {}
        }
    }

    fn append_firmware(
        machine: &VirtualMachine,
        data_root: &Path,
        runtime_media: Option<&VmRuntimeMediaPlan>,
        arguments: &mut Vec<String>,
    ) -> Result<(), QemuCommandBuildError> {
        if let Some(VmRuntimeMediaPlan::Android(media)) = runtime_media {
            return Self::append_android_firmware(media, arguments);
        }
        let FirmwareSelection::Uefi(firmware) = machine.guest_boot().firmware() else {
            return Ok(());
        };
        let code_path = data_root.join(firmware.code_relative_path());
        let vars_path = data_root
            .join(DIR_MACHINES)
            .join(machine.id().as_str())
            .join(firmware.vars_relative_path());
        if !code_path.is_file() {
            return Err(QemuCommandBuildError::MissingUefiCode(code_path));
        }
        if !vars_path.is_file() {
            return Err(QemuCommandBuildError::MissingUefiVars(vars_path));
        }

        arguments.push(String::from(ARG_DRIVE));
        arguments.push(format!(
            "if=pflash,format=raw,readonly=on,file={}",
            Self::escape_drive_value(&code_path.display().to_string())
        ));
        arguments.push(String::from(ARG_DRIVE));
        arguments.push(format!(
            "if=pflash,format=raw,file={}",
            Self::escape_drive_value(&vars_path.display().to_string())
        ));
        Ok(())
    }

    fn append_android_firmware(
        media: &AndroidRuntimeMediaPlan,
        arguments: &mut Vec<String>,
    ) -> Result<(), QemuCommandBuildError> {
        if !media.bootloader_path.is_file() {
            return Err(QemuCommandBuildError::MissingAndroidBootloader(media.bootloader_path.clone()));
        }
        if !media.pflash_path.is_file() {
            return Err(QemuCommandBuildError::MissingAndroidPflash(media.pflash_path.clone()));
        }
        arguments.push(String::from(ARG_DRIVE));
        arguments.push(format!(
            "if=pflash,format=raw,readonly=on,file={}",
            Self::escape_drive_value(&media.bootloader_path.display().to_string())
        ));
        arguments.push(String::from(ARG_DRIVE));
        arguments.push(format!(
            "if=pflash,format=raw,file={}",
            Self::escape_drive_value(&media.pflash_path.display().to_string())
        ));
        Ok(())
    }

    fn append_installer_iso(
        machine: &VirtualMachine,
        data_root: &Path,
        arguments: &mut Vec<String>,
    ) -> Result<(), QemuCommandBuildError> {
        let Some(iso) = machine.guest_boot().installer_iso() else {
            return Ok(());
        };
        let path = data_root
            .join(DIR_MACHINES)
            .join(machine.id().as_str())
            .join(iso.relative_path());
        if !path.is_file() {
            return Err(QemuCommandBuildError::MissingInstallerIso(path));
        }
        arguments.push(String::from(ARG_DRIVE));
        arguments.push(format!(
            "file={},media=cdrom,readonly=on,format=raw,id={}",
            Self::escape_drive_value(&path.display().to_string()),
            iso.id().as_str()
        ));
        Ok(())
    }

    fn append_boot_order(
        machine: &VirtualMachine,
        runtime_media: Option<&VmRuntimeMediaPlan>,
        arguments: &mut Vec<String>,
    ) -> Result<(), QemuCommandBuildError> {
        let boot_order = machine.guest_boot().boot_order();
        let runtime_has_disk = runtime_media.is_some_and(VmRuntimeMediaPlan::has_boot_disk);
        if boot_order.devices().contains(&BootDevice::Disk) && machine.disks().is_empty() && !runtime_has_disk {
            return Err(QemuCommandBuildError::DiskBootRequiresDisk);
        }
        if boot_order.devices().contains(&BootDevice::Network) && machine.networks().is_empty() {
            return Err(QemuCommandBuildError::NetworkBootRequiresAdapter);
        }
        let letters: String = boot_order
            .devices()
            .iter()
            .map(|device| Self::boot_device_letter(*device))
            .collect();
        let key = if boot_order.apply_once() { "once" } else { "order" };
        arguments.push(String::from(ARG_BOOT));
        arguments.push(format!("{key}={letters}"));
        Ok(())
    }

    fn append_runtime_disks(
        runtime_media: Option<&VmRuntimeMediaPlan>,
        arguments: &mut Vec<String>,
    ) -> Result<(), QemuCommandBuildError> {
        let Some(VmRuntimeMediaPlan::Android(media)) = runtime_media else {
            return Ok(());
        };
        if !media.os_disk_path.is_file() {
            return Err(QemuCommandBuildError::MissingAndroidOsDisk(media.os_disk_path.clone()));
        }
        arguments.push(String::from(ARG_DRIVE));
        arguments.push(format!(
            "file={},if=none,format=qcow2,id={ANDROID_OS_DRIVE_ID},aio=threads",
            Self::escape_drive_value(&media.os_disk_path.display().to_string())
        ));
        arguments.push(String::from(ARG_DEVICE));
        arguments.push(format!(
            "{DEVICE_VIRTIO_BLK_PCI},scsi=off,drive={ANDROID_OS_DRIVE_ID},id={ANDROID_OS_DEVICE_ID},bootindex=1"
        ));
        Ok(())
    }

    fn append_disks(
        machine: &VirtualMachine,
        data_root: &Path,
        image_root: &Path,
        arguments: &mut Vec<String>,
    ) {
        for attachment in machine.disks() {
            let primary = image_root
                .join(machine.id().as_str())
                .join(attachment.image().relative_path());
            let legacy = data_root
                .join(DIR_MACHINES)
                .join(machine.id().as_str())
                .join(attachment.image().relative_path());
            let path = if primary.is_file() || !legacy.is_file() {
                primary
            } else {
                legacy
            };
            arguments.push(String::from(ARG_DRIVE));
            arguments.push(format!(
                "file={},if={},format={},id={}",
                Self::escape_drive_value(&path.display().to_string()),
                Self::disk_bus_name(attachment.bus()),
                Self::disk_format_name(attachment.image().format()),
                attachment.image().id().as_str()
            ));
        }
    }

    fn append_networks(
        machine: &VirtualMachine,
        network_plan: &NetworkRuntimePlan,
        arguments: &mut Vec<String>,
    ) -> Result<(), QemuCommandBuildError> {
        for (index, attachment) in machine.networks().iter().enumerate() {
            let binding = network_plan
                .binding(attachment.id())
                .ok_or_else(|| QemuCommandBuildError::MissingPreparedNetworkBinding(
                    attachment.id().as_str().to_owned(),
                ))?;

            let backend_id = format!("net{index}");
            let netdev = match binding.backend() {
                PreparedNetworkBackend::UserNat => {
                    if !matches!(attachment.mode(), NetworkMode::ManagedNat | NetworkMode::UserNat) {
                        return Err(QemuCommandBuildError::PreparedNetworkModeMismatch(
                            attachment.id().as_str().to_owned(),
                        ));
                    }
                    let mut value = format!("{NETDEV_USER},id={backend_id}");
                    if let Some(address) = attachment.managed_address() {
                        value.push_str(&format!(
                            ",net={}/{},host={},dhcpstart={}",
                            address.subnet_address(),
                            address.prefix_length(),
                            address.gateway().unwrap_or_else(|| address.subnet_address()),
                            address.ipv4_address()
                        ));
                    }
                    for rule in attachment.port_forwards() {
                        let host_ip = rule.host_ip().map(|ip| ip.to_string()).unwrap_or_default();
                        let guest_ip = rule.guest_ip().map(|ip| ip.to_string()).unwrap_or_default();
                        value.push_str(&format!(
                            ",hostfwd={}:{}:{}-{}:{}",
                            Self::protocol_name(rule.protocol()),
                            host_ip,
                            rule.host_port(),
                            guest_ip,
                            rule.guest_port()
                        ));
                    }
                    value
                }
                PreparedNetworkBackend::HostTap { interface_name } => {
                    if !matches!(attachment.mode(), NetworkMode::Bridge | NetworkMode::Private) {
                        return Err(QemuCommandBuildError::PreparedNetworkModeMismatch(
                            attachment.id().as_str().to_owned(),
                        ));
                    }
                    format!(
                        "tap,id={backend_id},ifname={},script=no,downscript=no",
                        Self::escape_option_value(interface_name.as_str())
                    )
                }
                PreparedNetworkBackend::HostBridge {
                    bridge_name,
                    helper_path,
                } => {
                    if attachment.mode() != NetworkMode::Bridge {
                        return Err(QemuCommandBuildError::PreparedNetworkModeMismatch(
                            attachment.id().as_str().to_owned(),
                        ));
                    }
                    let mut value = format!(
                        "bridge,id={backend_id},br={}",
                        Self::escape_option_value(bridge_name.as_str())
                    );
                    if let Some(helper_path) = helper_path {
                        value.push_str(&format!(
                            ",helper={}",
                            Self::escape_option_value(&helper_path.display().to_string())
                        ));
                    }
                    value
                }
            };

            let mut device = format!(
                "{},netdev={backend_id}",
                Self::network_device_name(attachment.device_model())
            );
            if let Some(mac_address) = attachment.mac_address() {
                device.push_str(&format!(",mac={}", mac_address.to_canonical_string()));
            }

            arguments.push(String::from(ARG_NETDEV));
            arguments.push(netdev);
            arguments.push(String::from(ARG_DEVICE));
            arguments.push(device);
        }
        Ok(())
    }

    const fn acceleration_name(acceleration: AccelerationBackend) -> &'static str {
        match acceleration {
            AccelerationBackend::Whpx => ACCEL_WHPX,
            AccelerationBackend::Kvm => ACCEL_KVM,
            AccelerationBackend::Tcg => ACCEL_TCG,
        }
    }

    const fn disk_format_name(format: DiskFormat) -> &'static str {
        match format {
            DiskFormat::Qcow2 => FORMAT_QCOW2,
            DiskFormat::Raw => FORMAT_RAW,
        }
    }

    const fn disk_bus_name(bus: DiskBus) -> &'static str {
        match bus {
            DiskBus::Virtio => BUS_VIRTIO,
            DiskBus::Ide => BUS_IDE,
        }
    }

    const fn network_device_name(model: NetworkDeviceModel) -> &'static str {
        match model {
            NetworkDeviceModel::VirtioNetPci => DEVICE_VIRTIO_NET_PCI,
            NetworkDeviceModel::E1000 => DEVICE_E1000,
        }
    }

    const fn protocol_name(protocol: PortProtocol) -> &'static str {
        match protocol {
            PortProtocol::Tcp => PROTOCOL_TCP,
            PortProtocol::Udp => PROTOCOL_UDP,
        }
    }

    const fn boot_device_letter(device: BootDevice) -> char {
        match device {
            BootDevice::Disk => 'c',
            BootDevice::Cdrom => 'd',
            BootDevice::Network => 'n',
        }
    }

    fn escape_drive_value(value: &str) -> String {
        value.replace(',', ",,")
    }

    fn escape_option_value(value: &str) -> String {
        value.replace(',', ",,")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::net::{IpAddr, Ipv4Addr};
    use std::time::{SystemTime, UNIX_EPOCH};
    use turkuazvm_core::domain::disk::{DiskAttachment, DiskBus, DiskFormat, DiskId, DiskImage};
    use turkuazvm_core::domain::guest_boot::{
        BootOrder, FirmwareSelection, GuestBootConfiguration, GuestProfile, IsoAttachment, MediaId,
        UefiFirmware,
    };
    use turkuazvm_core::domain::runtime_media::{AndroidRuntimeMediaPlan, VmRuntimeMediaPlan};
    use turkuazvm_core::domain::network::{
        BridgeConfiguration, HostNetworkName, MacAddress, ManagedAddressConfiguration, NetworkAttachment, NetworkDeviceModel, NetworkId,
        NetworkMode, NetworkRuntimePlan, PortForwardRule, PortProtocol, PreparedNetworkBackend,
        PreparedNetworkBinding,
    };
    use turkuazvm_core::domain::virtual_machine::{VmId, VmResourceConfig};

    fn test_root() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be valid")
            .as_nanos();
        std::env::temp_dir().join(format!("turkuazvm-qemu-builder-{}-{nonce}", std::process::id()))
    }

    fn machine_with_disk(id: &str) -> VirtualMachine {
        let mut machine = VirtualMachine::create(
            VmId::parse(id).expect("id must be valid"),
            "Builder Test",
            VmResourceConfig {
                vcpu_count: 6,
                memory_mib: 8192,
            },
            AccelerationBackend::Kvm,
        )
        .expect("machine must be valid");
        let image = DiskImage::create(
            DiskId::parse("system").expect("disk id must be valid"),
            DiskFormat::Qcow2,
            64 * 1024 * 1024,
            "disks/system.qcow2",
        )
        .expect("disk must be valid");
        machine
            .attach_disk(DiskAttachment::new(image, DiskBus::Virtio, Some(1)))
            .expect("disk attach must succeed");
        machine
    }

    #[test]
    fn builder_maps_storage_and_user_nat_without_qemu_leaking_into_core() {
        let mut machine = machine_with_disk("builder-test");
        let forward = PortForwardRule::create(
            PortProtocol::Tcp,
            Some(Ipv4Addr::LOCALHOST),
            2222,
            None,
            22,
        )
        .expect("port forward must be valid");
        let network_id = NetworkId::parse("default").expect("network id must be valid");
        let network = NetworkAttachment::create(
            network_id.clone(),
            NetworkMode::UserNat,
            NetworkDeviceModel::VirtioNetPci,
            None,
            vec![forward],
            None,
        )
        .expect("network must be valid");
        machine
            .attach_network(network)
            .expect("network attach must succeed");
        let network_plan = NetworkRuntimePlan::new(vec![PreparedNetworkBinding::new(
            network_id,
            PreparedNetworkBackend::UserNat,
        )])
        .expect("network plan must be valid");
        let endpoint = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4444);

        let args = QemuCommandBuilder::build_arguments(
            &machine,
            endpoint,
            QemuDisplayRuntimePlan {
                mode: QemuDisplayMode::None,
                rfb_display_number: None,
            },
            &PathBuf::from("data"),
            &network_plan,
        )
        .expect("command build must succeed");
        let joined = args.join(" ");

        assert!(joined.contains("-m 8192"));
        assert!(joined.contains("-smp 6"));
        assert!(joined.contains("-accel kvm"));
        assert!(joined.contains("tcp:127.0.0.1:4444,server=on,wait=off"));
        assert!(joined.contains("-boot order=c"));
        assert!(joined.contains("file=data/machines/builder-test/disks/system.qcow2"));
        assert!(joined.contains("-netdev user,id=net0,hostfwd=tcp:127.0.0.1:2222-:22"));
        assert!(joined.contains("-device virtio-net-pci,netdev=net0"));
    }

    #[test]
    fn builder_maps_native_rfb_display_to_loopback_vnc() {
        let machine = machine_with_disk("display-test");
        let endpoint = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4444);
        let args = QemuCommandBuilder::build_arguments(
            &machine,
            endpoint,
            QemuDisplayRuntimePlan {
                mode: QemuDisplayMode::NativeRfb,
                rfb_display_number: Some(10),
            },
            &PathBuf::from("data"),
            &NetworkRuntimePlan::default(),
        )
        .expect("native RFB command build must succeed");
        let joined = args.join(" ");

        assert!(joined.contains("-display none"));
        assert!(joined.contains("-vnc 127.0.0.1:10,share=force-shared"));
    }

    #[test]
    fn builder_maps_uefi_iso_and_boot_once() {
        let root = test_root();
        let mut machine = machine_with_disk("guest-boot");
        let code = root.join("firmware/OVMF_CODE.fd");
        let vars = root.join("machines/guest-boot/firmware/OVMF_VARS.fd");
        let iso = root.join("machines/guest-boot/media/installer.iso");
        fs::create_dir_all(code.parent().expect("code parent must exist"))
            .expect("code parent must be created");
        fs::create_dir_all(vars.parent().expect("vars parent must exist"))
            .expect("vars parent must be created");
        fs::create_dir_all(iso.parent().expect("iso parent must exist"))
            .expect("iso parent must be created");
        fs::write(&code, b"code").expect("code must be created");
        fs::write(&vars, b"vars").expect("vars must be created");
        fs::write(&iso, b"iso").expect("iso must be created");

        let configuration = GuestBootConfiguration::create(
            GuestProfile::Windows,
            FirmwareSelection::Uefi(
                UefiFirmware::create("firmware/OVMF_CODE.fd", "firmware/OVMF_VARS.fd")
                    .expect("firmware must be valid"),
            ),
            BootOrder::create(vec![BootDevice::Cdrom, BootDevice::Disk], true)
                .expect("boot order must be valid"),
            Some(
                IsoAttachment::create(
                    MediaId::parse("installer").expect("media id must be valid"),
                    "media/installer.iso",
                )
                .expect("iso attachment must be valid"),
            ),
            None,
        )
        .expect("guest boot config must be valid");
        machine.configure_guest_boot(configuration);
        let endpoint = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4444);

        let args = QemuCommandBuilder::build_arguments(
            &machine,
            endpoint,
            QemuDisplayRuntimePlan {
                mode: QemuDisplayMode::Default,
                rfb_display_number: None,
            },
            &root,
            &NetworkRuntimePlan::default(),
        )
        .expect("command build must succeed");
        let joined = args.join(" ");

        assert!(joined.contains("if=pflash,format=raw,readonly=on"));
        assert!(joined.contains("OVMF_CODE.fd"));
        assert!(joined.contains("OVMF_VARS.fd"));
        assert!(joined.contains("media=cdrom,readonly=on,format=raw,id=installer"));
        assert!(joined.contains("-boot once=dc"));
        assert!(!joined.contains("-display none"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn builder_maps_prepared_host_bridge_backend() {
        let mut machine = machine_with_disk("bridge-test");
        let network_id = NetworkId::parse("bridge0").expect("network id must be valid");
        let bridge_name = HostNetworkName::parse("br0").expect("bridge name must be valid");
        let network = NetworkAttachment::create(
            network_id.clone(),
            NetworkMode::Bridge,
            NetworkDeviceModel::VirtioNetPci,
            None,
            Vec::new(),
            Some(
                BridgeConfiguration::create(Some(bridge_name.clone()), None)
                    .expect("bridge config must be valid"),
            ),
        )
        .expect("network must be valid");
        machine
            .attach_network(network)
            .expect("network attach must succeed");
        let plan = NetworkRuntimePlan::new(vec![PreparedNetworkBinding::new(
            network_id,
            PreparedNetworkBackend::HostBridge {
                bridge_name,
                helper_path: Some(PathBuf::from("/usr/lib/qemu/qemu-bridge-helper")),
            },
        )])
        .expect("network plan must be valid");
        let endpoint = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4444);

        let args = QemuCommandBuilder::build_arguments(
            &machine,
            endpoint,
            QemuDisplayRuntimePlan {
                mode: QemuDisplayMode::None,
                rfb_display_number: None,
            },
            &PathBuf::from("data"),
            &plan,
        )
        .expect("bridge command build must succeed");
        let joined = args.join(" ");
        assert!(joined.contains("-netdev bridge,id=net0,br=br0,helper=/usr/lib/qemu/qemu-bridge-helper"));
    }

    #[test]
    fn builder_maps_prepared_existing_tap_backend() {
        let mut machine = machine_with_disk("tap-test");
        let network_id = NetworkId::parse("tap0").expect("network id must be valid");
        let tap_name = HostNetworkName::parse("TurkuazVM TAP").expect("tap name must be valid");
        let network = NetworkAttachment::create(
            network_id.clone(),
            NetworkMode::Bridge,
            NetworkDeviceModel::E1000,
            None,
            Vec::new(),
            Some(
                BridgeConfiguration::create(None, Some(tap_name.clone()))
                    .expect("bridge config must be valid"),
            ),
        )
        .expect("network must be valid");
        machine
            .attach_network(network)
            .expect("network attach must succeed");
        let plan = NetworkRuntimePlan::new(vec![PreparedNetworkBinding::new(
            network_id,
            PreparedNetworkBackend::HostTap {
                interface_name: tap_name,
            },
        )])
        .expect("network plan must be valid");
        let endpoint = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4444);

        let args = QemuCommandBuilder::build_arguments(
            &machine,
            endpoint,
            QemuDisplayRuntimePlan {
                mode: QemuDisplayMode::None,
                rfb_display_number: None,
            },
            &PathBuf::from("data"),
            &plan,
        )
        .expect("tap command build must succeed");
        let joined = args.join(" ");
        assert!(joined.contains("-netdev tap,id=net0,ifname=TurkuazVM TAP,script=no,downscript=no"));
        assert!(joined.contains("-device e1000,netdev=net0"));
    }


    #[test]
    fn builder_maps_managed_nat_to_portable_qemu_user_backend() {
        let mut machine = machine_with_disk("managed-nat-test");
        let network_id = NetworkId::parse("turkuaz-net-01").expect("network id must be valid");
        let address = ManagedAddressConfiguration::create(
            Ipv4Addr::new(192, 168, 240, 10),
            24,
            Some(Ipv4Addr::new(192, 168, 240, 1)),
            Vec::new(),
        )
        .expect("managed address must be valid");
        let forward = PortForwardRule::create(
            PortProtocol::Tcp,
            Some(Ipv4Addr::LOCALHOST),
            2201,
            Some(Ipv4Addr::new(192, 168, 240, 10)),
            22,
        )
        .expect("port forward must be valid");
        let network = NetworkAttachment::create_with_address(
            network_id.clone(),
            NetworkMode::ManagedNat,
            NetworkDeviceModel::VirtioNetPci,
            Some(MacAddress::parse("02:54:00:12:34:56").expect("mac must be valid")),
            vec![forward],
            None,
            Some(address),
            Some(NetworkId::parse("turkuaz-net-01").expect("fabric id must be valid")),
        )
        .expect("managed network must be valid");
        machine.attach_network(network).expect("network attach must succeed");
        let plan = NetworkRuntimePlan::new(vec![PreparedNetworkBinding::new(
            network_id,
            PreparedNetworkBackend::UserNat,
        )])
        .expect("network plan must be valid");
        let endpoint = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4444);

        let args = QemuCommandBuilder::build_arguments(
            &machine,
            endpoint,
            QemuDisplayRuntimePlan { mode: QemuDisplayMode::None, rfb_display_number: None },
            &PathBuf::from("data"),
            &plan,
        )
        .expect("managed NAT command build must succeed");
        let joined = args.join(" ");

        assert!(joined.contains("-netdev user,id=net0,net=192.168.240.0/24,host=192.168.240.1,dhcpstart=192.168.240.10"));
        assert!(joined.contains("hostfwd=tcp:127.0.0.1:2201-192.168.240.10:22"));
        assert!(joined.contains("virtio-net-pci,netdev=net0,mac=02:54:00:12:34:56"));
    }

    #[test]
    fn builder_accepts_android_runtime_boot_disk_without_generic_vm_disk() {
        let root = test_root();
        fs::create_dir_all(&root).expect("test root must be created");
        let bootloader = root.join("bootloader.qemu");
        let pflash = root.join("pflash.img");
        let overlay = root.join("os-overlay.qcow2");
        fs::write(&bootloader, b"bootloader").expect("bootloader must be created");
        fs::write(&pflash, b"pflash").expect("pflash must be created");
        fs::write(&overlay, b"qcow2").expect("overlay must be created");

        let mut machine = VirtualMachine::create(
            VmId::parse("android-builder-test").expect("id must be valid"),
            "Android Builder Test",
            VmResourceConfig { vcpu_count: 4, memory_mib: 4096 },
            AccelerationBackend::Kvm,
        )
        .expect("machine must be valid");
        machine.configure_guest_boot(GuestBootConfiguration::default_for_profile(GuestProfile::Android, None).expect("guest profile must be valid"));
        let runtime_media = VmRuntimeMediaPlan::Android(AndroidRuntimeMediaPlan {
            bootloader_path: bootloader,
            pflash_path: pflash,
            os_disk_path: overlay,
        });
        let endpoint = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4444);

        let args = QemuCommandBuilder::build_arguments_with_gpu(
            &machine,
            endpoint,
            QemuDisplayRuntimePlan { mode: QemuDisplayMode::None, rfb_display_number: None },
            QemuGpuRuntimeSettings { backend: GpuBackend::Software, hostmem_mib: 1024, experimental: false },
            &root,
            &NetworkRuntimePlan::default(),
            Some(&runtime_media),
        )
        .expect("Android runtime command build must succeed");
        let joined = args.join(" ");
        assert!(joined.contains("if=pflash,format=raw,readonly=on"));
        assert!(joined.contains("format=qcow2,id=turkuaz-android-os"));
        assert!(joined.contains("virtio-blk-pci-non-transitional"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn builder_maps_virgl_venus_gpu_with_headless_gl_context() {
        let machine = machine_with_disk("gpu-venus-test");
        let endpoint = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4444);
        let args = QemuCommandBuilder::build_arguments_with_gpu(
            &machine,
            endpoint,
            QemuDisplayRuntimePlan {
                mode: QemuDisplayMode::NativeRfb,
                rfb_display_number: Some(11),
            },
            QemuGpuRuntimeSettings {
                backend: GpuBackend::VirglVenus,
                hostmem_mib: 2048,
                experimental: false,
            },
            &PathBuf::from("data"),
            &NetworkRuntimePlan::default(),
            None,
        )
        .expect("VirGL/Venus command build must succeed");
        let joined = args.join(" ");

        assert!(joined.contains("-vga none"));
        assert!(joined.contains("virtio-vga-gl,hostmem=2048M,blob=true,venus=true"));
        assert!(joined.contains("-display egl-headless,gl=on"));
        assert!(joined.contains("-vnc 127.0.0.1:11,share=force-shared"));
    }

    #[test]
    fn builder_rejects_gfxstream_without_experimental_flag() {
        let machine = machine_with_disk("gpu-gfxstream-test");
        let endpoint = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4444);
        let error = QemuCommandBuilder::build_arguments_with_gpu(
            &machine,
            endpoint,
            QemuDisplayRuntimePlan {
                mode: QemuDisplayMode::None,
                rfb_display_number: None,
            },
            QemuGpuRuntimeSettings {
                backend: GpuBackend::Gfxstream,
                hostmem_mib: 2048,
                experimental: false,
            },
            &PathBuf::from("data"),
            &NetworkRuntimePlan::default(),
            None,
        )
        .expect_err("GfxStream without experimental flag must fail");

        assert_eq!(error, QemuCommandBuildError::ExperimentalGpuBackendNotEnabled);
    }

}
