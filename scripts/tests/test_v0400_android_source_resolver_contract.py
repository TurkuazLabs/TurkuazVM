# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0400_android_source_resolver_contract.py
# 📌 Amac: Windows Android SDK ana provider ve legacy Android CI resolver fallback mimarisini dogrular
# 📌 Modul - Python
# Version: 0.40.13
# Aciklama: SDK policy ile Service -> Provider Tool akisinin aktif oldugunu ve legacy CI discovery/cache yolunun kaybolmadigini kilitler
# Bagimli Oldugu Katman: Service | Repo | Tool | Config

from pathlib import Path
import yaml

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> None:
    config = yaml.safe_load(read("config/download-sources.yml"))
    require(config["schema_version"] == 5, "ANDROID_RESOLVER_SCHEMA_NOT_V5")
    require(config["paths"]["android_source_cache"].endswith("android-distribution-sources.yml"), "ANDROID_SOURCE_CACHE_PATH_MISSING")
    require(config["paths"]["android_sdk_tools"].endswith("data/tools/android-sdk"), "ANDROID_SDK_TOOL_PATH_MISSING")

    expected_api = {"10": 29, "11": 30, "12": 31, "12L": 32, "13": 33, "14": 34, "15": 35, "16": 36, "17": 37}
    policy = config["sources"]["android_release_policy"]
    for release in expected_api:
        require(policy[release] == "android_sdk", f"ANDROID_{release}_SDK_PROVIDER_MISSING")

    sdk = config["sources"]["android_sdk"]
    require(sdk["repository_base_url"] == "https://dl.google.com/android/repository", "ANDROID_SDK_REPOSITORY_CHANGED")
    require(sdk["package_index_url"].endswith("repository2-1.xml"), "ANDROID_SDK_PACKAGE_INDEX_MISSING")
    require(sdk["emulator_package_path"] == "emulator", "ANDROID_EMULATOR_PACKAGE_PATH_CHANGED")
    require(sdk["architecture"] == "x86_64", "ANDROID_SDK_ARCH_CHANGED")
    require({key: int(value) for key, value in sdk["api_levels"].items()} == expected_api, "ANDROID_SDK_API_MAP_CHANGED")
    require(sdk["variant_priority"][0] == "default", "ANDROID_SDK_AOSP_VARIANT_NOT_PRIMARY")
    for variant in sdk["variant_priority"]:
        require(variant in sdk["system_image_indexes"], f"ANDROID_SDK_INDEX_MISSING:{variant}")

    ci = config["sources"]["android_ci"]
    require(ci["base_url"] == "https://ci.android.com", "ANDROID_CI_PRIMARY_SOURCE_CHANGED")
    require(ci["official_base_url"] == "https://ci.android.com", "ANDROID_CI_OFFICIAL_SOURCE_CHANGED")
    require(ci["use_official_fallback"] is True, "ANDROID_CI_OFFICIAL_FALLBACK_DISABLED")
    require("{release}" in " ".join(ci["branch_templates"]), "ANDROID_BRANCH_TEMPLATE_MISSING")
    require("aosp_cf_x86_64_only_phone-userdebug" in ci["target_candidates"], "ANDROID_CURRENT_CUTTLEFISH_TARGET_MISSING")
    require("aosp_cf_x86_64_phone-userdebug" in ci["target_candidates"], "ANDROID_LEGACY_CUTTLEFISH_TARGET_MISSING")
    require(ci["channels"]["11"]["allow_device_bootloader_fallback"] is True, "ANDROID11_LEGACY_BOOTLOADER_FALLBACK_DISABLED")

    domain = read("crates/android-image/src/domain/distribution_source.rs")
    resolver = read("crates/android-image/src/services/android_distribution_source_resolver_service.rs")
    provider = read("crates/guest/src/tools/android_ci_source_provider_tool.rs")
    sdk_provider = read("crates/guest/src/tools/android_sdk_distribution_tool.rs")
    router = read("crates/guest/src/tools/android_distribution_router_tool.rs")
    http = read("crates/guest/src/tools/http_download_tool.rs")
    cache = read("crates/repositories/src/repositories/yaml_android_distribution_source_cache_repository.rs")
    distribution = read("crates/guest/src/tools/android_ci_distribution_tool.rs")
    engine = read("apps/engine/src/services/android_image_application_service.rs")
    desktop_settings = read("apps/desktop/src-tauri/src/tools/download_settings_tool.rs")

    for token in ("AndroidDistributionResolverPolicy", "AndroidDistributionSourceRequest", "AndroidDistributionSource", "cache_key"):
        require(token in domain, f"ANDROID_SOURCE_DOMAIN_MISSING:{token}")
    for token in ("AndroidDistributionSourceResolverService", "self.provider.resolve(&request)", "self.cache.load", "source.matches_request"):
        require(token in resolver, f"ANDROID_RESOLVER_SERVICE_MISSING:{token}")
    for token in ("status.json", "BUILD_INFO", "probe_range", "cvd-host_package.tar.gz", "parse_status_target_candidates", "parse_status_targets", "collect_status_targets", "parse_build_info_string", "HttpDownloadTool"):
        require(token in provider, f"ANDROID_CI_PROVIDER_DISCOVERY_MISSING:{token}")
    require('format!("{artifact_base_url}/raw/{BUILD_INFO_FILE}")' in provider, "ANDROID_BUILD_INFO_NOT_RAW_ENDPOINT")
    require('/view/{BUILD_INFO_FILE}' not in provider, "ANDROID_BUILD_INFO_VIEW_ENDPOINT_RETURNED")
    require('DEVICE_IMAGE_PRIMARY: &str = "aosp_cf_x86_64_phone-img"' in provider, "ANDROID_DOCUMENTED_X86_64_ARTIFACT_NOT_PRIMARY")
    require('DEVICE_IMAGE_FALLBACK: &str = "aosp_cf_x86_64_only_phone-img"' in provider, "ANDROID_ONLY_PHONE_ARTIFACT_FALLBACK_MISSING")
    for token in ("--range", "--max-filesize", "--write-out", "fetch_text_with_context"):
        require(token in http, f"ANDROID_SHARED_HTTP_PROBE_MISSING:{token}")
    for token in ("YamlAndroidDistributionSourceCacheRepository", "CACHE_SCHEMA_VERSION", "atomic_write", "resolved_at_unix"):
        require(token in cache, f"ANDROID_SOURCE_CACHE_REPO_MISSING:{token}")

    require("status.json" not in distribution, "ANDROID_DISTRIBUTION_TOOL_STILL_OWNS_STATUS_DISCOVERY")
    require("parse_last_known_good_build" not in distribution, "ANDROID_DISTRIBUTION_TOOL_STILL_OWNS_BUILD_DISCOVERY")
    for token in ("SharedAndroidDistributionSourceResolver", ".resolve(image)", "device_artifact_name", "host_artifact_name"):
        require(token in distribution, f"ANDROID_DISTRIBUTION_RESOLVER_WIRING_MISSING:{token}")

    for token in ("AndroidSdkDistributionTool", "system-images;android-", "remote_package_block", "parse_archive", "AndroidImageRuntimeKind::SdkEmulator"):
        require(token in sdk_provider, f"ANDROID_SDK_PROVIDER_MISSING:{token}")
    for token in ("AndroidSdk", "cfg!(windows)", "self.android_sdk"):
        require(token in router, f"ANDROID_SDK_ROUTER_MISSING:{token}")
    for token in ("AndroidDistributionSourceResolverService::new", "AndroidCiSourceProviderTool::new", "YamlAndroidDistributionSourceCacheRepository::new", "AndroidSdkDistributionTool::new"):
        require(token in engine, f"ANDROID_ENGINE_RESOLVER_COMPOSITION_MISSING:{token}")

    require("SETTINGS_SCHEMA_VERSION: u16 = 5" in desktop_settings, "DESKTOP_DOWNLOAD_SETTINGS_SCHEMA_DRIFT")
    for token in ("android_release_policy", "android_source_cache", "android_sdk_tools", "android_sdk", "official_base_url", "branch_templates", "target_candidates", "branch_hints"):
        require(token in desktop_settings, f"DESKTOP_DOWNLOAD_SETTINGS_LOSSY_FIELD:{token}")

    print("V0400_ANDROID_SOURCE_RESOLVER_CONTRACT=PASS")


if __name__ == "__main__":
    main()
