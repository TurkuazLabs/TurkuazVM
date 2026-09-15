# 📄 Dosya Yolu: /turkuazvm/tools/guest_network_tool.py
# 📌 Amac: Guest Agent uzerinden soft DHCP/DNS renew ve post-recovery network verify islemlerini uygular
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Network recovery state machine'in guest tarafindaki ilk ve dogrulama adimlarini Guest Bridge CLI adaptorune delege eder
# Bagimli Oldugu Katman: Tool | Service

from __future__ import annotations

import json
from typing import Any

from tools.command_runner import CommandError, CommandRunner
from tools.runtime_config import RuntimeConfig


class GuestNetworkError(RuntimeError):
    pass


class GuestNetworkTool:
    def __init__(self, config: RuntimeConfig, runner: CommandRunner | None = None) -> None:
        self.config = config
        self.runner = runner or CommandRunner(config)
        self.binary = str(config.runtime["binaries"]["guest_bridge_cli"])
        self.operation_flag = str(config.runtime["drag_drop"]["cli_flags"]["operation"])

    def soft_renew(self) -> dict[str, Any]:
        return self._operation(str(self.config.runtime["network"]["soft_renew_operation"]))

    def verify(self) -> dict[str, Any]:
        return self._operation(str(self.config.runtime["network"]["verify_operation"]))

    def _operation(self, operation: str) -> dict[str, Any]:
        try:
            result = self.runner.run((self.binary, self.operation_flag, operation))
        except CommandError as exc:
            raise GuestNetworkError(f"{self.config.codes['capability_missing']} {exc}") from exc
        try:
            payload = json.loads(result.stdout)
        except json.JSONDecodeError as exc:
            raise GuestNetworkError(f"{self.config.codes['network_attempts_exhausted']} guest network response invalid JSON") from exc
        if not isinstance(payload, dict) or payload.get("status") != "PASS":
            raise GuestNetworkError(f"{self.config.codes['network_attempts_exhausted']} guest network operation failed: {operation}")
        return payload
