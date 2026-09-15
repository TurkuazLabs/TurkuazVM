# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v027_managed_network_contract.py
# 📌 Amac: v0.27.0 managed network, IPAM, WinNAT ve servis yayinlama kontratini fail-closed dogrular
# 📌 Modul - Python
# Version: 0.32.0
# Aciklama: Tarihsel managed network modeli ile v0.32.0 portable QEMU NAT ve Advanced private/TAP uyumlulugunu birlikte test eder
# Bagimli Oldugu Katman: Controller | Service | Repo | Tool | View

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def require(path: str, tokens: tuple[str, ...]) -> None:
    text = (ROOT / path).read_text(encoding="utf-8")
    missing = [token for token in tokens if token not in text]
    if missing:
        raise AssertionError(f"{path}: missing tokens: {missing}")


def main() -> int:
    require("crates/core/src/domain/network.rs", (
        "ManagedNat", "Private", "ManagedAddressConfiguration", "fabric_id: Option<NetworkId>",
        "ManagedFabricRequired", "add_port_forward", "NetworkMode::ManagedNat | NetworkMode::UserNat",
    ))
    require("crates/repositories/src/repositories/yaml_vm_repository.rs", (
        "MANIFEST_SCHEMA_VERSION: u16 = 6", "managed_address", "fabric_id",
    ))
    require("apps/engine/src/config/engine_config.rs", (
        "EXPECTED_CONFIG_SCHEMA_VERSION: u16 = 22", "default_profile", "ManagedNetworkPoolPolicy",
        "windows_managed_helper",
    ))
    require("apps/engine/src/services/engine_application_service.rs", (
        "NativeNetworkSubnetTool::resolve_private_24", "allocate_managed_ipv4", "deterministic_vm_mac",
        "PublishNetworkServiceCommand::new", "NetworkMode::ManagedNat", "NetworkMode::Private",
    ))
    require("crates/engine-api/src/lib.rs", (
        "ENGINE_API_VERSION: u16 = 24", "AttachNetworkProfile", "PublishVmService", "UnpublishVmService",
        "pub fabric_id: Option<String>", "pub ipv4_address: Option<String>",
    ))
    require("crates/platform/src/tools/native_network_tool.rs", (
        "Managed DHCP UDP/67 bind failed", "prepare_windows_managed", "register_managed_dhcp_lease",
        "NetworkMode::ManagedNat | NetworkMode::UserNat", "PreparedNetworkBackend::UserNat",
        "NetworkMode::Private => self.prepare_windows_managed", "PreparedNetworkBackend::HostTap",
    ))
    require("crates/platform/src/tools/native_network_subnet_tool.rs", (
        "Get-NetRoute -AddressFamily IPv4", "192, 168", "172, 31", "10, 240",
    ))
    require("scripts/network_windows_managed.ps1", (
        "tapctl.exe", "Invoke-TurkuazNetshBridge", '"bridge", "create"', "New-NetNat", "Add-NetNatStaticMapping",
        "bridge_if_index", "TURKUAZ_NETWORK_ADMIN_REQUIRED",
    ))
    require("scripts/start_turkuazvm.ps1", (
        "Advanced TAP tapctl", "-NeedsOpenVpn $false", "QEMU User NAT hazir; TAP/OpenVPN gerekmez.",
    ))
    require("config/turkuazvm.yml", (
        "schema_version: 22", "default_profile: managed_nat", "subnet: 192.168.240.0",
        "private_network_id: turkuaz-private-01", 'openvpn_package_id: ""',
    ))
    launcher = (ROOT / "scripts/start_turkuazvm.ps1").read_text(encoding="utf-8")
    if 'Start-Process -FilePath "powershell.exe"' in launcher and '-Verb RunAs' in launcher:
        raise AssertionError("scripts/start_turkuazvm.ps1: global launcher elevation must not return")
    print("V027_MANAGED_NETWORK_CONTRACT_OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
