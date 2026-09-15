# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0407_android_ci_dead_code_contract.py
# 📌 Amac: Android CI resolver normal Rust buildinde dead-code deny regresyonunu fail-closed kilitler
# 📌 Modul - Python
# Version: 0.40.7
# Aciklama: Duplicate test-only legacy parser helperlarinin geri donmesini engeller ve testlerin production parser yolunu kullandigini dogrular
# Bagimli Oldugu Katman: Tool | Service

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> None:
    provider = read("crates/guest/src/tools/android_ci_source_provider_tool.rs")
    workspace = read("Cargo.toml")

    require('dead_code = "deny"' in workspace, "WORKSPACE_DEAD_CODE_DENY_MISSING")

    for obsolete in (
        "NESTED_BUILD_FIELDS",
        "fn parse_last_known_good_build(",
        "fn parse_target_collection(",
        "fn target_entry_matches(",
        "fn parse_build_from_target_entry(",
        "fn find_legacy_target_build(",
    ):
        require(obsolete not in provider, f"ANDROID_CI_OBSOLETE_DEAD_CODE_PRESENT:{obsolete}")

    for required in (
        "fn parse_status_target_candidates(",
        "fn parse_status_targets(",
        "fn collect_status_targets(",
        "production_parser_handles_modern_and_legacy_status_shapes",
        'parse_status_target_candidates(modern, "aosp-android17-release"',
        'parse_status_target_candidates(legacy, "aosp-android11-gsi"',
    ):
        require(required in provider, f"ANDROID_CI_PRODUCTION_PARSER_CONTRACT_MISSING:{required}")

    resolve_block = provider.split("impl AndroidDistributionSourceProviderPort", 1)[1]
    require(
        "parse_status_target_candidates(" in resolve_block,
        "ANDROID_CI_RESOLVE_NOT_USING_PRODUCTION_STATUS_PARSER",
    )

    print("V0407_ANDROID_CI_DEAD_CODE_CONTRACT=PASS")


if __name__ == "__main__":
    main()
