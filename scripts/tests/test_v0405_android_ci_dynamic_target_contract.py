# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0405_android_ci_dynamic_target_contract.py
# 📌 Amac: Eski Android CI branchlerinde exact target adi degisse bile Cuttlefish x86_64 target discovery davranisini fail-closed kilitler
# 📌 Modul - Python
# Version: 0.40.5
# Aciklama: Provider'in status.json dosyasini branch basina tek kez okuyup exact hedeflerden sonra dinamik historical target adaylarini denedigini ve 12L branch fallbacklerini korudugunu dogrular
# Bagimli Oldugu Katman: Tool | Config | Service

from pathlib import Path
import yaml

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> None:
    provider = read("crates/guest/src/tools/android_ci_source_provider_tool.rs")
    config = yaml.safe_load(read("config/download-sources.yml"))

    for token in (
        "parse_status_target_candidates",
        "collect_status_targets",
        "normalize_target_id",
        "target_discovery_rank",
        "TARGET_DISCOVERY_REQUIRED_TOKENS",
        "TARGET_DISCOVERY_EXCLUDED_TOKENS",
        "Cuttlefish x86_64 userdebug target bulunamadi",
        "status_targets=",
        "resolve_known_build",
        "release_values_match",
    ):
        require(token in provider, f"ANDROID_DYNAMIC_TARGET_DISCOVERY_MISSING:{token}")

    require(
        'let status_url = format!("{base_url}/builds/branches/{branch}/{BRANCH_STATUS_FILE}");' in provider,
        "ANDROID_BRANCH_STATUS_FETCH_MISSING",
    )
    require(
        "for target in &request.target_candidates" not in provider,
        "ANDROID_STATUS_FETCH_STILL_EXACT_TARGET_NESTED",
    )
    require(
        'normalized.contains("phone-userdebug")' in provider,
        "ANDROID_HISTORICAL_PHONE_TARGET_RANK_MISSING",
    )
    require(
        'matches!((expected, actual), ("12L", "12") | ("12L", "12.1"))' in provider,
        "ANDROID_12L_RELEASE_ALIAS_MISSING",
    )

    branch_hints = config["sources"]["android_ci"]["channels"]["12L"]["branch_hints"]
    for branch in (
        "aosp-android12L-gsi",
        "aosp-android12L-release",
        "aosp-android12L-platform-release",
        "aosp-android12L-dev",
    ):
        require(branch in branch_hints, f"ANDROID_12L_BRANCH_FALLBACK_MISSING:{branch}")

    print("V0405_ANDROID_CI_DYNAMIC_TARGET_CONTRACT=PASS")


if __name__ == "__main__":
    main()
