# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_android_start_readiness_contract.py
# 📌 Amac: Android VM start readiness ve sirali image atama akisinin UI kontratini dogrular
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Disk, Android image assignment, bulk start ve wizard yonlendirmelerinin regression kontrolunu yapar
# Bagimli Oldugu Katman: View | Tool

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
APP_JS = (ROOT / "apps" / "desktop" / "ui" / "app.js").read_text(encoding="utf-8")
WORKSPACE_VIEW = (ROOT / "apps" / "desktop" / "ui" / "views" / "vm_workspace_view.js").read_text(encoding="utf-8")
UI_SOURCE = APP_JS + "\n" + WORKSPACE_VIEW
INDEX = (ROOT / "apps" / "desktop" / "ui" / "index.html").read_text(encoding="utf-8")
ENGINE = (ROOT / "apps" / "engine" / "src" / "services" / "engine_application_service.rs").read_text(encoding="utf-8")

REQUIRED_APP_TOKENS = (
    "androidAssignments: {}",
    "refreshAndroidAssignmentReadiness",
    "hasAndroidImageAssignment(machine)",
    "isRecoverableStartPreflightError(machine)",
    'data-action="assign-image"',
    "openAndroidImagesForVm",
    "handleAndroidImageFlowNext",
    'showToast("Siradaki adim", "Android image atayin.", "info")',
)

for token in REQUIRED_APP_TOKENS:
    if token not in UI_SOURCE:
        raise SystemExit(f"ANDROID_START_READINESS_TOKEN_MISSING: {token}")

if 'id="android-image-next-button"' not in INDEX:
    raise SystemExit("ANDROID_IMAGE_WIZARD_NEXT_ACTION_MISSING")
if 'id="create-assign-image"' in INDEX:
    raise SystemExit("ANDROID_DIRECT_POST_CREATE_BYPASS_PRESENT")

if "Android VM requires an assigned Ready Android image before start" not in ENGINE:
    raise SystemExit("ANDROID_ENGINE_ASSIGNMENT_MESSAGE_MISSING")

print("ANDROID_START_READINESS_CONTRACT_OK")
