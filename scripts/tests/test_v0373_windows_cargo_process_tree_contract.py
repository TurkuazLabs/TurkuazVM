# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0373_windows_cargo_process_tree_contract.py
# 📌 Amac: Windows cargo build tamamlandiktan sonra launcher'in process agacinda takilip Desktop baslatma adimina gecememesi regresyonunu engeller
# 📌 Modul - Python
# Version: 0.37.3
# Aciklama: Checked native process beklemesini tek process ile sinirlar ve Desktop binary'nin ilk build sonrasi dogrudan baslatildigini dogrular
# Bagimli Oldugu Katman: Tool | View

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def main() -> int:
    cargo = read("Cargo.toml")
    match = re.search(r'(?ms)^\[workspace\.package\].*?^version\s*=\s*"([^"]+)"', cargo)
    assert match, "workspace version missing"
    assert tuple(map(int, match.group(1).split("."))) >= (0, 37, 3)

    tool = read("tools/windows_native_process_tool.ps1")
    checked = tool.split("function Invoke-TurkuazNativeCapture", 1)[0]
    assert "System.Diagnostics.ProcessStartInfo" in checked
    assert "$Process.WaitForExit()" in checked
    assert "$Process.ExitCode" in checked
    assert "Start-Process" not in checked, "checked native command must not use Start-Process -Wait"

    launcher = read("scripts/start_turkuazvm.ps1")
    required = (
        '"-p", "turkuazvm-engine"',
        '"-p", "turkuazvm-display"',
        '"-p", "turkuazvm-desktop"',
        'target/debug/turkuazvm-desktop.exe',
        'Write-Host "TurkuazVM Desktop baslatiliyor..."',
        'Write-Host "DESKTOP_STARTED pid=$($DesktopProcess.Id)"',
        'DESKTOP_PROCESS_EARLY_EXIT',
    )
    for token in required:
        assert token in launcher, token

    assert '@("run", "-p", "turkuazvm-desktop")' not in launcher
    assert 'Invoke-TurkuazNativeChecked -FilePath "cargo" -Arguments @("run"' not in launcher

    print("V0373_WINDOWS_CARGO_PROCESS_TREE_CONTRACT=PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
