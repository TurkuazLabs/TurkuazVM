# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0374_android_create_release_download_contract.py
# 📌 Amac: Yeni VM sihirbazinda secilen Android surumunun resolver policy ile dogru image hazirlama akisina baglandigini dogrular
# 📌 Modul - Python
# Version: 0.40.0
# Aciklama: Android release filtreleme, polling, global Cuttlefish target fallback ve branch hint policy kontratlarini kilitler
# Bagimli Oldugu Katman: Service | Tool | View | Config

from pathlib import Path
import yaml

ROOT = Path(__file__).resolve().parents[2]
read = lambda path: (ROOT / path).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> None:
    app = read("apps/desktop/ui/app.js")
    view = read("apps/desktop/ui/views/vm_workspace_view.js")
    index = read("apps/desktop/ui/index.html")
    config = yaml.safe_load(read("config/download-sources.yml"))
    require("function currentCreateAndroidRelease()" in app, "CREATE_ANDROID_RELEASE_HELPER_MISSING")
    require("function handleCreateAndroidImageAction()" in app, "CREATE_ANDROID_DOWNLOAD_ACTION_MISSING")
    require("ensureAndroidRelease(releaseId)" in app, "CREATE_ANDROID_DOWNLOAD_NOT_CONNECTED")
    require("androidImageRelease(image) === releaseId" in app, "CREATE_ANDROID_READY_RELEASE_FILTER_MISSING")
    require("createAndroidImagePollTimer" in app, "CREATE_ANDROID_POLLING_MISSING")
    require('data-home-action="new-vm"' not in view, "HOME_DUPLICATE_NEW_VM_ACTION_PRESENT")
    require('id="new-vm-button"' in index, "GLOBAL_NEW_VM_ACTION_MISSING")
    require("Android Image Hazirla" in index, "CREATE_ANDROID_ACTION_LABEL_MISSING")

    ci = config["sources"]["android_ci"]
    targets = set(ci["target_candidates"])
    for target in (
        "aosp_cf_x86_64_only_phone-userdebug",
        "aosp_cf_x86_64_only_phone-aosp_current-userdebug",
        "aosp_cf_x86_64_phone-trunk_staging-userdebug",
        "aosp_cf_x86_64_phone-userdebug",
    ):
        require(target in targets, f"ANDROID_GLOBAL_TARGET_FALLBACK_MISSING:{target}")
    channels = ci["channels"]
    require("aosp-android16-gsi" in channels["16"]["branch_hints"], "ANDROID_16_GSI_HINT_MISSING")
    require("aosp-android15-gsi" in channels["15"]["branch_hints"], "ANDROID_15_GSI_HINT_MISSING")
    require(config["sources"]["android_release_policy"]["10"] == "source_build", "ANDROID10_SOURCE_BUILD_POLICY_MISSING")
    print("V0374_ANDROID_CREATE_RELEASE_DOWNLOAD_CONTRACT=PASS")


if __name__ == "__main__":
    main()
