# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0376_android11_build_info_contract.py
# 📌 Amac: Android 11 CI BUILD_INFO legacy anahtarlarini ve SDK/release fallback dogrulamasini Source Provider katmaninda korur
# 📌 Modul - Python
# Version: 0.40.0
# Aciklama: BUILD_INFO parserinin Distribution Tool'dan Provider Tool'a tasinmasi sonrasi Android 11 legacy metadata uyumlulugunu kilitler
# Bagimli Oldugu Katman: Tool | Service | Config

from pathlib import Path
import yaml

ROOT = Path(__file__).resolve().parents[2]
read = lambda path: (ROOT / path).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> None:
    provider = read("crates/guest/src/tools/android_ci_source_provider_tool.rs")
    config = yaml.safe_load(read("config/download-sources.yml"))
    require('"ro.build.version.sdk"' in provider, "ANDROID_BUILD_INFO_LEGACY_SDK_ALIAS_MISSING")
    require('"ro.build.version.release"' in provider, "ANDROID_BUILD_INFO_LEGACY_RELEASE_ALIAS_MISSING")
    require("(request.expected_sdk, actual_sdk)" in provider, "ANDROID_SDK_OPTIONAL_VALIDATION_MISSING")
    require("(request.release.as_deref(), actual_release.as_deref())" in provider, "ANDROID_RELEASE_FALLBACK_VALIDATION_MISSING")
    require("sdk_level: actual_sdk.or(request.expected_sdk)" in provider, "ANDROID_EXPECTED_SDK_METADATA_FALLBACK_MISSING")
    require("android_release: actual_release.or_else(|| request.release.clone())" in provider, "ANDROID_RELEASE_METADATA_FALLBACK_MISSING")
    channel = config["sources"]["android_ci"]["channels"]["11"]
    require(channel["expected_sdk"] == 30, "ANDROID_11_EXPECTED_SDK_CHANGED")
    require(channel["branch_hints"][0] == "aosp-android11-gsi", "ANDROID_11_RELEASE_BRANCH_HINT_CHANGED")
    require(channel["allow_device_bootloader_fallback"] is True, "ANDROID_11_LEGACY_FALLBACK_DISABLED")
    require("aosp_cf_x86_64_phone-userdebug" in config["sources"]["android_ci"]["target_candidates"], "ANDROID_11_LEGACY_TARGET_MISSING")
    print("V0376_ANDROID11_BUILD_INFO_CONTRACT=PASS")


if __name__ == "__main__":
    main()
