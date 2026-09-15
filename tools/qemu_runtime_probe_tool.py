# 📄 Dosya Yolu: /turkuazvm/tools/qemu_runtime_probe_tool.py
# 📌 Amac: L harness icin gercek qemu-system prosesi baslatir ve QMP control-plane smoke testini uygular
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Rastgele localhost TCP QMP endpointi ile guest boot etmeden QEMU greeting, command ve schema davranisini gercek proseste dogrular
# Bagimli Oldugu Katman: Tool | Service

from __future__ import annotations

import socket
import subprocess
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from tools.qmp_client import QmpClient, QmpError
from tools.runtime_config import RuntimeConfig
from tools.runtime_validation_config import RuntimeValidationConfig


class QemuRuntimeProbeError(RuntimeError):
    pass


@dataclass
class QemuRuntimeSession:
    process: subprocess.Popen[str]
    endpoint: str
    argv: tuple[str, ...]


class QemuRuntimeProbeTool:
    def __init__(self, config: RuntimeValidationConfig, runtime_config: RuntimeConfig) -> None:
        self.config = config
        self.runtime_config = runtime_config
        self.cfg = config.validation

    def qmp_control_plane(self, qemu_binary: str) -> dict[str, Any]:
        host = str(self.cfg["qmp"]["bind_host"])
        port = self._reserve_port(host)
        endpoint_value = str(self.cfg["qmp"]["endpoint_template"]).format(host=host, port=port)
        client_endpoint = str(self.cfg["qmp"]["client_endpoint_template"]).format(host=host, port=port)
        argv = [qemu_binary]
        argv.extend(str(item) for item in self.cfg["qmp"]["launch_arguments"])
        argv.extend(("-qmp", endpoint_value))
        session = self._start(tuple(argv), client_endpoint)
        try:
            self._wait_for_endpoint(host, port, session.process)
            with QmpClient(self.runtime_config, session.endpoint) as qmp:
                status = qmp.execute(str(self.cfg["qmp"]["query_status_command"]), {})
                schema = qmp.execute(str(self.cfg["qmp"]["query_schema_command"]), {})
            self._validate_schema(schema)
            return {
                "status_response": status,
                "schema_entries": len(schema) if isinstance(schema, list) else 0,
                "endpoint_transport": str(self.cfg["qmp"]["transport"]),
                "argv": list(session.argv),
            }
        except (QmpError, OSError, ValueError) as exc:
            raise QemuRuntimeProbeError(f"{self.config.codes['qmp_probe_failed']} {exc}") from exc
        finally:
            self._stop(session)

    def _start(self, argv: tuple[str, ...], endpoint: str) -> QemuRuntimeSession:
        try:
            process = subprocess.Popen(
                argv,
                stdin=subprocess.DEVNULL,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                shell=False,
            )
        except OSError as exc:
            raise QemuRuntimeProbeError(f"{self.config.codes['qmp_launch_failed']} {exc}") from exc
        return QemuRuntimeSession(process=process, endpoint=endpoint, argv=argv)

    def _wait_for_endpoint(self, host: str, port: int, process: subprocess.Popen[str]) -> None:
        deadline = time.monotonic() + float(self.cfg["command"]["process_start_timeout_seconds"])
        while time.monotonic() < deadline:
            if process.poll() is not None:
                stderr = process.stderr.read() if process.stderr else ""
                raise QemuRuntimeProbeError(f"{self.config.codes['qmp_launch_failed']} QEMU exited: {stderr.strip()}")
            try:
                with socket.create_connection((host, port), timeout=0.2):
                    return
            except OSError:
                time.sleep(0.05)
        raise QemuRuntimeProbeError(f"{self.config.codes['qmp_launch_failed']} QMP endpoint timeout")

    def _stop(self, session: QemuRuntimeSession) -> None:
        process = session.process
        if process.poll() is not None:
            return
        process.terminate()
        try:
            process.wait(timeout=float(self.cfg["command"]["process_stop_timeout_seconds"]))
        except subprocess.TimeoutExpired:
            process.kill()
            try:
                process.wait(timeout=float(self.cfg["command"]["process_stop_timeout_seconds"]))
            except subprocess.TimeoutExpired as exc:
                raise QemuRuntimeProbeError(f"{self.config.codes['process_cleanup_failed']} QEMU cleanup timeout") from exc

    def _validate_schema(self, schema: Any) -> None:
        if not isinstance(schema, list):
            raise QemuRuntimeProbeError(f"{self.config.codes['qmp_schema_invalid']} schema root invalid")
        commands = {
            str(item.get("name"))
            for item in schema
            if isinstance(item, dict) and item.get("meta-type") == "command" and item.get("name")
        }
        required = {str(item) for item in self.cfg["qmp"]["required_schema_commands"]}
        missing = sorted(required - commands)
        if missing:
            raise QemuRuntimeProbeError(f"{self.config.codes['qmp_schema_invalid']} missing schema commands: {missing}")

    @staticmethod
    def _reserve_port(host: str) -> int:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
            sock.bind((host, 0))
            return int(sock.getsockname()[1])
