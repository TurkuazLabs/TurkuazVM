# 📄 Dosya Yolu: /turkuazvm/tools/runtime_validation_config.py
# 📌 Amac: L runtime validation configini tek kaynak olarak yukler ve K runtime configi ile fail-closed eslestirir
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Validation harness icinde inline config ve daginik status kullanilmasini engeller
# Bagimli Oldugu Katman: Tool | Service | Repo | View

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any

import yaml

ROOT = Path(__file__).resolve().parents[1]
CONFIG_PATH = ROOT / "config" / "runtime-validation.yml"


class RuntimeValidationConfigError(RuntimeError):
    pass


@dataclass(frozen=True)
class RuntimeValidationConfig:
    root: Path
    validation: dict[str, Any]

    @property
    def codes(self) -> dict[str, str]:
        return self.validation["status_codes"]

    @property
    def statuses(self) -> dict[str, str]:
        return self.validation["statuses"]

    def workspace_root(self) -> Path:
        value = Path(str(self.validation["workspace"]["root"]))
        return value if value.is_absolute() else self.root / value

    def evidence_root(self) -> Path:
        value = Path(str(self.validation["workspace"]["evidence_directory"]))
        return value if value.is_absolute() else self.root / value


def load_runtime_validation_config(path: Path = CONFIG_PATH) -> RuntimeValidationConfig:
    if not path.is_file():
        raise RuntimeValidationConfigError(f"TVM-L-101 config missing: {path}")
    try:
        payload = yaml.safe_load(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise RuntimeValidationConfigError(f"TVM-L-102 config parse failed: {exc}") from exc
    if not isinstance(payload, dict) or payload.get("schema") != 1 or payload.get("version") != "0.15.0":
        raise RuntimeValidationConfigError("TVM-L-102 unsupported runtime validation config")
    validation = payload.get("runtime_validation")
    if not isinstance(validation, dict):
        raise RuntimeValidationConfigError("TVM-L-102 runtime_validation section missing")
    codes = validation.get("status_codes")
    statuses = validation.get("statuses")
    if not isinstance(codes, dict) or not codes.get("pass"):
        raise RuntimeValidationConfigError("TVM-L-102 status_codes missing")
    if not isinstance(statuses, dict) or statuses.get("pass") != "PASS":
        raise RuntimeValidationConfigError(f"{codes['config_invalid']} statuses invalid")

    base_rel = validation.get("base_runtime_config")
    if not isinstance(base_rel, str) or not base_rel:
        raise RuntimeValidationConfigError(f"{codes['config_invalid']} base runtime config missing")
    base_path = ROOT / base_rel
    if not base_path.is_file():
        raise RuntimeValidationConfigError(f"{codes['config_missing']} base runtime config missing: {base_path}")
    base = yaml.safe_load(base_path.read_text(encoding="utf-8"))
    if not isinstance(base, dict) or base.get("version") != "0.14.1":
        raise RuntimeValidationConfigError(f"{codes['config_invalid']} K runtime config version mismatch")

    production = validation.get("required_tests", {}).get("production")
    smoke = validation.get("required_tests", {}).get("harness_smoke")
    if not isinstance(production, list) or not production or not all(isinstance(item, str) for item in production):
        raise RuntimeValidationConfigError(f"{codes['config_invalid']} production test list invalid")
    if not isinstance(smoke, list) or not smoke or not set(smoke).issubset(set(production)):
        raise RuntimeValidationConfigError(f"{codes['config_invalid']} harness smoke list invalid")

    qemu_img = validation.get("qemu_img", {})
    if qemu_img.get("backing_format_option") != "-F" or qemu_img.get("backing_file_option") != "-b":
        raise RuntimeValidationConfigError(f"{codes['config_invalid']} qemu-img backing options drift")

    return RuntimeValidationConfig(root=ROOT, validation=validation)
