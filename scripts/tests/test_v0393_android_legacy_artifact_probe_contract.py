# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0393_android_legacy_artifact_probe_contract.py
# 📌 Amac: Android 11 legacy range-GET artifact probe ve device bootloader fallback davranisini v0.40 Provider Tool icinde kilitler
# 📌 Modul - Python
# Version: 0.40.0
# Aciklama: HEAD false-negative riskinin geri gelmesini engeller ve fallback yetkisini yalniz Android 11 resolver policy kanalinda tutar
# Bagimli Oldugu Katman: Service | Tool | Config

from pathlib import Path
import yaml

ROOT = Path(__file__).resolve().parents[2]
CONFIG = yaml.safe_load((ROOT / "config/download-sources.yml").read_text(encoding="utf-8"))
PROVIDER = (ROOT / "crates/guest/src/tools/android_ci_source_provider_tool.rs").read_text(encoding="utf-8")
HTTP = (ROOT / "crates/guest/src/tools/http_download_tool.rs").read_text(encoding="utf-8")
DIST = (ROOT / "crates/guest/src/tools/android_ci_distribution_tool.rs").read_text(encoding="utf-8")
ENGINE_CONFIG = (ROOT / "apps/engine/src/config/engine_config.rs").read_text(encoding="utf-8")
ENGINE_SERVICE = (ROOT / "apps/engine/src/services/android_image_application_service.rs").read_text(encoding="utf-8")


def main() -> None:
    assert CONFIG["schema_version"] == 5, "DOWNLOAD_SOURCES_SCHEMA_NOT_V5"
    channels = CONFIG["sources"]["android_ci"]["channels"]
    assert channels["11"]["allow_device_bootloader_fallback"] is True, "ANDROID11_DEVICE_BOOTLOADER_FALLBACK_DISABLED"
    for release in ("17", "16", "15", "14", "13", "12L", "12"):
        assert channels[release]["allow_device_bootloader_fallback"] is False, f"ANDROID_{release}_FALLBACK_TOO_BROAD"
    probe_block = PROVIDER[PROVIDER.index("fn artifact_exists("):PROVIDER.index("fn resolve_candidate(")]
    assert '"--head"' not in probe_block, "ANDROID_ARTIFACT_HEAD_PROBE_RETURNED"
    for token in ("ARTIFACT_PROBE_RANGE", "ARTIFACT_PROBE_MAX_FILESIZE_BYTES", "probe_range", "allow_device_bootloader_fallback", "host_package_available"):
        assert token in PROVIDER, f"ANDROID_PROVIDER_LEGACY_PROBE_MISSING:{token}"
    for token in ('"--range"', '"--max-filesize"', '"--write-out"'):
        assert token in HTTP, f"ANDROID_SHARED_HTTP_PROBE_MISSING:{token}"
    for token in ("copy_x86_64_bootloader_from_device", "bootloader_source", "host_package_available", "allow_device_bootloader_fallback"):
        assert token in DIST, f"ANDROID_DISTRIBUTION_LEGACY_FALLBACK_MISSING:{token}"
    assert "allow_device_bootloader_fallback: bool" in ENGINE_CONFIG, "ENGINE_CONFIG_FALLBACK_FIELD_MISSING"
    assert "channel.allow_device_bootloader_fallback" in ENGINE_SERVICE, "ENGINE_SERVICE_FALLBACK_WIRING_MISSING"
    print("V0393_ANDROID_LEGACY_ARTIFACT_PROBE_CONTRACT=PASS")


if __name__ == "__main__":
    main()
