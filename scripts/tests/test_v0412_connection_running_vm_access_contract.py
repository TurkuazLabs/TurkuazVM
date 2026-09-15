# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0412_connection_running_vm_access_contract.py
# 📌 Amac: v0.41.2 Baglanti Merkezi calisan VM SSH/RDP erisim hazirlama akisinin dead-end olmamasini fail-closed dogrular
# 📌 Modul - Python
# Version: 0.41.2
# Aciklama: Running Turkuaz NAT VM icin View onayi ve Service stop -> publish -> restart orkestrasyonunu kilitler
# Bagimli Oldugu Katman: Service | Tool | View

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def require(condition: bool, code: str) -> None:
    if not condition:
        raise AssertionError(code)


def main() -> None:
    service = read("apps/desktop/src-tauri/src/services/desktop_service.rs")
    app = read("apps/desktop/ui/app.js")
    index = read("apps/desktop/ui/index.html")

    for token in (
        'const VM_STATE_RUNNING: &str = "running";',
        "fn prepare_vm_tcp_access(",
        "let was_running = match machine.state.as_str()",
        "VM_STATE_STOPPED => false",
        "VM_STATE_RUNNING => true",
        "self.stop_vm(vm_id.clone())",
        "ExternalConnectionTool::find_available_loopback_tcp_port(host_port_start, host_port_end)",
        "self.publish_vm_service(",
        "Ok(_) => self.start_vm(vm_id)",
        "VM onceki calisma durumuna geri getirildi",
    ):
        require(token in service, f"RUNNING_VM_ACCESS_SERVICE_TOKEN_MISSING:{token}")

    require(
        "SSH erisimi yalniz VM dururken hazirlanabilir" not in service,
        "RUNNING_VM_ACCESS_OLD_SSH_STOPPED_ONLY_GUARD_PRESENT",
    )
    require(
        "RDP erisimi yalniz VM dururken hazirlanabilir" not in service,
        "RUNNING_VM_ACCESS_OLD_RDP_STOPPED_ONLY_GUARD_PRESENT",
    )

    for token in (
        "const vmRunning = vmState === STATE_RUNNING;",
        "const vmCanReconfigure = vmStopped || vmRunning;",
        "elements.prepareSshAccess.disabled = !vmCanReconfigure;",
        "elements.prepareRdpAccess.disabled = !vmCanReconfigure;",
        "const restartsRunningVm = vmState === STATE_RUNNING;",
        "VM kisa sure durdurulacak, localhost port yayini eklenecek ve VM yeniden baslatilacak",
        "Host portu eklendi ve VM yeniden baslatildi.",
        "SSH Erisimini Hazirla VM'yi kisa sure durdurur",
        "RDP Erisimini Hazirla VM'yi kisa sure durdurur",
    ):
        require(token in app, f"RUNNING_VM_ACCESS_VIEW_TOKEN_MISSING:{token}")

    require(
        "elements.prepareSshAccess.disabled = !vmStopped;" not in app,
        "RUNNING_VM_ACCESS_OLD_SSH_DISABLED_GUARD_PRESENT",
    )
    require(
        "elements.prepareRdpAccess.disabled = !vmStopped;" not in app,
        "RUNNING_VM_ACCESS_OLD_RDP_DISABLED_GUARD_PRESENT",
    )
    require(
        "calisan VM config'i degistirilmez" not in app,
        "RUNNING_VM_ACCESS_OLD_DEAD_END_HINT_PRESENT",
    )

    require(
        "Calisan Turkuaz NAT VM icin gerekli port yayini onayla otomatik uygulanir." in index,
        "RUNNING_VM_ACCESS_MODAL_COPY_MISSING",
    )

    print("V0412_CONNECTION_RUNNING_VM_ACCESS_CONTRACT=PASS")


if __name__ == "__main__":
    main()
