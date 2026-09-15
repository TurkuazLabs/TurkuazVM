# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0414_android_emulator_port_config_wiring_contract.py
# 📌 Amac: Android Emulator console ve ADB port araliklarinin magic sayi yerine merkezi configten runtime'a tasindigini fail-closed dogrular
# 📌 Modul - Python
# Version: 0.40.14
# Aciklama: v0.40.13 Windows Cargo dead-code blokajini ve runtime port-range drift riskini tekrarini engeller
# Bagimli Oldugu Katman: Service | Tool | Config

from pathlib import Path

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
    source_config = yaml.safe_load(read("config/download-sources.yml"))
    main_config = yaml.safe_load(read("config/turkuazvm.yml"))
    sdk = source_config["sources"]["android_sdk"]
    adb = main_config["android"]["adb"]

    console_min = int(sdk["emulator_console_port_min"])
    console_max = int(sdk["emulator_console_port_max"])
    require(console_min % 2 == 0 and console_max % 2 == 0, "ANDROID_EMULATOR_CONSOLE_RANGE_NOT_EVEN")
    require(int(adb["host_port_min"]) == console_min + 1, "ANDROID_EMULATOR_ADB_MIN_NOT_PAIRED")
    require(int(adb["host_port_max"]) == console_max + 1, "ANDROID_EMULATOR_ADB_MAX_NOT_PAIRED")

    engine_config = read("apps/engine/src/config/engine_config.rs")
    image_service = read("apps/engine/src/services/android_image_application_service.rs")
    android_service = read("apps/engine/src/services/android_application_service.rs")
    runtime_media = read("crates/guest/src/tools/android_runtime_media_tool.rs")

    require_tokens(engine_config, (
        "pub emulator_console_port_min: u16",
        "pub emulator_console_port_max: u16",
        "emulator_console_port_min: android_sdk_console_min",
        "emulator_console_port_max: android_sdk_console_max",
    ), "ENGINE_ANDROID_SDK_PORT_CONFIG_MISSING")
    require_tokens(image_service, (
        "let emulator_console_port_min = settings.sdk.emulator_console_port_min;",
        "let emulator_console_port_max = settings.sdk.emulator_console_port_max;",
        "emulator_console_port_min,",
        "emulator_console_port_max,",
    ), "ANDROID_IMAGE_SERVICE_PORT_WIRING_MISSING")
    require_tokens(runtime_media, (
        "pub emulator_console_port_min: u16",
        "pub emulator_console_port_max: u16",
        "self.settings.emulator_console_port_min",
        "self.settings.emulator_console_port_max",
        "adb_port.checked_sub(1)",
    ), "ANDROID_RUNTIME_MEDIA_PORT_CONFIG_MISSING")
    require_tokens(android_service, (
        "self.settings.adb_host_port_min",
        "self.settings.adb_host_port_max",
        "self.is_windows_emulator_adb_port",
    ), "ANDROID_APPLICATION_ADB_CONFIG_MISSING")

    require("(5555..=5681)" not in android_service, "ANDROID_APPLICATION_MAGIC_PORT_RANGE_RETURNED")
    require("(5555..=5681)" not in runtime_media, "ANDROID_RUNTIME_MEDIA_MAGIC_PORT_RANGE_RETURNED")
    require("allow(dead_code)" not in engine_config, "ENGINE_DEAD_CODE_ALLOW_FORBIDDEN")

    print("V0414_ANDROID_EMULATOR_PORT_CONFIG_WIRING_CONTRACT=PASS")


if __name__ == "__main__":
    main()
