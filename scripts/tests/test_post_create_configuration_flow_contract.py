# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_post_create_configuration_flow_contract.py
# 📌 Amac: VM olusturma sonrasi medya, Android Image, Ag ve Ozet adimlarinin sirali Ileri kontratini dogrular
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Her adimin basari olmadan ilerlememesini, hedef VM kilidini ve dogrudan bypass kontrolu olmamasini fail-closed kontrol eder
# Bagimli Oldugu Katman: Tool | View

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
INDEX = (ROOT / "apps" / "desktop" / "ui" / "index.html").read_text(encoding="utf-8")
APP_JS = (ROOT / "apps" / "desktop" / "ui" / "app.js").read_text(encoding="utf-8")
STYLE = (ROOT / "apps" / "desktop" / "ui" / "styles.css").read_text(encoding="utf-8")

REQUIRED_VIEW_IDS = (
    'id="android-image-flow-progress"',
    'id="android-image-flow-footer"',
    'id="android-image-next-button"',
    'id="installer-media-modal"',
    'id="installer-media-next-button"',
    'id="network-flow-progress"',
    'id="network-flow-footer"',
    'id="network-next-button"',
    'id="configuration-complete-modal"',
    'id="configuration-complete-done"',
)

REQUIRED_SCRIPT_TOKENS = (
    "androidImageFlowVmId: null",
    "androidImageFlowReady: false",
    "installerMediaFlowVmId: null",
    "installerMediaReady: false",
    "networkFlowVmId: null",
    "networkFlowReady: false",
    "configureAndroidImageFlow",
    "setAndroidImageFlowReady",
    "handleAndroidImageFlowNext",
    "openInstallerMediaModal",
    "setInstallerMediaReady",
    "handleInstallerMediaNext",
    "configureNetworkFlow",
    "setNetworkFlowReady",
    "handleNetworkFlowNext",
    "openConfigurationCompleteModal",
    "closeConfigurationCompleteModal",
    'elements.androidImageQuickVm.disabled = true;',
    'elements.networkVm.disabled = true;',
)

REQUIRED_STYLE_TOKENS = (
    ".configuration-flow-footer",
    ".configuration-wizard-progress",
    ".configuration-summary-grid",
    ".configuration-complete-card",
)

for token in REQUIRED_VIEW_IDS:
    if token not in INDEX:
        raise SystemExit(f"POST_CREATE_CONFIGURATION_VIEW_TOKEN_MISSING: {token}")

for token in REQUIRED_SCRIPT_TOKENS:
    if token not in APP_JS:
        raise SystemExit(f"POST_CREATE_CONFIGURATION_SCRIPT_TOKEN_MISSING: {token}")

for token in REQUIRED_STYLE_TOKENS:
    if token not in STYLE:
        raise SystemExit(f"POST_CREATE_CONFIGURATION_STYLE_TOKEN_MISSING: {token}")

network_start = APP_JS.index("async function handleNetworkAttach")
network_end = APP_JS.index("async function handleNetworkFlowNext", network_start)
network_handler = APP_JS[network_start:network_end]
if "openConfigurationCompleteModal" in network_handler:
    raise SystemExit("POST_CREATE_NETWORK_AUTO_ADVANCE_PRESENT")

assign_start = APP_JS.index("async function handleAndroidImageListAction")
assign_end = APP_JS.index("async function refreshAndroidImageAssignment", assign_start)
assign_handler = APP_JS[assign_start:assign_end]
if "openNetworkModal" in assign_handler:
    raise SystemExit("POST_CREATE_ANDROID_IMAGE_AUTO_ADVANCE_PRESENT")

media_start = APP_JS.index("async function attachInstallerMedia")
media_end = APP_JS.index("async function handleInstallerMediaNext", media_start)
media_handler = APP_JS[media_start:media_end]
if "openNetworkModal" in media_handler:
    raise SystemExit("POST_CREATE_INSTALLER_MEDIA_AUTO_ADVANCE_PRESENT")

for stale in ('id="create-add-network"', 'id="create-assign-image"'):
    if stale in INDEX:
        raise SystemExit(f"POST_CREATE_DIRECT_BYPASS_PRESENT: {stale}")

print("POST_CREATE_CONFIGURATION_FLOW_CONTRACT_OK")
