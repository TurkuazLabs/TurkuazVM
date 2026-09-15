# 📄 Dosya Yolu: /turkuazvm/tools/secret_rewrap_tool.py
# 📌 Amac: Host-bound secret referanslarini destination host secure-store adaptorune kontrollu olarak rewrap ettirir
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Secret degerini command line'a koymadan JSON stdin ile harici secure-store adaptorune iletir ve fail-closed sonuc alir
# Bagimli Oldugu Katman: Tool | Service

from __future__ import annotations

import json
from typing import Any

from tools.command_runner import CommandError, CommandRunner
from tools.runtime_config import RuntimeConfig


class SecretRewrapError(RuntimeError):
    pass


class SecretRewrapTool:
    def __init__(self, config: RuntimeConfig, runner: CommandRunner | None = None) -> None:
        self.config = config
        self.runner = runner or CommandRunner(config)
        self.binary = str(config.runtime["binaries"]["secret_rewrap_cli"])

    def rewrap(self, request: dict[str, Any]) -> dict[str, Any]:
        safe_request = dict(request)
        if "secret_value" in safe_request:
            raise SecretRewrapError(f"{self.config.codes['forbidden_sidecar_copy']} raw secret value is forbidden")
        try:
            result = self.runner.run((self.binary,), input_text=json.dumps(safe_request, sort_keys=True))
        except CommandError as exc:
            raise SecretRewrapError(f"{self.config.codes['secret_rewrap_required']} {exc}") from exc
        try:
            payload = json.loads(result.stdout)
        except json.JSONDecodeError as exc:
            raise SecretRewrapError(f"{self.config.codes['secret_rewrap_required']} rewrap response invalid JSON") from exc
        if not isinstance(payload, dict) or payload.get("status") != "PASS" or not payload.get("destination_ref"):
            raise SecretRewrapError(f"{self.config.codes['secret_rewrap_required']} rewrap was not confirmed")
        return payload
