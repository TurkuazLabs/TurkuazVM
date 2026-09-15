# 📄 Dosya Yolu: /turkuazvm/tools/tvgb_transfer_tool.py
# 📌 Amac: Desktop drag-drop dosyalarini mevcut authenticated TVGB transfer adaptorune integrity parametreleriyle iletir
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Capability negotiation, SHA-256, resume-required ve overwrite-reject kosullarini harici Guest Bridge CLI uzerinde zorunlu uygular
# Bagimli Oldugu Katman: Tool | Service

from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path
from typing import Any

from tools.command_runner import CommandError, CommandRunner
from tools.runtime_config import RuntimeConfig


class TvgbTransferError(RuntimeError):
    pass


class TvgbTransferTool:
    def __init__(self, config: RuntimeConfig, runner: CommandRunner | None = None) -> None:
        self.config = config
        self.runner = runner or CommandRunner(config)
        self.cfg = config.runtime["drag_drop"]
        self.binary = str(config.runtime["binaries"]["guest_bridge_cli"])

    def capabilities(self) -> set[str]:
        operation_flag = str(self.cfg["cli_flags"]["operation"])
        operation = str(self.cfg["capabilities_operation"])
        try:
            result = self.runner.run((self.binary, operation_flag, operation))
        except CommandError as exc:
            raise TvgbTransferError(f"{self.config.codes['drag_drop_rejected']} {exc}") from exc
        try:
            payload = json.loads(result.stdout)
        except json.JSONDecodeError as exc:
            raise TvgbTransferError(f"{self.config.codes['drag_drop_rejected']} capability response invalid JSON") from exc
        values = payload.get("capabilities") if isinstance(payload, dict) else None
        if not isinstance(values, list) or not all(isinstance(item, str) for item in values):
            raise TvgbTransferError(f"{self.config.codes['drag_drop_rejected']} capability response invalid")
        return set(values)

    def send_file(self, source: Path, guest_destination: str) -> dict[str, Any]:
        self._validate_destination(guest_destination)
        if not source.is_file():
            raise TvgbTransferError(f"{self.config.codes['drag_drop_rejected']} source file missing")
        required = set(self.cfg["required_capabilities"])
        available = self.capabilities()
        if not required.issubset(available):
            raise TvgbTransferError(f"{self.config.codes['capability_missing']} TVGB capability missing: {sorted(required - available)}")
        digest = self._sha256(source)
        flags = self.cfg["cli_flags"]
        argv = (
            self.binary,
            str(flags["operation"]),
            str(self.cfg["send_operation"]),
            str(flags["source"]),
            str(source.resolve()),
            str(flags["destination"]),
            guest_destination,
            str(flags["sha256"]),
            digest,
            str(flags["resume"]),
            str(self.cfg["resume_value"]),
            str(flags["overwrite"]),
            str(self.cfg["overwrite_value"]),
        )
        try:
            result = self.runner.run(argv, timeout=int(self.config.runtime["command"]["long_timeout_seconds"]))
        except CommandError as exc:
            raise TvgbTransferError(f"{self.config.codes['drag_drop_rejected']} {exc}") from exc
        try:
            payload = json.loads(result.stdout)
        except json.JSONDecodeError as exc:
            raise TvgbTransferError(f"{self.config.codes['drag_drop_rejected']} transfer response invalid JSON") from exc
        if not isinstance(payload, dict) or payload.get("status") != "PASS" or payload.get("sha256") != digest:
            raise TvgbTransferError(f"{self.config.codes['drag_drop_rejected']} transfer integrity not confirmed")
        return payload


    def _validate_destination(self, destination: str) -> None:
        max_length = int(self.cfg["max_relative_destination_length"])
        if not destination or len(destination) > max_length or "\x00" in destination:
            raise TvgbTransferError(f"{self.config.codes['drag_drop_rejected']} guest destination invalid")
        normalized = destination.replace("\\", "/")
        parts = [part for part in normalized.split("/") if part not in ("", ".")]
        if normalized.startswith("/") or re.match(r"^[A-Za-z]:", normalized) or ".." in parts:
            raise TvgbTransferError(f"{self.config.codes['drag_drop_rejected']} guest destination must be managed relative path")

    @staticmethod
    def _sha256(source: Path) -> str:
        digest = hashlib.sha256()
        with source.open("rb") as stream:
            for chunk in iter(lambda: stream.read(1024 * 1024), b""):
                digest.update(chunk)
        return digest.hexdigest()
