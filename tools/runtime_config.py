# 📄 Dosya Yolu: /turkuazvm/tools/runtime_config.py
# 📌 Amac: K runtime ve J host resilience policy dosyalarini tek fail-closed loader uzerinden sunar
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Runtime katmanlarinin daginik sabit veya inline config kullanmasini engeller ve policy driftini dogrular
# Bagimli Oldugu Katman: Tool | Service | Repo | View

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any

import yaml

ROOT = Path(__file__).resolve().parents[1]
RUNTIME_CONFIG_PATH = ROOT / "config" / "host-resilience-runtime.yml"


class RuntimeConfigError(RuntimeError):
    pass


@dataclass(frozen=True)
class RuntimeConfig:
    root: Path
    runtime: dict[str, Any]
    policy: dict[str, Any]

    @property
    def codes(self) -> dict[str, str]:
        return self.runtime["status_codes"]

    def workspace_root(self) -> Path:
        value = Path(self.runtime["workspace"]["root"])
        if value.is_absolute():
            return value
        return self.root / value


def _load_yaml(path: Path, missing_code: str, invalid_code: str) -> dict[str, Any]:
    if not path.is_file():
        raise RuntimeConfigError(f"{missing_code} config missing: {path}")
    try:
        payload = yaml.safe_load(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise RuntimeConfigError(f"{invalid_code} config parse failed: {exc}") from exc
    if not isinstance(payload, dict):
        raise RuntimeConfigError(f"{invalid_code} config root must be mapping")
    return payload


def load_runtime_config(path: Path = RUNTIME_CONFIG_PATH) -> RuntimeConfig:
    payload = _load_yaml(path, "CONFIG_MISSING", "CONFIG_INVALID")
    if payload.get("schema") != 1 or payload.get("version") != "0.14.1":
        raise RuntimeConfigError("CONFIG_INVALID unsupported runtime config schema/version")
    runtime = payload.get("runtime")
    if not isinstance(runtime, dict):
        raise RuntimeConfigError("CONFIG_INVALID runtime section missing")
    codes = runtime.get("status_codes")
    if not isinstance(codes, dict) or not codes.get("pass"):
        raise RuntimeConfigError("CONFIG_INVALID status_codes missing")

    policy_rel = runtime.get("policy", {}).get("host_resilience_path")
    if not isinstance(policy_rel, str) or not policy_rel:
        raise RuntimeConfigError(f"{codes['config_invalid']} host resilience policy path missing")
    policy_path = ROOT / policy_rel
    policy_payload = _load_yaml(policy_path, codes["config_missing"], codes["config_invalid"])
    if policy_payload.get("schema") != 1 or policy_payload.get("version") != "0.14.0":
        raise RuntimeConfigError(f"{codes['config_invalid']} J policy schema/version mismatch")
    policy = policy_payload.get("host_resilience")
    if not isinstance(policy, dict):
        raise RuntimeConfigError(f"{codes['config_invalid']} host_resilience policy section missing")

    _validate_cross_policy(runtime, policy, codes)
    return RuntimeConfig(root=ROOT, runtime=runtime, policy=policy)


def _validate_cross_policy(runtime: dict[str, Any], policy: dict[str, Any], codes: dict[str, str]) -> None:
    image = policy["image_recovery"]
    network = policy["network_recovery"]
    guest = policy["guest_integration"]

    if image["immutable_source_required"] is not True or image["rescue_overlay_required"] is not True:
        raise RuntimeConfigError(f"{codes['config_invalid']} runtime requires immutable overlay J policy")
    if image["automatic_repair_allowed"] is not False:
        raise RuntimeConfigError(f"{codes['config_invalid']} automatic repair policy drift")
    if network["preserve_guest_mac"] is not True:
        raise RuntimeConfigError(f"{codes['config_invalid']} guest MAC preservation policy drift")
    if guest["drag_drop"]["transport"] != "tvgb_authenticated_file_transfer":
        raise RuntimeConfigError(f"{codes['config_invalid']} drag-drop transport policy drift")
    if guest["shared_folder"]["default_enabled"] is not False:
        raise RuntimeConfigError(f"{codes['config_invalid']} shared folder opt-in policy drift")
    if guest["shared_folder"]["default_access"] != "read_only":
        raise RuntimeConfigError(f"{codes['config_invalid']} shared folder access policy drift")

    configured_roles = set(runtime["rebind"]["source_inventory_roles"])
    required_roles = set(image["require_sidecar_inventory"])
    if configured_roles != required_roles:
        raise RuntimeConfigError(f"{codes['config_invalid']} sidecar role drift between J and K")

    attempts = int(runtime["network"]["max_runtime_attempts"])
    if attempts != int(network["max_recovery_attempts"]):
        raise RuntimeConfigError(f"{codes['config_invalid']} network attempt limit drift between J and K")
