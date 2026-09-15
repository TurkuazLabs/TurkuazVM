# 📄 Dosya Yolu: /turkuazvm/tools/network_recovery_tool.py
# 📌 Amac: QMP uzerinden guest MAC'i koruyarak NIC backend reconnect ve rescue backend gecisini uygular
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: QMP device lifecycle eventini bekler, argument allowlist uygular ve host network mutation yapmadan backend'i yeniden kurar
# Bagimli Oldugu Katman: Tool | Service

from __future__ import annotations

import re
from dataclasses import dataclass
from typing import Any, Callable

from tools.qmp_client import QmpClient, QmpError
from tools.runtime_config import RuntimeConfig


class NetworkRecoveryError(RuntimeError):
    pass


@dataclass(frozen=True)
class NicRuntimeState:
    netdev_id: str
    device_id: str
    driver: str
    mac: str
    backend_type: str
    backend_arguments: dict[str, Any]
    device_arguments: dict[str, Any]


class NetworkRecoveryTool:
    MAC_RE = re.compile(r"^[0-9a-fA-F]{2}(?::[0-9a-fA-F]{2}){5}$")

    def __init__(
        self,
        config: RuntimeConfig,
        qmp_factory: Callable[[RuntimeConfig, str], QmpClient] = QmpClient,
    ) -> None:
        self.config = config
        self.qmp_factory = qmp_factory

    def validate_state(self, state: NicRuntimeState) -> None:
        codes = self.config.codes
        if not self.MAC_RE.fullmatch(state.mac):
            raise NetworkRecoveryError(f"{codes['guest_mac_change_blocked']} invalid guest MAC")
        if state.driver not in set(self.config.runtime["network"]["allowed_nic_drivers"]):
            raise NetworkRecoveryError(f"{codes['network_arguments_rejected']} NIC driver not allowed")
        if not state.netdev_id or not state.device_id:
            raise NetworkRecoveryError(f"{codes['network_arguments_rejected']} NIC identifiers required")
        self._validate_backend_arguments(state.backend_type, state.backend_arguments)
        allowed_device_keys = set(self.config.runtime["qmp"]["allowed_device_argument_keys"])
        unknown = set(state.device_arguments) - allowed_device_keys
        if unknown:
            raise NetworkRecoveryError(f"{codes['network_arguments_rejected']} device arguments rejected: {sorted(unknown)}")

    def reconnect_link(self, endpoint: str, state: NicRuntimeState) -> dict[str, Any]:
        self.validate_state(state)
        commands = self.config.runtime["qmp"]["network_commands"]
        try:
            with self.qmp_factory(self.config, endpoint) as qmp:
                qmp.execute(str(commands["set_link"]), {"name": state.device_id, "up": False})
                qmp.execute(str(commands["set_link"]), {"name": state.device_id, "up": True})
        except QmpError as exc:
            raise NetworkRecoveryError(str(exc)) from exc
        return {"device_id": state.device_id, "mac": state.mac.lower(), "status": "RECONNECTED"}

    def recreate_backend(
        self,
        endpoint: str,
        state: NicRuntimeState,
        *,
        backend_type: str | None = None,
        backend_arguments: dict[str, Any] | None = None,
    ) -> dict[str, Any]:
        self.validate_state(state)
        target_type = backend_type or state.backend_type
        target_args = dict(backend_arguments if backend_arguments is not None else state.backend_arguments)
        self._validate_backend_arguments(target_type, target_args)
        commands = self.config.runtime["qmp"]["network_commands"]
        delete_event = str(self.config.runtime["qmp"]["device_delete_event"])

        device_add_args: dict[str, Any] = {
            "driver": state.driver,
            "id": state.device_id,
            "netdev": state.netdev_id,
            "mac": state.mac.lower(),
        }
        device_add_args.update(state.device_arguments)
        netdev_add_args: dict[str, Any] = {"type": target_type, "id": state.netdev_id}
        netdev_add_args.update(target_args)

        try:
            with self.qmp_factory(self.config, endpoint) as qmp:
                qmp.execute(str(commands["device_del"]), {"id": state.device_id})
                qmp.wait_for_event(delete_event, device_id=state.device_id)
                qmp.execute(str(commands["netdev_del"]), {"id": state.netdev_id})
                qmp.execute(str(commands["netdev_add"]), netdev_add_args)
                qmp.execute(str(commands["device_add"]), device_add_args)
        except QmpError as exc:
            raise NetworkRecoveryError(str(exc)) from exc

        return {
            "netdev_id": state.netdev_id,
            "device_id": state.device_id,
            "driver": state.driver,
            "mac": state.mac.lower(),
            "backend_type": target_type,
            "backend_arguments": target_args,
        }

    def rescue_backend_for_platform(self, endpoint: str, platform_name: str, capabilities: dict[str, bool]) -> str:
        rescue = self.config.policy["network_recovery"]["rescue_backends"]
        candidates = list(rescue.get(platform_name, []))
        qmp_backends = self.qmp_backend_types(endpoint)
        for backend in candidates:
            if backend not in qmp_backends:
                continue
            if backend == "passt" and not capabilities.get("passt", False):
                continue
            return str(backend)
        raise NetworkRecoveryError(f"{self.config.codes['capability_missing']} no rescue backend available")

    def qmp_backend_types(self, endpoint: str) -> set[str]:
        schema_command = str(self.config.runtime["qmp"]["schema_command"])
        enum_name = str(self.config.runtime["qmp"]["net_client_driver_enum"])
        try:
            with self.qmp_factory(self.config, endpoint) as qmp:
                schema = qmp.execute(schema_command, {})
        except QmpError as exc:
            raise NetworkRecoveryError(str(exc)) from exc
        if not isinstance(schema, list):
            raise NetworkRecoveryError(f"{self.config.codes['qmp_command_failed']} QMP schema response invalid")
        for item in schema:
            if not isinstance(item, dict) or item.get("name") != enum_name or item.get("meta-type") != "enum":
                continue
            members = item.get("members")
            if isinstance(members, list):
                values = {member.get("name") for member in members if isinstance(member, dict) and isinstance(member.get("name"), str)}
                if values:
                    return set(values)
            legacy_values = item.get("values")
            if isinstance(legacy_values, list) and all(isinstance(value, str) for value in legacy_values):
                return set(legacy_values)
        raise NetworkRecoveryError(f"{self.config.codes['capability_missing']} QMP network backend schema missing")

    def _validate_backend_arguments(self, backend_type: str, arguments: dict[str, Any]) -> None:
        codes = self.config.codes
        allow_map = self.config.runtime["qmp"]["allowed_backend_argument_keys"]
        if backend_type not in allow_map:
            raise NetworkRecoveryError(f"{codes['network_arguments_rejected']} backend type rejected: {backend_type}")
        forbidden = set(self.config.runtime["qmp"]["forbidden_backend_argument_keys"])
        if set(arguments) & forbidden:
            raise NetworkRecoveryError(f"{codes['network_arguments_rejected']} forbidden backend argument")
        unknown = set(arguments) - set(allow_map[backend_type])
        if unknown:
            raise NetworkRecoveryError(f"{codes['network_arguments_rejected']} backend arguments rejected: {sorted(unknown)}")
