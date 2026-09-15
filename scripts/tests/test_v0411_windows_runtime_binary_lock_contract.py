# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0411_windows_runtime_binary_lock_contract.py
# 📌 Amac: Calisan TurkuazVM runtime exe dosyalarinin Windows Cargo buildini os error 5 ile kilitlemesini engeller
# 📌 Modul - Python
# Version: 0.40.11
# Aciklama: Launcher exact executable path kontrolu, already-running Desktop davranisi ve stale Engine/Display cleanup kontratini dogrular
# Bagimli Oldugu Katman: Tool | View

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> None:
    launcher = read("scripts/start_turkuazvm.ps1")
    native_tool = read("tools/windows_native_process_tool.ps1")

    for token in (
        "Get-TurkuazNativeProcessesByExecutablePath",
        "Stop-TurkuazNativeProcessesByExecutablePath",
        "Wait-TurkuazNativeFileRelease",
        "RUNTIME_PROCESS_STOP_TIMEOUT",
        "RUNTIME_BINARY_LOCKED",
    ):
        require(token in native_tool, f"WINDOWS_RUNTIME_LOCK_TOOL_MISSING: {token}")

    for token in (
        "DESKTOP_ALREADY_RUNNING",
        "target/debug/turkuazvm-desktop.exe",
        "target/debug/turkuazvm-engine.exe",
        "target/debug/turkuazvm-display.exe",
        "Stop-TurkuazNativeProcessesByExecutablePath",
        "Wait-TurkuazNativeFileRelease",
    ):
        require(token in launcher, f"WINDOWS_RUNTIME_LOCK_LAUNCHER_MISSING: {token}")

    require(
        launcher.index("DESKTOP_ALREADY_RUNNING") < launcher.index('Invoke-TurkuazNativeChecked -FilePath "cargo"'),
        "DESKTOP_ALREADY_RUNNING_CHECK_MUST_PRECEDE_CARGO_BUILD",
    )
    require(
        launcher.index("Stop-TurkuazNativeProcessesByExecutablePath")
        < launcher.index('Invoke-TurkuazNativeChecked -FilePath "cargo"'),
        "STALE_RUNTIME_CLEANUP_MUST_PRECEDE_CARGO_BUILD",
    )

    print("V0411_WINDOWS_RUNTIME_BINARY_LOCK_CONTRACT=PASS")


if __name__ == "__main__":
    main()
