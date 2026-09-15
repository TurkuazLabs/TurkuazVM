# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0379_android10_legacy_ci_status_contract.py
# 📌 Amac: Android 10 SDK politikasini ve legacy Android CI status parser uyumlulugunu dogrular
# 📌 Modul - Python
# Version: 0.40.13
# Aciklama: Windows Android 10 icin SDK provider zorunlulugunu ve legacy CI production parserinin korunmasini test eder
# Bagimli Oldugu Katman: Tool | Service | Config

from pathlib import Path
import yaml

ROOT = Path(__file__).resolve().parents[2]
PROVIDER = (ROOT / "crates/guest/src/tools/android_ci_source_provider_tool.rs").read_text(encoding="utf-8")
CONFIG = yaml.safe_load((ROOT / "config/download-sources.yml").read_text(encoding="utf-8"))


def main() -> None:
    for token in (
        "fn parse_status_target_candidates(",
        "fn parse_status_targets(",
        "fn collect_status_targets(",
        "fn numeric_build_id(",
        "const TARGET_ID_FIELDS:",
        "const BUILD_ID_FIELDS:",
        "production_parser_handles_modern_and_legacy_status_shapes",
    ):
        assert token in PROVIDER, f"PRODUCTION_STATUS_PARSER_MISSING:{token}"
    for obsolete in (
        "const NESTED_BUILD_FIELDS:",
        "fn parse_last_known_good_build(",
        "fn parse_target_collection(",
        "fn target_entry_matches(",
        "fn parse_build_from_target_entry(",
        "fn find_legacy_target_build(",
    ):
        assert obsolete not in PROVIDER, f"OBSOLETE_STATUS_PARSER_RETURNED:{obsolete}"

    assert CONFIG["sources"]["android_release_policy"]["10"] == "android_sdk", "ANDROID10_WINDOWS_SDK_POLICY_MISSING"
    assert CONFIG["sources"]["android_sdk"]["api_levels"]["10"] == 29, "ANDROID10_API29_MAPPING_MISSING"
    assert "10" not in CONFIG["sources"]["android_ci"]["channels"], "ANDROID10_DEAD_CI_CHANNEL_RETURNED"
    assert "does not contain targets array" not in PROVIDER, "LEGACY_ARRAY_ONLY_PARSER_RETURNED"
    print("V0379_ANDROID10_LEGACY_CI_STATUS_CONTRACT=PASS")


if __name__ == "__main__":
    main()
