# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0409_android_ci_numeric_build_compile_contract.py
# 📌 Amac: Android CI production parserinda numeric build-id helperinin cagrilip tanimli olmasini fail-closed kilitler
# 📌 Modul - Python
# Version: 0.40.9
# Aciklama: collect_status_targets icindeki unresolved numeric_build_id compile regresyonunun geri donmesini engeller
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
    verifier = read("scripts/verify_structure.ps1")

    require("numeric_build_id(value)" in provider, "ANDROID_CI_NUMERIC_BUILD_CALL_MISSING")
    require("fn numeric_build_id(value: &serde_json::Value) -> Option<String>" in provider, "ANDROID_CI_NUMERIC_BUILD_FUNCTION_MISSING")
    require("BUILD_ID_FIELDS" in provider, "ANDROID_CI_BUILD_ID_FIELDS_MISSING")
    require("candidate.bytes().all(|byte| byte.is_ascii_digit())" in provider, "ANDROID_CI_NUMERIC_BUILD_VALIDATION_MISSING")
    require("numeric_build_id" in verifier, "ANDROID_CI_NUMERIC_BUILD_LAUNCHER_GATE_MISSING")

    collect_block = provider.split("fn collect_status_targets(", 1)[1].split("fn numeric_build_id(", 1)[0]
    require("numeric_build_id(value)" in collect_block, "ANDROID_CI_COLLECT_NOT_USING_NUMERIC_BUILD_HELPER")

    print("V0409_ANDROID_CI_NUMERIC_BUILD_COMPILE_CONTRACT=PASS")


if __name__ == "__main__":
    main()
