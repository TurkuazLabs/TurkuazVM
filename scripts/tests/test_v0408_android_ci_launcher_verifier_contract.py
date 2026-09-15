# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0408_android_ci_launcher_verifier_contract.py
# 📌 Amac: Android CI launcher verifier ile historical resolver regression kontratinin production parserdan sapmasini engeller
# 📌 Modul - Python
# Version: 0.40.8
# Aciklama: Kaldirilmis legacy parser helperinin provider icin tekrar zorunlu tutulmasini fail-closed kilitler; negatif dead-code guardlarini korur
# Bagimli Oldugu Katman: Tool | Service

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> None:
    verifier = read("scripts/verify_structure.ps1")
    historical = read("scripts/tests/test_v0400_android_source_resolver_contract.py")
    provider = read("crates/guest/src/tools/android_ci_source_provider_tool.rs")

    obsolete = "parse_" + "last_known_good_build"

    provider_gate = verifier.split(
        '$AndroidSourceProviderContent = Get-Content', 1
    )[1].split(
        'if ($AndroidSourceProviderContent.Contains("/view/{BUILD_INFO_FILE}"))', 1
    )[0]
    require(obsolete not in provider_gate, "ANDROID_LAUNCHER_PROVIDER_GATE_STALE_LEGACY_PARSER")

    historical_provider_gate = historical.split(
        'for token in ("status.json"', 1
    )[1].split(
        'require(token in provider', 1
    )[0]
    require(obsolete not in historical_provider_gate, "ANDROID_HISTORICAL_PROVIDER_GATE_STALE_LEGACY_PARSER")

    require(f"fn {obsolete}(" not in provider, "ANDROID_PROVIDER_LEGACY_PARSER_RETURNED")

    for required in (
        "parse_status_target_candidates",
        "parse_status_targets",
        "collect_status_targets",
    ):
        require(required in provider_gate, f"ANDROID_LAUNCHER_VERIFIER_PRODUCTION_PARSER_MISSING:{required}")
        require(required in historical_provider_gate, f"ANDROID_HISTORICAL_TEST_PRODUCTION_PARSER_MISSING:{required}")
        require(required in provider, f"ANDROID_PROVIDER_PRODUCTION_PARSER_MISSING:{required}")

    require(obsolete in verifier, "ANDROID_DISTRIBUTION_NEGATIVE_DEAD_CODE_GUARD_LOST")
    require(obsolete in historical, "ANDROID_HISTORICAL_NEGATIVE_DEAD_CODE_GUARD_LOST")

    print("V0408_ANDROID_CI_LAUNCHER_VERIFIER_CONTRACT=PASS")


if __name__ == "__main__":
    main()
