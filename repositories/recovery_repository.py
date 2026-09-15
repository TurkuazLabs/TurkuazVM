# 📄 Dosya Yolu: /turkuazvm/repositories/recovery_repository.py
# 📌 Amac: Recovery session state, sidecar clone ve runtime journal verisini workspace icinde atomik olarak saklar
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Source image'e dokunmadan session dizini olusturur, izinli sidecar rolleri kopyalar ve YAML state persist eder
# Bagimli Oldugu Katman: Repo | Service

from __future__ import annotations

import os
import shutil
import tempfile
import uuid
from pathlib import Path

from tools.project_version_tool import project_version
from typing import Any

import yaml

from tools.runtime_config import RuntimeConfig


class RecoveryRepositoryError(RuntimeError):
    pass


class RecoveryRepository:
    def __init__(self, config: RuntimeConfig, root: Path | None = None) -> None:
        self.config = config
        self.root = (root or config.workspace_root()).resolve()
        self.root.mkdir(parents=True, exist_ok=True)

    def create_session(self, vm_id: str) -> Path:
        safe_vm_id = self._safe_name(vm_id)
        session_id = uuid.uuid4().hex
        session = self.root / safe_vm_id / session_id
        session.mkdir(parents=True, exist_ok=False)
        (session / str(self.config.runtime["workspace"]["sidecar_directory"])).mkdir()
        return session

    def rescue_disk_path(self, session: Path) -> Path:
        return session / str(self.config.runtime["workspace"]["rescue_disk_file"])

    def sidecar_path(self, session: Path, role: str, source: Path) -> Path:
        allowed = set(self.config.runtime["rebind"]["copied_sidecar_roles"])
        forbidden = set(self.config.runtime["rebind"]["forbidden_sidecar_roles"])
        if role in forbidden or role not in allowed:
            raise RecoveryRepositoryError(f"{self.config.codes['forbidden_sidecar_copy']} sidecar role cannot be copied: {role}")
        if source.is_symlink() or not source.is_file():
            raise RecoveryRepositoryError(f"{self.config.codes['workspace_invalid']} sidecar source must be regular file")
        destination_dir = session / str(self.config.runtime["workspace"]["sidecar_directory"])
        destination = destination_dir / f"{self._safe_name(role)}{source.suffix}"
        shutil.copy2(source, destination, follow_symlinks=False)
        return destination

    def write_state(self, session: Path, payload: dict[str, Any]) -> Path:
        state_name = str(self.config.runtime["workspace"]["state_file"])
        destination = session / state_name
        header = (
            "# 📄 Dosya Yolu: /turkuazvm/.turkuazvm/recovery/<vm>/<session>/session.yml\n"
            "# 📌 Amac: K runtime recovery session state ve journal kaydini saklar\n"
            "# 📌 Modul - YAML\n"
            f"# Version: {project_version()}\n"
            "# Aciklama: Session adimlari, source hash, overlay, sidecar ve rebind sonucunu atomik olarak persist eder\n"
            "# Bagimli Oldugu Katman: Repo | Service | View\n\n"
        )
        self._atomic_write(destination, header + yaml.safe_dump(payload, sort_keys=False))
        return destination

    def read_state(self, session: Path) -> dict[str, Any]:
        path = session / str(self.config.runtime["workspace"]["state_file"])
        if not path.is_file():
            raise RecoveryRepositoryError(f"{self.config.codes['workspace_invalid']} session state missing")
        payload = yaml.safe_load(path.read_text(encoding="utf-8"))
        if not isinstance(payload, dict):
            raise RecoveryRepositoryError(f"{self.config.codes['workspace_invalid']} session state invalid")
        return payload

    def _safe_name(self, value: str) -> str:
        cleaned = "".join(char for char in value if char.isalnum() or char in ("-", "_", "."))
        if not cleaned or cleaned in (".", ".."):
            raise RecoveryRepositoryError(f"{self.config.codes['workspace_invalid']} invalid workspace identifier")
        return cleaned[:128]

    @staticmethod
    def _atomic_write(destination: Path, text: str) -> None:
        fd, temp_name = tempfile.mkstemp(prefix=destination.name, dir=str(destination.parent))
        try:
            with os.fdopen(fd, "w", encoding="utf-8", newline="\n") as stream:
                stream.write(text)
                stream.flush()
                os.fsync(stream.fileno())
            os.replace(temp_name, destination)
        finally:
            if os.path.exists(temp_name):
                os.unlink(temp_name)
