# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0361_launcher_android_schema_gate_contract.py
# 📌 Amac: Launcher structure gate ile Android image repository schema migration kontratinin birlikte calistigini dogrular
# 📌 Modul - Python
# Version: 0.36.1
# Aciklama: v0.36.0 image schema 3 gecisinin eski launcher schema 2 sabitine takilip Desktop acilisini engellemesini regression olarak onler
# Bagimli Oldugu Katman: Repo | Tool

from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[2]
VERIFY = ROOT / "scripts" / "verify_structure.ps1"
REPOSITORY = ROOT / "crates" / "repositories" / "src" / "repositories" / "yaml_android_image_repository.rs"


def main() -> int:
    verify = VERIFY.read_text(encoding="utf-8")
    repository = REPOSITORY.read_text(encoding="utf-8")

    schema_match = re.search(r"const\s+IMAGE_SCHEMA_VERSION:\s*u16\s*=\s*(\d+)\s*;", repository)
    if schema_match is None:
        raise AssertionError("current Android image schema constant missing")
    if int(schema_match.group(1)) < 3:
        raise AssertionError("Android image schema must be v3 or newer")

    required_repo_tokens = (
        "const LEGACY_IMAGE_SCHEMA_V2: u16 = 2;",
        "const LEGACY_IMAGE_SCHEMA_VERSION: u16 = 1;",
        "const ASSIGNMENT_SCHEMA_VERSION: u16 = 2;",
        "const LEGACY_ASSIGNMENT_SCHEMA_VERSION: u16 = 1;",
    )
    missing_repo = [token for token in required_repo_tokens if token not in repository]
    if missing_repo:
        raise AssertionError(f"Android image migration constants missing: {missing_repo}")

    forbidden_verify = '"IMAGE_SCHEMA_VERSION: u16 = 2"'
    if forbidden_verify in verify:
        raise AssertionError("launcher still hardcodes obsolete Android image schema v2")

    required_verify_tokens = (
        "$ImageSchemaVersionMatch = [regex]::Match",
        "ANDROID_IMAGE_REPOSITORY_SCHEMA_VERSION_MISSING",
        "ANDROID_IMAGE_REPOSITORY_SCHEMA_VERSION_UNSUPPORTED",
        '"LEGACY_IMAGE_SCHEMA_V2: u16 = 2"',
        '"LEGACY_IMAGE_SCHEMA_VERSION: u16 = 1"',
    )
    missing_verify = [token for token in required_verify_tokens if token not in verify]
    if missing_verify:
        raise AssertionError(f"launcher Android schema verifier missing: {missing_verify}")

    print("V0361_LAUNCHER_ANDROID_SCHEMA_GATE_CONTRACT=PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
