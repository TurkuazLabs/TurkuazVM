# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0363_rust_android_ci_compile_contract.py
# 📌 Amac: Android CI Rust provider/distribution use bloklarina macro expression sizmasini fail-closed dogrular
# 📌 Modul - Python
# Version: 0.40.0
# Aciklama: Source Resolver ayrimi sonrasinda provider runtime URL formatlamasinin import syntaxina sizmasini engeller
# Bagimli Oldugu Katman: Tool | Service

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PROVIDER = ROOT / "crates/guest/src/tools/android_ci_source_provider_tool.rs"
VERIFY = ROOT / "scripts/verify_structure.ps1"


def collect_use_statements(text: str) -> list[str]:
    statements: list[str] = []
    current: list[str] = []
    collecting = False
    for raw_line in text.splitlines():
        stripped = raw_line.strip()
        if not collecting:
            if stripped.startswith("use ") or stripped.startswith("pub use "):
                current = [raw_line]
                collecting = ";" not in stripped
                if not collecting:
                    statements.append("\n".join(current))
                    current = []
            continue
        current.append(raw_line)
        if ";" in stripped:
            statements.append("\n".join(current))
            current = []
            collecting = False
    return statements


def main() -> int:
    provider = PROVIDER.read_text(encoding="utf-8")
    verify = VERIFY.read_text(encoding="utf-8")
    if "ANDROID_CI_RUST_IMPORT_SYNTAX_INVALID" not in verify:
        raise AssertionError("launcher pre-build gate missing Android CI Rust import syntax guard")
    violations: list[str] = []
    for rust_path in ROOT.rglob("*.rs"):
        if "target" in rust_path.parts:
            continue
        text = rust_path.read_text(encoding="utf-8")
        for statement in collect_use_statements(text):
            if "!" in statement:
                violations.append(f"{rust_path.relative_to(ROOT).as_posix()}: {statement.strip()}")
    if violations:
        raise AssertionError(f"macro/expression found inside Rust use statement: {violations}")
    for token in ('format!("{}/", base_url.trim_end_matches(\'/\'))', 'format!("{base_url}/builds/branches/{branch}/{BRANCH_STATUS_FILE}")'):
        if token not in provider:
            raise AssertionError(f"valid Android CI provider runtime URL formatting missing: {token}")
    print("V0363_RUST_ANDROID_CI_COMPILE_CONTRACT=PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
