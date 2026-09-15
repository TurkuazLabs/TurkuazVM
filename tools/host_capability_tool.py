# 📄 Dosya Yolu: /turkuazvm/tools/host_capability_tool.py
# 📌 Amac: Host donanim ve runtime binary capability envanterini side-effect olmadan toplar
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Hardware rebind ve backend secimi icin platform, CPU, RAM ve executable capability bilgisi uretir
# Bagimli Oldugu Katman: Tool | Service

from __future__ import annotations

import hashlib
import os
import platform
import shutil
from dataclasses import dataclass, asdict
from typing import Any

from tools.runtime_config import RuntimeConfig


@dataclass(frozen=True)
class HostInventory:
    platform: str
    machine: str
    architecture: str
    cpu_count: int
    memory_bytes: int
    binaries: dict[str, bool]
    fingerprint: str

    def as_dict(self) -> dict[str, Any]:
        return asdict(self)


class HostCapabilityTool:
    def __init__(self, config: RuntimeConfig) -> None:
        self.config = config

    def probe(self) -> HostInventory:
        system = platform.system().lower()
        platform_names = self.config.runtime["network"]["platform_names"]
        normalized = str(platform_names.get(system, system))
        machine = platform.machine().lower()
        architecture = platform.architecture()[0].lower()
        cpu_count = int(os.cpu_count() or 1)
        memory_bytes = self._memory_bytes()
        binary_cfg = self.config.runtime["binaries"]
        binaries = {name: shutil.which(str(executable)) is not None for name, executable in binary_cfg.items()}
        raw = "|".join((normalized, machine, architecture, str(cpu_count), str(memory_bytes)))
        fingerprint = hashlib.sha256(raw.encode("utf-8")).hexdigest()
        return HostInventory(normalized, machine, architecture, cpu_count, memory_bytes, binaries, fingerprint)

    @staticmethod
    def _memory_bytes() -> int:
        if hasattr(os, "sysconf"):
            try:
                pages = int(os.sysconf("SC_PHYS_PAGES"))
                page_size = int(os.sysconf("SC_PAGE_SIZE"))
                if pages > 0 and page_size > 0:
                    return pages * page_size
            except (ValueError, OSError, AttributeError):
                pass
        return 0
