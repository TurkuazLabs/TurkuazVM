# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0372_header_path_contract.py
# 📌 Amac: Windows verify_structure header-path hatasinin tum teknik dosyalarda geri donmesini engeller
# 📌 Modul - Python
# Version: 0.37.2
# Aciklama: Release kapsamindaki tum teknik metin dosyalarinda canonical /turkuazvm path ve zorunlu header tokenlarini dogrular
# Bagimli Oldugu Katman: Tool | View

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TEXT_EXTENSIONS = {
    ".rs", ".toml", ".md", ".ps1", ".yml", ".yaml", ".html", ".css", ".js",
    ".json5", ".sh", ".service", ".example", ".gitignore", ".cmd", ".kt", ".xml",
    ".bp", ".mk", ".py",
}
REQUIRED = ("Dosya Yolu:", "Amac:", "Modul -", "Version:", "Aciklama:", "Bagimli Oldugu Katman:")

def main() -> int:
    checked = 0
    for path in ROOT.rglob("*"):
        if not path.is_file():
            continue
        relative = path.relative_to(ROOT)
        if relative.parts and relative.parts[0] in {"data", "target"}:
            continue
        if path.suffix not in TEXT_EXTENSIONS and path.name != ".gitignore":
            continue
        text = path.read_text(encoding="utf-8")
        header = "\n".join(text.splitlines()[:7])
        expected_path = f"/turkuazvm/{relative.as_posix()}"
        assert expected_path in header, f"HEADER_PATH_INVALID: /{relative.as_posix()}"
        for token in REQUIRED:
            assert token in header, f"HEADER_TOKEN_MISSING: /{relative.as_posix()} -> {token}"
        checked += 1
    assert checked >= 400, checked
    print(f"V0372_HEADER_PATH_CONTRACT=PASS files={checked}")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
