# 📄 Dosya Yolu: /turkuazvm/tools/runtime_validation_probe_tool.py
# 📌 Amac: L harness icin host platform, mimari ve gercek runtime binary capability envanterini toplar
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: QEMU, virtiofsd, swtpm ve TurkuazVM guest adaptorlerinin varlik ve surum bilgisini shell kullanmadan probe eder
# Bagimli Oldugu Katman: Tool | Service

from __future__ import annotations

import platform
import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from tools.runtime_validation_config import RuntimeValidationConfig


class RuntimeValidationProbeError(RuntimeError):
    pass


@dataclass(frozen=True)
class BinaryProbe:
    name: str
    configured: str
    resolved: str | None
    available: bool
    version_text: str | None

    def as_dict(self) -> dict[str, Any]:
        return {
            "name": self.name,
            "configured": self.configured,
            "resolved": self.resolved,
            "available": self.available,
            "version_text": self.version_text,
        }


class RuntimeValidationProbeTool:
    def __init__(self, config: RuntimeValidationConfig) -> None:
        self.config = config
        self.cfg = config.validation

    def host(self) -> dict[str, Any]:
        system = platform.system().lower()
        platform_name = str(self.cfg["host"]["platforms"].get(system, system))
        machine = platform.machine().lower()
        architecture = self._architecture(machine)
        return {
            "platform": platform_name,
            "platform_raw": system,
            "machine": machine,
            "architecture": architecture,
            "python": platform.python_version(),
            "release": platform.release(),
        }

    def binaries(self, architecture: str) -> dict[str, dict[str, Any]]:
        configured = self.cfg["binaries"]
        qemu_system_map = configured["qemu_system"]
        result: dict[str, dict[str, Any]] = {}
        binary_map = {
            "qemu_img": str(configured["qemu_img"]),
            "qemu_system": str(qemu_system_map.get(architecture, "")),
            "virtiofsd": str(configured["virtiofsd"]),
            "swtpm": str(configured["swtpm"]),
            "guest_bridge_cli": str(configured["guest_bridge_cli"]),
            "secret_rewrap_cli": str(configured["secret_rewrap_cli"]),
        }
        for name, binary in binary_map.items():
            result[name] = self._probe(name, binary).as_dict()
        return result

    def _architecture(self, machine: str) -> str:
        aliases = self.cfg["host"]["architecture_aliases"]
        for canonical, values in aliases.items():
            if machine in {str(item).lower() for item in values}:
                return str(canonical)
        return machine

    def _probe(self, name: str, binary: str) -> BinaryProbe:
        if not binary:
            return BinaryProbe(name, binary, None, False, None)
        resolved = shutil.which(binary)
        if resolved is None:
            return BinaryProbe(name, binary, None, False, None)
        version = self._version(Path(resolved))
        return BinaryProbe(name, binary, resolved, True, version)

    def _version(self, binary: Path) -> str | None:
        timeout = int(self.cfg["command"]["probe_timeout_seconds"])
        max_output = int(self.cfg["command"]["max_output_bytes"])
        for flag in ("--version", "-version"):
            try:
                completed = subprocess.run(
                    (str(binary), flag),
                    capture_output=True,
                    text=True,
                    shell=False,
                    timeout=timeout,
                    check=False,
                )
            except (OSError, subprocess.TimeoutExpired):
                continue
            text = (completed.stdout or completed.stderr).strip()
            if text:
                return text[:max_output].splitlines()[0]
        return None
