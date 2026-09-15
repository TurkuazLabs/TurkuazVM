# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0371_windows_native_process_contract.py
# 📌 Amac: Windows PowerShell 5.1 native stderr akisinin launcher tarafindan fatal PowerShell hatasi sanilmasini regresyona karsi korur
# 📌 Modul - Python
# Version: 0.37.1
# Aciklama: rustup/rustc/cargo/QEMU ve WinGet native komutlarinin exit-code tabanli Tool uzerinden calistigini ve exact Rust toolchain policy'sinin configten okundugunu dogrular
# Bagimli Oldugu Katman: Tool

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def require(path: str, *tokens: str) -> None:
    text = read(path)
    missing = [token for token in tokens if token not in text]
    if missing:
        raise AssertionError(f"{path}: missing tokens: {missing}")


def main() -> int:
    cargo = read("Cargo.toml")
    version_match = re.search(
        r'(?ms)^\[workspace\.package\].*?^version\s*=\s*"([^"]+)"', cargo
    )
    if not version_match:
        raise AssertionError("workspace version missing")
    assert tuple(map(int, version_match.group(1).split("."))) >= (0, 37, 1)

    require(
        "tools/windows_native_process_tool.ps1",
        "function Invoke-TurkuazNativeChecked",
        "function Invoke-TurkuazNativeCapture",
        "Start-Process",
        "-RedirectStandardOutput",
        "-RedirectStandardError",
        "$Process.ExitCode",
    )

    launcher = read("scripts/start_turkuazvm.ps1")
    require(
        "scripts/start_turkuazvm.ps1",
        "$ErrorActionPreference = \"Stop\"",
        "$NativeProcessToolPath",
        ". $NativeProcessToolPath",
        "function Get-RustToolchainPolicy",
        "$RequiredRustToolchain = $RustToolchainPolicy.Channel",
        "function Initialize-RequiredRustToolchain",
        'Invoke-TurkuazNativeCapture -FilePath $RustcPath -Arguments @("--version")',
        'Invoke-TurkuazNativeCapture -FilePath $RustcPath -Arguments @("-vV")',
        'Invoke-TurkuazNativeChecked -FilePath $RustupPath',
        'Invoke-TurkuazNativeChecked -FilePath "cargo"',
    )
    forbidden_launcher_tokens = (
        "& $RustcPath --version",
        "& $RustcPath -vV",
        "Invoke-NativeChecked -FilePath \"cargo\"",
        'Rust 1.98.0+ kurulmali.',
    )
    for token in forbidden_launcher_tokens:
        assert token not in launcher, f"legacy native invocation returned: {token}"

    require(
        "scripts/bootstrap_windows_dependencies.ps1",
        "Invoke-TurkuazNativeChecked",
        "$Winget.Source",
    )
    bootstrap = read("scripts/bootstrap_windows_dependencies.ps1")
    assert "& $Winget.Source install" not in bootstrap

    require("rust-toolchain.toml", 'channel = "1.98.0"')
    assert 'rust-version = "1.98.0"' in cargo

    print("V0371_WINDOWS_NATIVE_PROCESS_CONTRACT=PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
