# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_post_create_storage_flow_contract.py
# 📌 Amac: VM olusturma sonrasi Disk ve Ileri adimi UX kontratini regression olarak dogrular
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Disk basarisi, hedef VM kilidi ve Android Image/Kurulum Medyasi/Ag sirali yonlendirmelerini fail-closed kontrol eder
# Bagimli Oldugu Katman: Tool | View

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
INDEX = (ROOT / "apps" / "desktop" / "ui" / "index.html").read_text(encoding="utf-8")
APP_JS = (ROOT / "apps" / "desktop" / "ui" / "app.js").read_text(encoding="utf-8")
STYLE = (ROOT / "apps" / "desktop" / "ui" / "styles.css").read_text(encoding="utf-8")

REQUIRED_VIEW_TOKENS = (
    'id="storage-flow-progress"',
    'id="storage-flow-footer"',
    'id="storage-next-button"',
    'id="storage-next-button" class="primary-button" type="button" disabled',
    'id="storage-flow-image-step"',
    'id="storage-flow-media-step"',
    'id="storage-flow-network-step"',
    'id="storage-flow-summary-step"',
)

REQUIRED_SCRIPT_TOKENS = (
    "storageFlowVmId: null",
    "storageFlowNextStep: null",
    "storageFlowDiskReady: false",
    "configureStorageFlow",
    "setStorageFlowDiskReady",
    "handleStorageFlowNext",
    'const nextStep = uiState.createdVmSourceKind === "android_image" ? "android-image" : (["iso", "manual"].includes(uiState.createdVmSourceKind) ? "installer-media" : "network");',
    "await openStorageModal(vmId, size, { vmId, nextStep });",
    "elements.storageVm.disabled = true;",
    'await openAndroidImagesForVm(vmId, { vmId, nextStep: "network" });',
    'openInstallerMediaModal(vmId);',
    'await openNetworkModal(vmId, { vmId, fromAndroid: false });',
)

REQUIRED_STYLE_TOKENS = (
    ".storage-wizard-progress",
    ".configuration-flow-footer",
    ".configuration-wizard-progress",
)

for token in REQUIRED_VIEW_TOKENS:
    if token not in INDEX:
        raise SystemExit(f"POST_CREATE_STORAGE_VIEW_TOKEN_MISSING: {token}")

for token in REQUIRED_SCRIPT_TOKENS:
    if token not in APP_JS:
        raise SystemExit(f"POST_CREATE_STORAGE_SCRIPT_TOKEN_MISSING: {token}")

for token in REQUIRED_STYLE_TOKENS:
    if token not in STYLE:
        raise SystemExit(f"POST_CREATE_STORAGE_STYLE_TOKEN_MISSING: {token}")

handler_start = APP_JS.index("async function handleStorageDiskCreate")
handler_end = APP_JS.index("async function handleStorageFlowNext", handler_start)
disk_handler = APP_JS[handler_start:handler_end]
if "openAndroidImagesForVm" in disk_handler or "openNetworkModal" in disk_handler:
    raise SystemExit("POST_CREATE_STORAGE_AUTO_ADVANCE_PRESENT")

print("POST_CREATE_STORAGE_FLOW_CONTRACT_OK")
