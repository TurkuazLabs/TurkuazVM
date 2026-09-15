# 📄 Dosya Yolu: /turkuazvm/repositories/runtime_validation_repository.py
# 📌 Amac: L runtime validation session ve evidence dosyalarini atomik YAML olarak saklar
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Runtime test sonucunu Repo katmaninda persist eder ve yarim evidence yazimini engeller
# Bagimli Oldugu Katman: Repo | Service

from __future__ import annotations

import os
from pathlib import Path

from tools.project_version_tool import project_version
from typing import Any

import yaml

from tools.runtime_validation_config import RuntimeValidationConfig


class RuntimeValidationRepositoryError(RuntimeError):
    pass


class RuntimeValidationRepository:
    def __init__(self, config: RuntimeValidationConfig) -> None:
        self.config = config
        self.workspace = config.workspace_root()
        self.evidence_root = config.evidence_root()

    def create_session(self, session_id: str) -> Path:
        safe = self._safe_id(session_id)
        path = self.workspace / str(self.config.validation["workspace"]["sessions_directory"]) / safe
        path.mkdir(parents=True, exist_ok=False)
        return path

    def write_evidence(self, payload: dict[str, Any]) -> Path:
        self.evidence_root.mkdir(parents=True, exist_ok=True)
        target = self.evidence_root / str(self.config.validation["workspace"]["evidence_file"])
        self._atomic_yaml(target, payload)
        return target

    def write_summary(self, payload: dict[str, Any]) -> Path:
        self.evidence_root.mkdir(parents=True, exist_ok=True)
        target = self.evidence_root / str(self.config.validation["workspace"]["summary_file"])
        self._atomic_yaml(target, payload)
        return target

    def _atomic_yaml(self, target: Path, payload: dict[str, Any]) -> None:
        tmp = target.with_suffix(target.suffix + ".tmp")
        header = (
            "# 📄 Dosya Yolu: /turkuazvm/artifacts/runtime-validation/" + target.name + "\n"
            "# 📌 Amac: L runtime validation evidence sonucunu makine-okunur olarak saklar\n"
            "# 📌 Modul - YAML\n"
            f"# Version: {project_version()}\n"
            "# Aciklama: Runtime harness tarafindan atomik uretilen validation artifactidir\n"
            "# Bagimli Oldugu Katman: Repo | Tool | View\n\n"
        )
        body = yaml.safe_dump(payload, sort_keys=False, allow_unicode=False)
        tmp.write_text(header + body, encoding="utf-8")
        os.replace(tmp, target)

    def _safe_id(self, value: str) -> str:
        if not value or len(value) > 96 or any(ch not in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_." for ch in value):
            raise RuntimeValidationRepositoryError(f"{self.config.codes['fixture_invalid']} invalid session id")
        return value
