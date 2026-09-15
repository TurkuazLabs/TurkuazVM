# 📄 Dosya Yolu: /turkuazvm/tools/qmp_client.py
# 📌 Amac: QEMU QMP endpointine timeout kontrollu JSON komutlari ve event bekleme yetenegi saglar
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Network recovery executor icin QMP greeting, capabilities, command response ve DEVICE_DELETED event akisini yonetir
# Bagimli Oldugu Katman: Tool

from __future__ import annotations

import json
import socket
import time
from pathlib import Path
from typing import Any

from tools.runtime_config import RuntimeConfig


class QmpError(RuntimeError):
    pass


class QmpClient:
    def __init__(self, config: RuntimeConfig, endpoint: str) -> None:
        self.config = config
        self.endpoint = endpoint
        self.sock: socket.socket | None = None
        self.buffer = b""
        self.pending_events: list[dict[str, Any]] = []
        self.sequence = 0

    def __enter__(self) -> "QmpClient":
        self.connect()
        return self

    def __exit__(self, exc_type: object, exc: object, tb: object) -> None:
        self.close()

    def connect(self) -> None:
        timeout = float(self.config.runtime["qmp"]["connect_timeout_seconds"])
        try:
            if self.endpoint.startswith("tcp://"):
                host_port = self.endpoint.removeprefix("tcp://")
                host, port_text = host_port.rsplit(":", 1)
                sock = socket.create_connection((host, int(port_text)), timeout=timeout)
            else:
                sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
                sock.settimeout(timeout)
                sock.connect(str(Path(self.endpoint)))
        except (OSError, ValueError) as exc:
            raise QmpError(f"{self.config.codes['qmp_connect_failed']} QMP connect failed: {exc}") from exc
        self.sock = sock
        greeting = self._read_message(timeout)
        greeting_key = str(self.config.runtime["qmp"]["greeting_key"])
        if greeting_key not in greeting:
            self.close()
            raise QmpError(f"{self.config.codes['qmp_connect_failed']} invalid QMP greeting")
        self.execute(str(self.config.runtime["qmp"]["capabilities_command"]), {})

    def close(self) -> None:
        if self.sock is not None:
            try:
                self.sock.close()
            finally:
                self.sock = None

    def execute(self, command: str, arguments: dict[str, Any]) -> Any:
        if self.sock is None:
            raise QmpError(f"{self.config.codes['qmp_connect_failed']} QMP is not connected")
        self.sequence += 1
        command_id = f"tvm-k-{self.sequence}"
        payload = {"execute": command, "arguments": arguments, "id": command_id}
        self._send(payload)
        timeout = float(self.config.runtime["qmp"]["command_timeout_seconds"])
        deadline = time.monotonic() + timeout
        error_key = str(self.config.runtime["qmp"]["response_error_key"])
        return_key = str(self.config.runtime["qmp"]["response_return_key"])
        while time.monotonic() < deadline:
            message = self._read_message(max(0.05, deadline - time.monotonic()))
            if "event" in message:
                self.pending_events.append(message)
                continue
            if message.get("id") != command_id:
                continue
            if error_key in message:
                raise QmpError(f"{self.config.codes['qmp_command_failed']} {message[error_key]}")
            if return_key in message:
                return message[return_key]
        raise QmpError(f"{self.config.codes['qmp_command_failed']} QMP command timeout: {command}")

    def wait_for_event(self, event_name: str, *, device_id: str | None = None) -> dict[str, Any]:
        timeout = float(self.config.runtime["qmp"]["command_timeout_seconds"])
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            for index, event in enumerate(list(self.pending_events)):
                if self._event_matches(event, event_name, device_id):
                    return self.pending_events.pop(index)
            message = self._read_message(max(0.05, deadline - time.monotonic()))
            if "event" in message:
                error_event = str(self.config.runtime["qmp"]["device_unplug_error_event"])
                if self._event_matches(message, error_event, device_id):
                    raise QmpError(f"{self.config.codes['qmp_command_failed']} guest rejected device unplug")
                if self._event_matches(message, event_name, device_id):
                    return message
                self.pending_events.append(message)
        raise QmpError(f"{self.config.codes['qmp_device_delete_timeout']} event timeout: {event_name}")

    @staticmethod
    def _event_matches(event: dict[str, Any], name: str, device_id: str | None) -> bool:
        if event.get("event") != name:
            return False
        if device_id is None:
            return True
        data = event.get("data")
        if not isinstance(data, dict):
            return False
        return data.get("device") == device_id or data.get("path") == device_id

    def _send(self, payload: dict[str, Any]) -> None:
        assert self.sock is not None
        raw = json.dumps(payload, separators=(",", ":")).encode("utf-8") + b"\r\n"
        try:
            self.sock.sendall(raw)
        except OSError as exc:
            raise QmpError(f"{self.config.codes['qmp_command_failed']} QMP send failed: {exc}") from exc

    def _read_message(self, timeout: float) -> dict[str, Any]:
        if self.sock is None:
            raise QmpError(f"{self.config.codes['qmp_connect_failed']} QMP is not connected")
        self.sock.settimeout(timeout)
        while b"\n" not in self.buffer:
            try:
                chunk = self.sock.recv(65536)
            except socket.timeout as exc:
                raise QmpError(f"{self.config.codes['qmp_command_failed']} QMP read timeout") from exc
            except OSError as exc:
                raise QmpError(f"{self.config.codes['qmp_command_failed']} QMP read failed: {exc}") from exc
            if not chunk:
                raise QmpError(f"{self.config.codes['qmp_command_failed']} QMP connection closed")
            self.buffer += chunk
        line, self.buffer = self.buffer.split(b"\n", 1)
        line = line.rstrip(b"\r")
        try:
            payload = json.loads(line.decode("utf-8"))
        except (UnicodeDecodeError, json.JSONDecodeError) as exc:
            raise QmpError(f"{self.config.codes['qmp_command_failed']} invalid QMP JSON") from exc
        if not isinstance(payload, dict):
            raise QmpError(f"{self.config.codes['qmp_command_failed']} invalid QMP message")
        return payload
