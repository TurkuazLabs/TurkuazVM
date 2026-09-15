# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0320_qemu_user_nat_contract.py
# 📌 Amac: v0.32.0 portable QEMU User NAT tabanli Turkuaz NAT kontratini dogrular
# 📌 Modul - Python
# Version: 0.32.0
# Aciklama: Varsayilan agin OpenVPN/TAP zorunlulugu olmadan QEMU User NAT backendine hazirlandigini fail-closed denetler
# Bagimli Oldugu Katman: Service | Tool | View

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PLATFORM = (ROOT / "crates/platform/src/tools/native_network_tool.rs").read_text(encoding="utf-8")
QEMU = (ROOT / "crates/qemu/src/tools/qemu_command_builder.rs").read_text(encoding="utf-8")
LAUNCHER = (ROOT / "scripts/start_turkuazvm.ps1").read_text(encoding="utf-8")
CONFIG = (ROOT / "config/turkuazvm.yml").read_text(encoding="utf-8")
UI = (ROOT / "apps/desktop/ui/index.html").read_text(encoding="utf-8")
APP = (ROOT / "apps/desktop/ui/app.js").read_text(encoding="utf-8")
ENGINE = (ROOT / "apps/engine/src/services/engine_application_service.rs").read_text(encoding="utf-8")
LINUX_LAUNCHER = (ROOT / "scripts/start_turkuazvm_linux.sh").read_text(encoding="utf-8")

required_platform = (
    "Version: 0.32.0",
    "NetworkMode::ManagedNat | NetworkMode::UserNat",
    "PreparedNetworkBackend::UserNat",
    "managed_nat: true",
    "cleanup_previous_lease_for_machine",
)
for token in required_platform:
    if token not in PLATFORM:
        raise SystemExit(f"V0320_PLATFORM_MISSING: {token}")

required_qemu = (
    "Version: 0.32.0",
    '",net={}/{},host={},dhcpstart={}"',
    "hostfwd={}:{}:{}-{}:{}",
    "builder_maps_managed_nat_to_portable_qemu_user_backend",
)
for token in required_qemu:
    if token not in QEMU:
        raise SystemExit(f"V0320_QEMU_MISSING: {token}")

if '-NeedsOpenVpn $false' not in LAUNCHER:
    raise SystemExit("V0320_LAUNCHER_OPENVPN_NOT_OPTIONAL")
if 'if ($null -eq $Tapctl) { $Failures.Add(' in LAUNCHER:
    raise SystemExit("V0320_LAUNCHER_STILL_REQUIRES_TAP")
if 'default_profile: managed_nat' not in CONFIG:
    raise SystemExit("V0320_DEFAULT_PROFILE_CHANGED")
if 'openvpn_package_id: ""' not in CONFIG:
    raise SystemExit("V0320_OPENVPN_CONFIG_NOT_OPTIONAL")
if 'Turkuaz NAT - QEMU NAT + Internet' not in UI:
    raise SystemExit("V0320_UI_PORTABLE_NAT_LABEL_MISSING")
if 'QEMU NAT guest IP adresine hosttan dogrudan baglanmaz' not in APP:
    raise SystemExit("V0320_CONNECTION_HOSTFWD_GUIDANCE_MISSING")
if 'Some(host_ip)' not in ENGINE or 'let host_ip = std::net::Ipv4Addr::LOCALHOST;' not in ENGINE:
    raise SystemExit("V0320_HOSTFWD_NOT_LOCAL_ONLY")
if 'ensure_host_port_available' not in ENGINE:
    raise SystemExit("V0320_GLOBAL_HOST_PORT_GUARD_MISSING")
if 'nextAvailableHostPort' not in APP:
    raise SystemExit("V0320_UI_HOST_PORT_ALLOCATOR_MISSING")
if 'status "Turkuaz NAT Runtime" "QEMU_USER_NAT"' not in LINUX_LAUNCHER:
    raise SystemExit("V0320_LINUX_LAUNCHER_NAT_MISSING")
if 'status "OpenVPN / TAP" "NOT_REQUIRED"' not in LINUX_LAUNCHER:
    raise SystemExit("V0320_LINUX_LAUNCHER_TAP_STILL_REQUIRED")

print("V0320_QEMU_USER_NAT_CONTRACT=PASS")
