# 📄 Dosya Yolu: /turkuazvm/tools/qemu_img_tool.py
# 📌 Amac: Kaynak diski mutate etmeden qemu-img info/check ve qcow2 rescue overlay islemlerini uygular
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Source hash guard, read-only check ve backing-file overlay olusturma davranisini fail-closed uygular
# Bagimli Oldugu Katman: Tool | Service

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any

from tools.command_runner import CommandError, CommandRunner
from tools.runtime_config import RuntimeConfig


class QemuImgError(RuntimeError):
    pass


class QemuImgTool:
    def __init__(self, config: RuntimeConfig, runner: CommandRunner | None = None) -> None:
        self.config = config
        self.runner = runner or CommandRunner(config)
        self.binary = str(config.runtime["binaries"]["qemu_img"])
        self.image_cfg = config.runtime["image"]

    def hash_file(self, path: Path) -> str:
        algorithm = str(self.image_cfg["hash_algorithm"])
        digest = hashlib.new(algorithm)
        with path.open("rb") as stream:
            for chunk in iter(lambda: stream.read(1024 * 1024), b""):
                digest.update(chunk)
        return digest.hexdigest()

    def info(self, source: Path) -> dict[str, Any]:
        self._require_source(source)
        result = self.runner.run((self.binary, "info", "--output=json", str(source)))
        try:
            payload = json.loads(result.stdout)
        except json.JSONDecodeError as exc:
            raise QemuImgError(f"{self.config.codes['rescue_overlay_failed']} qemu-img info invalid JSON") from exc
        if not isinstance(payload, dict) or not payload.get("format"):
            raise QemuImgError(f"{self.config.codes['rescue_overlay_failed']} qemu-img format missing")
        return payload

    def check_read_only(self, source: Path) -> dict[str, Any]:
        self._require_source(source)
        before = self.hash_file(source)
        result = self.runner.run(
            (self.binary, "check", "--output=json", str(source)),
            allowed_returncodes=(0, 1, 2, 3, 63),
        )
        after = self.hash_file(source)
        if before != after:
            raise QemuImgError(f"{self.config.codes['source_hash_changed']} source changed during read-only check")
        payload: dict[str, Any] = {}
        text = result.stdout.strip()
        if text:
            try:
                loaded = json.loads(text)
            except json.JSONDecodeError as exc:
                raise QemuImgError(f"{self.config.codes['rescue_overlay_failed']} qemu-img check invalid JSON") from exc
            if isinstance(loaded, dict):
                payload = loaded
        payload["exit_code"] = result.returncode
        payload["source_sha256"] = before
        return payload

    def create_rescue_overlay(self, source: Path, destination: Path) -> dict[str, Any]:
        self._require_source(source)
        if destination.exists():
            raise QemuImgError(f"{self.config.codes['rescue_overlay_failed']} rescue destination already exists")
        destination.parent.mkdir(parents=True, exist_ok=True)
        before = self.hash_file(source)
        info = self.info(source)
        source_format = str(info["format"])
        overlay_format = str(self.image_cfg["overlay_format"])
        try:
            self.runner.run(
                (
                    self.binary,
                    "create",
                    "-f",
                    overlay_format,
                    str(self.image_cfg["backing_format_option"]),
                    source_format,
                    str(self.image_cfg["backing_file_option"]),
                    str(source.resolve()),
                    str(destination),
                ),
                timeout=int(self.config.runtime["command"]["long_timeout_seconds"]),
            )
        except CommandError as exc:
            raise QemuImgError(f"{self.config.codes['rescue_overlay_failed']} {exc}") from exc
        after = self.hash_file(source)
        if before != after:
            destination.unlink(missing_ok=True)
            raise QemuImgError(f"{self.config.codes['source_hash_changed']} source changed while overlay was created")
        if not destination.is_file():
            raise QemuImgError(f"{self.config.codes['rescue_overlay_failed']} overlay was not created")
        return {
            "source": str(source.resolve()),
            "source_format": source_format,
            "source_sha256": before,
            "overlay": str(destination.resolve()),
            "overlay_format": overlay_format,
        }

    def _require_source(self, source: Path) -> None:
        if source.is_symlink() or not source.is_file():
            raise QemuImgError(f"{self.config.codes['workspace_invalid']} source image missing: {source}")
