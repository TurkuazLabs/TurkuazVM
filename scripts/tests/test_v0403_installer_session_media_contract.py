# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0403_installer_session_media_contract.py
# 📌 Amac: Tek-oturumluk installer ISO yasam dongusunun stop ve runtime-exit yollarinda korunmasini dogrular
# 📌 Modul - Python
# Version: 0.40.3
# Aciklama: ISO'nun calisan oturum boyunca bagli kalmasini, oturum sonlandiginda persistence'tan dusmesini ve sonraki boot'un diskten olmasini kilitler
# Bagimli Oldugu Katman: Service | Repo | View

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def require(path: str, needles: list[str]) -> None:
    text = (ROOT / path).read_text(encoding="utf-8")
    for needle in needles:
        if needle not in text:
            raise AssertionError(f"{path}: missing contract token: {needle}")


def main() -> None:
    require(
        "crates/core/src/domain/guest_boot.rs",
        [
            "pub fn expire_one_shot_installer_media",
            "self.installer_iso.is_none() || !self.boot_order.apply_once()",
            "filter(|device| *device != BootDevice::Cdrom)",
            "self.installer_iso = None;",
            "one_shot_installer_expires_to_disk_boot_after_runtime_session",
        ],
    )
    require(
        "crates/core/src/services/vm_lifecycle_service.rs",
        [
            ".expire_one_shot_installer_media()",
            "one_shot_installer_stays_attached_while_running_and_expires_on_stop",
            "guest_reset_live_ejects_one_shot_installer_and_keeps_vm_running",
            "runtime_exit_expires_one_shot_installer_before_recovery_restart",
            "HypervisorRuntimeEventKind::GuestReset",
            ".eject_installer_media(&event.vm_id, &media_id)",
            "HypervisorRuntimeEventKind::ProcessExited",
        ],
    )
    require(
        "crates/qemu/src/tools/qmp_client_tool.rs",
        [
            "pub fn request_media_eject",
            "COMMAND_EJECT",
            '"device": block_device',
            '"force": true',
            "installer_media_eject_sends_forced_block_device_command",
        ],
    )
    require(
        "crates/qemu/src/tools/qemu_runtime_tool.rs",
        [
            'const QMP_EVENT_RESET: &str = "RESET";',
            "HypervisorRuntimeEventKind::GuestReset",
            "qmp.request_media_eject(media_id)",
        ],
    )
    require(
        "apps/desktop/ui/app.js",
        [
            "ISO ilk boot boyunca takili kalir",
            "guest yeniden baslatildiginda veya VM durdugunda otomatik cikarilir",
            "sonraki boot diskten yapilir",
        ],
    )
    print("V0403_INSTALLER_SESSION_MEDIA_CONTRACT=PASS")


if __name__ == "__main__":
    main()
