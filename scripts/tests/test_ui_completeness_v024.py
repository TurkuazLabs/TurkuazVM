# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_ui_completeness_v024.py
# 📌 Amac: v0.24.0 yonetim ekranlarinin backend ve Desktop baglantilarini fail-closed dogrular
# 📌 Modul - Python
# Version: 0.30.0
# Aciklama: Disk ag gunluk kurulum medyasi ve VM yonetim akislarinda gorunen kontrollerin gercek command zincirine bagli oldugunu denetler
# Bagimli Oldugu Katman: Controller | Service | Repo | Tool | View

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
API = (ROOT / "crates/engine-api/src/lib.rs").read_text(encoding="utf-8")
ENGINE = (ROOT / "apps/engine/src/services/engine_application_service.rs").read_text(encoding="utf-8")
CONTROLLER = (ROOT / "apps/desktop/src-tauri/src/controllers/desktop_controller.rs").read_text(encoding="utf-8")
SERVICE = (ROOT / "apps/desktop/src-tauri/src/services/desktop_service.rs").read_text(encoding="utf-8")
MAIN = (ROOT / "apps/desktop/src-tauri/src/main.rs").read_text(encoding="utf-8")
HTML = (ROOT / "apps/desktop/ui/index.html").read_text(encoding="utf-8")
JS = (ROOT / "apps/desktop/ui/app.js").read_text(encoding="utf-8")
CORE_CONFIG = (ROOT / "crates/core/src/services/vm_configuration_service.rs").read_text(encoding="utf-8")


def require_all(content: str, tokens: tuple[str, ...], scope: str) -> None:
    for token in tokens:
        if token not in content:
            raise SystemExit(f"UI_COMPLETENESS_{scope}_MISSING: {token}")


require_all(API, (
    "ENGINE_API_VERSION: u16 = 24",
    "ResizeVmDisk",
    "DeleteVmDisk",
    "UpdateVm",
    "DeleteVm",
    "ConfigureInstallerMedia",
    "VmDiskDto",
    "VmNetworkDto",
), "API")

require_all(ENGINE, (
    "EngineAction::ResizeVmDisk",
    "EngineAction::DeleteVmDisk",
    "EngineAction::UpdateVm",
    "EngineAction::DeleteVm",
    "EngineAction::ConfigureInstallerMedia",
    "VmConfigurationService",
    "GuestBootService",
), "ENGINE")

require_all(CONTROLLER, (
    "pub fn resize_vm_disk",
    "pub fn delete_vm_disk",
    "pub fn update_vm",
    "pub fn delete_vm",
    "pub fn configure_installer_media",
    "pub fn pick_installer_iso",
    "pub fn list_local_logs",
    "pub fn open_local_log",
    "pub fn detach_vm_network",
), "CONTROLLER")

require_all(SERVICE, (
    "EngineAction::ResizeVmDisk",
    "EngineAction::DeleteVmDisk",
    "EngineAction::UpdateVm",
    "EngineAction::DeleteVm",
    "EngineAction::ConfigureInstallerMedia",
    "LocalFilePickerTool::pick_iso",
    "LocalLogCatalogTool::list",
    "LocalFileViewerTool::open_data_log",
    "EngineAction::DetachVmNetwork",
), "SERVICE")

require_all(MAIN, (
    "resize_vm_disk,",
    "delete_vm_disk,",
    "update_vm,",
    "delete_vm,",
    "configure_installer_media,",
    "pick_installer_iso,",
    "list_local_logs,",
    "open_local_log,",
    "detach_vm_network,",
), "MAIN")

require_all(HTML, (
    'id="logs-button"',
    'id="logs-modal"',
    'id="storage-disk-list"',
    'id="network-attachment-list"',
    'id="installer-media-modal"',
    'id="vm-edit-modal"',
), "VIEW")

require_all(JS, (
    "handleStorageDiskAction",
    'invoke("resize_vm_disk"',
    'invoke("delete_vm_disk"',
    "handleNetworkDetach",
    'invoke("detach_vm_network"',
    "openLogsModal",
    'invoke("list_local_logs"',
    'invoke("open_local_log"',
    "openInstallerMediaModal",
    'invoke("configure_installer_media"',
    "handleVmEdit",
    'invoke("update_vm"',
    'invoke("delete_vm"',
), "SCRIPT")

require_all(CORE_CONFIG, (
    "VmMustBeOffline",
    "VmState::Stopped | VmState::Error",
), "DELETE_POLICY")

for stale in ("create-assign-image", "create-add-network", "handlePostCreateImage", "handlePostCreateNetwork"):
    if stale in HTML or stale in JS:
        raise SystemExit(f"UI_COMPLETENESS_STALE_BYPASS_PRESENT: {stale}")

print("UI_COMPLETENESS_V024_GATE=PASS")
