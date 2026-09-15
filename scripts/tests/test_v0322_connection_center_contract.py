# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0322_connection_center_contract.py
# 📌 Amac: v0.32.2 Connection Center SSH/RDP baglanti kontratini fail-closed dogrular
# 📌 Modul - Python
# Version: 0.32.2
# Aciklama: Tool, Service, Controller, Tauri ve View katmanlarindaki gercek baglanti aksiyonlarini ve network state guvenligini denetler
# Bagimli Oldugu Katman: Controller | Service | Tool | View

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def require(path: str, tokens: tuple[str, ...]) -> None:
    text = (ROOT / path).read_text(encoding="utf-8")
    missing = [token for token in tokens if token not in text]
    if missing:
        raise AssertionError(f"{path}: missing tokens: {missing}")


def main() -> int:
    require("apps/desktop/src-tauri/src/tools/external_connection_tool.rs", (
        "Version: 0.32.2",
        "pub struct ExternalConnectionTool",
        "find_available_loopback_tcp_port",
        "pub fn test_tcp",
        "pub fn open_ssh",
        "pub fn open_rdp",
        "validate_username",
    ))
    require("apps/desktop/src-tauri/src/services/desktop_service.rs", (
        "pub struct ConnectionProbeSummary",
        "pub fn prepare_vm_ssh_access",
        "pub fn prepare_vm_rdp_access",
        "SSH_HOST_PORT_START: u16 = 2222",
        "RDP_HOST_PORT_START: u16 = 33890",
        "ExternalConnectionTool::find_available_loopback_tcp_port",
        "ExternalConnectionTool::test_tcp",
        "DesktopHostMode::Local",
        "VM_STATE_STOPPED",
    ))
    require("apps/desktop/src-tauri/src/controllers/desktop_controller.rs", (
        "pub fn prepare_vm_ssh_access",
        "pub fn prepare_vm_rdp_access",
        "pub fn test_tcp_connection",
        "pub fn open_ssh_connection",
        "pub fn open_rdp_connection",
    ))
    require("apps/desktop/src-tauri/src/main.rs", (
        "prepare_vm_ssh_access",
        "prepare_vm_rdp_access",
        "test_tcp_connection",
        "open_ssh_connection",
        "open_rdp_connection",
    ))
    require("apps/desktop/ui/index.html", (
        "Baglanti Merkezi",
        'id="prepare-ssh-access"',
        'id="prepare-rdp-access"',
        'id="test-ssh-connection"',
        'id="open-ssh-connection"',
        'id="open-rdp-connection"',
        'id="connection-start-vm"',
    ))
    require("apps/desktop/ui/app.js", (
        "function populateNetworkVmSelect",
        "async function prepareConnectionAccess",
        "async function testConnection",
        "async function openSshConnection",
        "async function openRdpConnection",
        "async function startConnectionVm",
        'const command = kind === "ssh" ? "prepare_vm_ssh_access" : "prepare_vm_rdp_access"',
        'await invoke("test_tcp_connection"',
        'await invoke("open_ssh_connection"',
        "Remote Engine Turkuaz NAT loopback",
        "QEMU NAT guest IP adresine hosttan dogrudan baglanmaz",
        "elements.networkAttachButton.disabled = !canMutateNetwork",
    ))
    print("V0322_CONNECTION_CENTER_CONTRACT=PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
