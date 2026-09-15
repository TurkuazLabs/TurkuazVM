# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0412_windows_ps51_process_collection_contract.py
# 📌 Amac: Windows PowerShell 5.1 native process taramasinda generic-list binder regresyonunu engeller
# 📌 Modul - Python
# Version: 0.40.12
# Aciklama: Exact-path process discovery fonksiyonunun PS5.1-safe plain array kullanmasini ve stale generic List[object] yolunun geri donmemesini dogrular
# Bagimli Oldugu Katman: Tool | View

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> None:
    native_tool = read("tools/windows_native_process_tool.ps1")
    launcher = read("scripts/start_turkuazvm.ps1")

    require(
        "$ProcessMatches = @()" in native_tool,
        "PS51_PROCESS_COLLECTION_ARRAY_MISSING",
    )
    require(
        "$ProcessMatches += [pscustomobject]@{" in native_tool,
        "PS51_PROCESS_COLLECTION_APPEND_MISSING",
    )
    require(
        "return $ProcessMatches" in native_tool,
        "PS51_PROCESS_COLLECTION_RETURN_MISSING",
    )
    require(
        "New-Object System.Collections.Generic.List[object]" not in native_tool,
        "PS51_UNSAFE_GENERIC_PROCESS_LIST_RETURNED",
    )
    require(
        "$Matches.Add([pscustomobject]" not in native_tool,
        "PS51_AUTOMATIC_MATCHES_VARIABLE_REUSED",
    )
    require(
        "Get-TurkuazNativeProcessesByExecutablePath -ExecutablePaths @($DesktopExecutable)" in launcher,
        "DESKTOP_EXACT_PATH_DISCOVERY_MISSING",
    )

    print("V0412_WINDOWS_PS51_PROCESS_COLLECTION_CONTRACT=PASS")


if __name__ == "__main__":
    main()
