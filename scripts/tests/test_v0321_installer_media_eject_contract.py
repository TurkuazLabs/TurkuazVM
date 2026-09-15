# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0321_installer_media_eject_contract.py
# 📌 Amac: Kurulum sonrasi ISO cikarimi ve kalici disk boot zincirini fail-closed dogrular
# 📌 Modul - Python
# Version: 0.32.1
# Aciklama: Core, Engine API, Desktop Controller, Tauri ve UI katmanlarindaki medya cikarim kontratini korur
# Bagimli Oldugu Katman: Controller | Service | Tool | View

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def require(path: str, tokens: tuple[str, ...]) -> None:
    text = (ROOT / path).read_text(encoding="utf-8")
    missing = [token for token in tokens if token not in text]
    if missing:
        raise AssertionError(f"{path}: missing tokens: {missing}")


def main() -> int:
    require("crates/core/src/commands/eject_installer_media_command.rs", (
        "Version: 0.32.1",
        "pub struct EjectInstallerMediaCommand",
        "pub vm_id: String",
    ))
    require("crates/core/src/services/guest_boot_service.rs", (
        "pub fn eject_installer_media",
        "BootOrder::create(vec![BootDevice::Disk], false)",
        "machine.guest_boot().firmware().clone()",
        "ejects_installer_without_deleting_iso_and_persists_disk_boot",
        "assert_eq!(media.imported.borrow().len(), 1)",
    ))
    require("crates/engine-api/src/lib.rs", (
        "ENGINE_API_VERSION: u16 = 24",
        "EjectInstallerMedia { vm_id: String }",
        "installer_media_eject_round_trip_preserves_vm_id",
    ))
    require("apps/engine/src/services/engine_application_service.rs", (
        "EngineAction::EjectInstallerMedia { vm_id }",
        "EjectInstallerMediaCommand::new(vm_id)",
        'debug_error("installer_media_eject_failed")',
    ))
    require("apps/desktop/src-tauri/src/services/desktop_service.rs", (
        "pub fn eject_installer_media",
        "EngineAction::EjectInstallerMedia { vm_id }",
    ))
    require("apps/desktop/src-tauri/src/controllers/desktop_controller.rs", (
        "pub fn eject_installer_media",
        ".eject_installer_media(vm_id)",
    ))
    require("apps/desktop/src-tauri/src/main.rs", (
        "configure_installer_media, eject_installer_media",
        "eject_installer_media,",
    ))
    require("apps/desktop/ui/index.html", (
        'id="installer-media-eject-button"',
        "ISO Cikar ve Diskten Baslat",
        "sanal diski silmez",
    ))
    ui_source = (ROOT / "apps/desktop/ui/app.js").read_text(encoding="utf-8") + "\n" + (ROOT / "apps/desktop/ui/views/vm_workspace_view.js").read_text(encoding="utf-8")
    for token in (
        "async function ejectInstallerMedia()",
        'await invoke("eject_installer_media", { vmId })',
        'data-action="media"',
        'action === "media"',
        'machine.installer_media ? "ISO bagli." : "Bagli ISO yok."',
    ):
        if token not in ui_source:
            raise AssertionError(f"Desktop UI media contract missing token: {token}")
    require("crates/qemu/src/tools/qemu_command_builder.rs", (
        'let key = if boot_order.apply_once() { "once" } else { "order" };',
        'assert!(joined.contains("-boot order=c"));',
    ))
    print("V0321_INSTALLER_MEDIA_EJECT_CONTRACT=PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
