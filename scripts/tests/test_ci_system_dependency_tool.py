# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_ci_system_dependency_tool.py
# 📌 Amac: CI sistem bagimlilik tool config planlama ve Windows PATH expansion davranisini regression ile kilitler
# 📌 Modul - Python
# Version: 1.0.1
# Aciklama: Gercek paket kurulumu yapmadan compiler/runtime profillerini, case-insensitive Windows ortam degiskenlerini ve fail-closed platform kontrolunu dogrular
# Bagimli Oldugu Katman: Tool

from __future__ import annotations

import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from tools.ci_system_dependency_tool import CiSystemDependencyError, CiSystemDependencyTool


def main() -> int:
    with tempfile.TemporaryDirectory() as temp_dir:
        github_path = Path(temp_dir) / "github-path.txt"
        environ = {
            "PROGRAMFILES": r"C:\Program Files",
            "GITHUB_PATH": str(github_path),
            "PATH": "",
        }
        tool = CiSystemDependencyTool(environ=environ)

        compiler = tool.plan("compiler-linux", host_platform="linux")
        assert compiler.commands[0][:3] == ("sudo", "apt-get", "update")
        assert "libwebkit2gtk-4.1-dev" in compiler.commands[1]
        assert compiler.required_binaries == ("pkg-config",)

        runtime_windows = tool.plan("runtime-windows", host_platform="windows")
        assert runtime_windows.commands[0][:3] == ("choco", "install", "qemu")
        assert str(runtime_windows.path_exports[0]).endswith("Program Files\\qemu")
        assert "%ProgramFiles%" not in str(runtime_windows.path_exports[0])
        assert runtime_windows.required_binaries == ("qemu-img.exe", "qemu-system-x86_64.exe")

        try:
            tool.plan("runtime-windows", host_platform="linux")
        except CiSystemDependencyError as exc:
            assert "CI_DEP_PLATFORM_MISMATCH" in str(exc)
        else:
            raise AssertionError("platform mismatch must fail closed")

    print("CI SYSTEM DEPENDENCY REGRESSION: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
