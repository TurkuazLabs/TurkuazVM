# 📄 Dosya Yolu: /turkuazvm/tools/runtime_validation_fixture_tool.py
# 📌 Amac: L harness icin gecici gercek disk fixture olusturur ve immutable qemu-img overlay davranisini dogrular
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Guest boot gerektirmeden gercek qemu-img binary ile source hash, backing chain ve overlay check smoke testi uygular
# Bagimli Oldugu Katman: Tool | Service

from __future__ import annotations

import hashlib
from pathlib import Path
from typing import Any

from tools.qemu_img_tool import QemuImgError, QemuImgTool
from tools.runtime_validation_config import RuntimeValidationConfig


class RuntimeValidationFixtureError(RuntimeError):
    pass


class RuntimeValidationFixtureTool:
    def __init__(self, config: RuntimeValidationConfig, qemu_img: QemuImgTool) -> None:
        self.config = config
        self.qemu_img = qemu_img
        self.cfg = config.validation["qemu_img"]

    def qemu_img_overlay(self, session_dir: Path) -> dict[str, Any]:
        fixture_dir = session_dir / str(self.config.validation["workspace"]["fixtures_directory"])
        fixture_dir.mkdir(parents=True, exist_ok=True)
        workspace_cfg = self.config.validation["workspace"]
        source = fixture_dir / str(workspace_cfg["source_fixture_file"])
        overlay = fixture_dir / str(workspace_cfg["overlay_fixture_file"])
        size_bytes = self._parse_size(str(self.cfg["fixture_size"]))
        with source.open("wb") as stream:
            stream.truncate(size_bytes)
        before = self._hash(source)
        try:
            check_source = self.qemu_img.check_read_only(source)
            created = self.qemu_img.create_rescue_overlay(source, overlay)
            overlay_info = self.qemu_img.info(overlay)
            check_overlay = self.qemu_img.check_read_only(overlay)
        except QemuImgError as exc:
            raise RuntimeValidationFixtureError(f"{self.config.codes['qemu_img_real_failed']} {exc}") from exc
        after = self._hash(source)
        if before != after:
            raise RuntimeValidationFixtureError(f"{self.config.codes['source_mutation_detected']} source fixture changed")
        keys = self.cfg["info_keys"]
        format_key = str(keys["format"])
        backing_key = str(keys["backing_filename"])
        backing_format_key = str(keys["backing_format"])
        if str(overlay_info.get(format_key)) != str(self.cfg["overlay_format"]):
            raise RuntimeValidationFixtureError(f"{self.config.codes['backing_chain_invalid']} overlay format mismatch")
        backing_value = overlay_info.get(backing_key)
        if not isinstance(backing_value, str) or Path(backing_value).resolve() != source.resolve():
            raise RuntimeValidationFixtureError(f"{self.config.codes['backing_chain_invalid']} backing filename mismatch")
        expected_backing_format = str(self.cfg["expected_overlay_backing_format"])
        actual_backing_format = overlay_info.get(backing_format_key)
        if actual_backing_format is not None and str(actual_backing_format) != expected_backing_format:
            raise RuntimeValidationFixtureError(f"{self.config.codes['backing_chain_invalid']} backing format mismatch")
        return {
            "source": str(source.resolve()),
            "source_sha256": before,
            "source_check_exit_code": int(check_source.get("exit_code", -1)),
            "overlay": str(overlay.resolve()),
            "overlay_format": str(overlay_info.get(format_key)),
            "backing_filename": str(backing_value),
            "backing_format": str(actual_backing_format or expected_backing_format),
            "overlay_check_exit_code": int(check_overlay.get("exit_code", -1)),
            "create_result": created,
        }

    def _hash(self, path: Path) -> str:
        digest = hashlib.new(str(self.cfg["source_hash_algorithm"]))
        with path.open("rb") as stream:
            for chunk in iter(lambda: stream.read(1024 * 1024), b""):
                digest.update(chunk)
        return digest.hexdigest()

    def _parse_size(self, value: str) -> int:
        suffixes = {str(key): int(amount) for key, amount in self.config.validation["size_units"].items()}
        text = value.strip().upper()
        if text[-1:] in suffixes:
            number = text[:-1]
            if not number.isdigit():
                raise ValueError("invalid fixture size")
            return int(number) * suffixes[text[-1]]
        if not text.isdigit():
            raise ValueError("invalid fixture size")
        return int(text)
