# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0413_android_sdk_windows_provider_contract.py
# 📌 Amac: Windows Android 10-17 resmi SDK System Image ve Emulator provider mimarisini fail-closed dogrular
# 📌 Modul - Python
# Version: 0.40.13
# Aciklama: Config -> Service -> Tool -> Runtime -> View zincirinde SDK katalogu, AVD, WHPX ve log acma kontratini kilitler
# Bagimli Oldugu Katman: Service | Repo | Tool | View | Config

from pathlib import Path
import re
import yaml

ROOT = Path(__file__).resolve().parents[2]


def read(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def require_tokens(text: str, tokens: tuple[str, ...], label: str) -> None:
    missing = [token for token in tokens if token not in text]
    if missing:
        raise AssertionError(f"{label}:{missing}")


def main() -> None:
    config = yaml.safe_load(read("config/download-sources.yml"))
    require(config["schema_version"] == 5, "DOWNLOAD_SOURCE_SCHEMA_DRIFT")
    expected_api = {"10": 29, "11": 30, "12": 31, "12L": 32, "13": 33, "14": 34, "15": 35, "16": 36, "17": 37}
    policy = config["sources"]["android_release_policy"]
    require(set(policy) == set(expected_api), "ANDROID_RELEASE_POLICY_SET_DRIFT")
    require(all(policy[release] == "android_sdk" for release in expected_api), "WINDOWS_ANDROID_SDK_POLICY_MISSING")

    sdk = config["sources"]["android_sdk"]
    require(config["paths"]["android_sdk_tools"] == "./data/tools/android-sdk", "ANDROID_SDK_TOOL_ROOT_DRIFT")
    require(sdk["repository_base_url"] == "https://dl.google.com/android/repository", "ANDROID_SDK_REPOSITORY_NOT_OFFICIAL")
    require(sdk["package_index_url"] == "https://dl.google.com/android/repository/repository2-1.xml", "ANDROID_SDK_PACKAGE_INDEX_DRIFT")
    require(sdk["emulator_package_path"] == "emulator", "ANDROID_EMULATOR_PACKAGE_PATH_DRIFT")
    require(sdk["architecture"] == "x86_64", "ANDROID_SDK_ARCH_DRIFT")
    require(sdk["emulator_console_port_min"] == 5554 and sdk["emulator_console_port_max"] == 5680, "ANDROID_EMULATOR_PORT_RANGE_DRIFT")
    require(sdk["variant_priority"] == ["default", "google_apis", "google_apis_playstore"], "ANDROID_SDK_VARIANT_PRIORITY_DRIFT")
    require({release: int(api) for release, api in sdk["api_levels"].items()} == expected_api, "ANDROID_SDK_API_MAP_DRIFT")
    for variant in sdk["variant_priority"]:
        require(str(sdk["system_image_indexes"][variant]).startswith("https://dl.google.com/android/repository/"), f"ANDROID_SDK_INDEX_NOT_OFFICIAL:{variant}")

    engine_config = read("apps/engine/src/config/engine_config.rs")
    require_tokens(engine_config, (
        "pub struct AndroidSdkEngineConfig", "AndroidDistributionProviderEngineConfig::AndroidSdk",
        "AndroidReleaseProviderConfig::AndroidSdk", "android_sdk_tools", "android_sdk.api_levels",
        "emulator_console_port_min", "emulator_console_port_max", "parse_android_distribution_providers",
    ), "ENGINE_SDK_CONFIG_MISSING")

    provider = read("crates/guest/src/tools/android_sdk_distribution_tool.rs")
    require_tokens(provider, (
        "pub struct AndroidSdkDistributionTool", "system-images;android-{api_level};{variant};{}",
        "remote_package_block", "parse_archive", "host-os", "emulator.exe",
        "AndroidImageRuntimeKind::SdkEmulator", "validate_sdk_emulator_candidate", "distribution.yml",
        "install.log", "cfg!(windows)",
    ), "ANDROID_SDK_PROVIDER_MISSING")

    router = read("crates/guest/src/tools/android_distribution_router_tool.rs")
    require_tokens(router, (
        "AndroidDistributionProvider::AndroidSdk", "cfg!(windows)", "self.android_sdk.prepare_install",
        "self.android_sdk.install", "self.android_sdk.progress", "self.android_sdk.cancel",
    ), "ANDROID_SDK_ROUTER_MISSING")

    image_domain = read("crates/android-image/src/domain/image.rs")
    artifact_domain = read("crates/android-image/src/domain/artifact.rs")
    image_repo = read("crates/repositories/src/repositories/yaml_android_image_repository.rs")
    require_tokens(image_domain, ("pub enum AndroidImageRuntimeKind", "SdkEmulator", "stock_sdk_emulator_x86_64"), "ANDROID_IMAGE_RUNTIME_KIND_MISSING")
    require_tokens(artifact_domain, ("Ramdisk", "System", "validate_sdk_emulator_candidate"), "ANDROID_SDK_ARTIFACT_MODEL_MISSING")
    require("const IMAGE_SCHEMA_VERSION: u16 = 4" in image_repo, "ANDROID_IMAGE_SCHEMA_NOT_V4")
    require_tokens(image_repo, ("sdk_emulator", "LEGACY_IMAGE_SCHEMA_V3", "LEGACY_IMAGE_SCHEMA_V2"), "ANDROID_IMAGE_SCHEMA_MIGRATION_MISSING")

    runtime_media_domain = read("crates/core/src/domain/runtime_media.rs")
    runtime_media_tool = read("crates/guest/src/tools/android_runtime_media_tool.rs")
    runtime_tool = read("apps/engine/src/tools/android_sdk_emulator_runtime_tool.rs")
    runtime_router = read("apps/engine/src/tools/engine_hypervisor_runtime_tool.rs")
    lifecycle = read("crates/core/src/services/vm_lifecycle_service.rs")
    require_tokens(runtime_media_domain, ("AndroidSdkEmulatorRuntimeMediaPlan", "AndroidSdkEmulator", "uses_host_network_runtime"), "ANDROID_SDK_RUNTIME_MEDIA_DOMAIN_MISSING")
    require_tokens(runtime_media_tool, (
        "prepare_sdk_emulator", "emulator.exe", "config.ini", "userdata-qemu.img",
        "console_port", "AndroidImageRuntimeKind::SdkEmulator",
    ), "ANDROID_SDK_AVD_PREPARE_MISSING")
    require_tokens(runtime_tool, (
        'arg("-avd")', 'arg("-port")', 'arg("-accel").arg("auto")',
        'env("ANDROID_AVD_HOME"', 'env("ANDROID_SDK_ROOT"', "android-emulator", "port_in_use",
    ), "ANDROID_EMULATOR_RUNTIME_MISSING")
    require_tokens(runtime_router, ("AndroidSdkEmulatorRuntimeTool", "VmRuntimeMediaPlan::AndroidSdkEmulator"), "HYPERVISOR_ANDROID_SDK_ROUTING_MISSING")
    require_tokens(lifecycle, ("uses_host_network_runtime", "network.prepare_runtime", "bind_runtime_process"), "ANDROID_SDK_NETWORK_BYPASS_MISSING")

    adb = read("crates/guest/src/tools/adb_runtime_tool.rs")
    require_tokens(adb, ("EMULATOR_SERIAL_PREFIX", '"emulator-"', "emulator_serial", "target_serial"), "ANDROID_EMULATOR_ADB_TARGET_MISSING")

    ui = read("apps/desktop/ui/app.js")
    desktop_controller = read("apps/desktop/src-tauri/src/controllers/desktop_controller.rs")
    desktop_service = read("apps/desktop/src-tauri/src/services/desktop_service.rs")
    require_tokens(ui, (
        'ANDROID_SDK_IMAGE_SUFFIX = "-sdk-x86_64"', "androidSdkImageId", "isAndroidSdkImage",
        "handleCreateAndroidProgressCancel", "handleCreateAndroidProgressLog", 'invoke("open_android_image_install_log"',
    ), "ANDROID_SDK_UI_MISSING")
    require("open_android_image_install_log" in desktop_controller, "ANDROID_INSTALL_LOG_CONTROLLER_MISSING")
    require("open_android_image_install_log" in desktop_service, "ANDROID_INSTALL_LOG_SERVICE_MISSING")

    engine_api = read("crates/engine-api/src/lib.rs")
    match = re.search(r"ENGINE_API_VERSION:\s*u16\s*=\s*(\d+)\s*;", engine_api)
    require(match is not None and int(match.group(1)) == 24, "ENGINE_API_VERSION_CHANGED")

    print("V0413_ANDROID_SDK_WINDOWS_PROVIDER_CONTRACT=PASS")


if __name__ == "__main__":
    main()
