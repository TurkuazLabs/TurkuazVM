# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_windows_distribution_contract.py
# 📌 Amac: Windows NSIS/portable dagitim akisinin temel kontratini statik olarak dogrular
# 📌 Modul - Python Test
# Version: 0.41.4
# Aciklama: Tauri NSIS config, Windows distribution workflow ve packaging Tool arasindaki zorunlu baglantilari kontrol eder
# Bagimli Oldugu Katman: Tool | CI/CD | View

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def require(path: str, needle: str) -> None:
    content = (ROOT / path).read_text(encoding="utf-8")
    if needle not in content:
        raise AssertionError(f"{path}: missing {needle!r}")


def main() -> None:
    require("apps/desktop/src-tauri/tauri.windows.conf.json5", '"targets": ["nsis"]')
    require("apps/desktop/src-tauri/tauri.windows.conf.json5", '"installMode": "currentUser"')
    require("apps/desktop/src-tauri/tauri.windows.conf.json5", '"Turkish"')
    require("apps/desktop/src-tauri/tauri.windows.conf.json5", '"English"')
    require("apps/desktop/src-tauri/tauri.windows.conf.json5", '"displayLanguageSelector": true')
    require("apps/desktop/src-tauri/tauri.windows.conf.json5", '"distribution/windows/stage/": ""')

    require("scripts/package_windows_distribution.ps1", "turkuazvm-engine.exe")
    require("scripts/package_windows_distribution.ps1", "turkuazvm-display.exe")
    require("scripts/package_windows_distribution.ps1", "TurkuazVM.exe")
    require("scripts/package_windows_distribution.ps1", "Portable.zip")
    require("scripts/package_windows_distribution.ps1", "SHA256SUMS.txt")
    require("scripts/package_windows_distribution.ps1", '"tauri", "build"')

    require(".github/workflows/windows-distribution.yml", "windows-latest")
    require(".github/workflows/windows-distribution.yml", "package_windows_distribution.ps1")
    require(".github/workflows/windows-distribution.yml", "actions/upload-artifact@v4")
    require(".github/workflows/windows-distribution.yml", "gh release upload")
    require(".github/workflows/windows-distribution.yml", "refs/tags/v")

    print("WINDOWS_DISTRIBUTION_CONTRACT=PASS")


if __name__ == "__main__":
    main()
