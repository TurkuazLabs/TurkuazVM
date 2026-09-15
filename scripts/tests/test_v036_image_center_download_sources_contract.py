# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v036_image_center_download_sources_contract.py
# 📌 Amac: Goruntu Merkezi ile Android SDK ve legacy CI kaynak kontratlarini fail-closed dogrular
# 📌 Modul - Python
# Version: 0.40.13
# Aciklama: ISO merkezi, Android 10-17 SDK policy, legacy CI katalogu, schema ve GUI persistence akislarini test eder
# Bagimli Oldugu Katman: Controller | Service | Repo | Tool | View | Config

from pathlib import Path
import yaml

ROOT = Path(__file__).resolve().parents[2]


def read(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


def require(text: str, tokens: tuple[str, ...], label: str) -> None:
    missing = [token for token in tokens if token not in text]
    if missing:
        raise AssertionError(f"{label} missing: {missing}")


def main() -> int:
    sources_text = read("config/download-sources.yml")
    sources = yaml.safe_load(sources_text)
    channels = sources["sources"]["android_ci"]["channels"]
    expected_legacy_ci = {"17": 37, "16": 36, "15": 35, "14": 34, "13": 33, "12L": 32, "12": 31, "11": 30}
    if {key: int(value["expected_sdk"]) for key, value in channels.items()} != expected_legacy_ci:
        raise AssertionError("Legacy Android CI release/SDK channel catalog mismatch")

    policy = sources["sources"]["android_release_policy"]
    expected_api = {"10": 29, "11": 30, "12": 31, "12L": 32, "13": 33, "14": 34, "15": 35, "16": 36, "17": 37}
    if any(policy.get(release) != "android_sdk" for release in expected_api):
        raise AssertionError("Windows Android SDK provider policy missing")
    if {key: int(value) for key, value in sources["sources"]["android_sdk"]["api_levels"].items()} != expected_api:
        raise AssertionError("Android SDK release/API catalog mismatch")
    if sources["paths"].get("android_sdk_tools") != "./data/tools/android-sdk":
        raise AssertionError("Android SDK tool path missing")

    if not sources["sources"]["android_ci"].get("branch_templates") or not sources["sources"]["android_ci"].get("target_candidates"):
        raise AssertionError("Legacy Android CI resolver branch template/target candidates missing")
    for release, channel in channels.items():
        if not channel.get("branch_hints"):
            raise AssertionError(f"Android {release} legacy CI branch hints missing")

    main_config = read("config/turkuazvm.yml")
    require(main_config, (
        "schema_version: 22", "downloads:", "sources_path: config/download-sources.yml",
    ), "main config")
    for legacy_path_token in ("installer_media_download_root:", "root: ./data/cache/artifacts", "output_root: ./data/android-image-builds"):
        if legacy_path_token in main_config:
            raise AssertionError(f"legacy duplicated download path remains in main config: {legacy_path_token}")

    require(read("apps/engine/src/config/engine_config.rs"), (
        "EXPECTED_CONFIG_SCHEMA_VERSION: u16 = 22", "load_download_sources", "AndroidSdkEngineConfig",
        "android_sdk_tools", "AndroidReleaseProviderConfig::AndroidSdk", "parse_android_distribution_channels",
    ), "engine config")
    require(read("crates/guest/src/tools/android_ci_source_provider_tool.rs"), (
        "AndroidCiSourceProviderTool", "request.expected_sdk", "status.json", "BUILD_INFO", "artifact_exists",
    ), "legacy android ci provider")
    require(read("crates/guest/src/tools/android_sdk_distribution_tool.rs"), (
        "AndroidSdkDistributionTool", "system-images;android-", "emulator.exe", "AndroidImageRuntimeKind::SdkEmulator",
    ), "android sdk provider")
    require(read("crates/android-image/src/services/android_distribution_source_resolver_service.rs"), (
        "AndroidDistributionSourceResolverService", "branch_templates", "branch_hints", "self.cache.load",
    ), "legacy android source resolver")
    require(read("crates/android-image/src/domain/image.rs"), (
        "pub requested_release: Option<String>", "AndroidImageRuntimeKind", "SdkEmulator",
    ), "android image domain")
    require(read("crates/repositories/src/repositories/yaml_android_image_repository.rs"), (
        "const IMAGE_SCHEMA_VERSION: u16 = 4", "requested_release: Option<String>", "sdk_emulator",
    ), "android image repo")

    html = read("apps/desktop/ui/index.html")
    js = read("apps/desktop/ui/app.js")
    require(html, (
        'id="image-center-iso-tab"', 'id="image-center-android-tab"', 'id="image-center-iso-panel"',
        'id="image-center-standard-iso-select"', 'id="image-center-standard-iso-action"', 'class="image-center-iso-list expert-only"',
        'id="download-settings-form"', 'id="download-installer-media-path"', 'id="download-android-images-path"',
        'id="download-android-ci-base-url"', 'id="download-official-fallback"',
    ), "desktop html")
    require(js, (
        "function imageCenterIsoEntries()", "function renderImageCenterStandardIso(entries)", "async function openImagesCenter(tab = \"iso\")",
        "async function handleImageCenterIsoAction(event)", "trackImageCenterIsoDownload", "uiState.imageCenterIsoDownloads.size",
        "await ensureAndroidRelease(releaseId);", "requested_release: releaseId", "const canAutoInstall = true;",
        "ANDROID_SDK_IMAGE_SUFFIX", "androidSdkImageId", "isAndroidSdkImage",
        "handleCreateAndroidProgressLog", 'invoke("open_android_image_install_log"',
        "async function refreshDownloadSettings()", "async function handleDownloadSettingsSave(event)",
        'invoke("get_download_settings")', 'invoke("save_download_settings"',
    ), "desktop js")
    require(read("apps/desktop/src-tauri/src/tools/download_settings_tool.rs"), (
        "DownloadSettingsTool", "DownloadSettingsUpdate", "config/download-sources.yml", "android_sdk_tools", "android_sdk", "write_file",
    ), "download settings tool")
    require(read("apps/desktop/src-tauri/src/controllers/desktop_controller.rs"), (
        "get_download_settings", "save_download_settings", "SaveDownloadSettingsRequest", "open_android_image_install_log",
    ), "desktop controller")
    require(read("apps/desktop/src-tauri/src/services/desktop_service.rs"), (
        "DownloadSettingsSummary", "DownloadSettingsTool::read", "DownloadSettingsTool::update", "open_android_image_install_log",
    ), "desktop service")

    print("V036_IMAGE_CENTER_DOWNLOAD_SOURCES_CONTRACT=PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
