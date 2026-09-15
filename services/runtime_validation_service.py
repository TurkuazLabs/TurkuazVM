# 📄 Dosya Yolu: /turkuazvm/services/runtime_validation_service.py
# 📌 Amac: L runtime validation harness testlerini probe, fixture ve evidence katmanlari uzerinden orkestre eder
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Smoke ve production gate kararlarini merkezi confige gore hesaplar; eksik gercek fixture durumunda fail-closed BLOCKED uretir
# Bagimli Oldugu Katman: Service | Repo | Tool

from __future__ import annotations

from copy import deepcopy
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from repositories.runtime_validation_repository import RuntimeValidationRepository
from tools.qemu_runtime_probe_tool import QemuRuntimeProbeError, QemuRuntimeProbeTool
from tools.runtime_validation_config import RuntimeValidationConfig
from tools.runtime_validation_fixture_tool import RuntimeValidationFixtureError, RuntimeValidationFixtureTool
from tools.runtime_validation_probe_tool import RuntimeValidationProbeTool


class RuntimeValidationServiceError(RuntimeError):
    pass


class RuntimeValidationService:
    def __init__(
        self,
        config: RuntimeValidationConfig,
        *,
        repository: RuntimeValidationRepository,
        probe: RuntimeValidationProbeTool,
        fixture: RuntimeValidationFixtureTool,
        qemu_runtime: QemuRuntimeProbeTool,
    ) -> None:
        self.config = config
        self.repository = repository
        self.probe = probe
        self.fixture = fixture
        self.qemu_runtime = qemu_runtime
        self.cfg = config.validation

    def run(self, *, session_id: str, mode: str) -> dict[str, Any]:
        allowed_modes = set(str(value) for value in self.cfg["modes"].values())
        if mode not in allowed_modes:
            raise RuntimeValidationServiceError(f"{self.config.codes['config_invalid']} unsupported validation mode")
        session_dir = self.repository.create_session(session_id)
        host = self.probe.host()
        binaries = self.probe.binaries(str(host["architecture"]))
        tests = self._initial_tests()
        test_ids = self.cfg["test_ids"]
        tests[str(test_ids["qemu_img"])] = self._run_qemu_img(session_dir, binaries)
        tests[str(test_ids["qmp_control_plane"])] = self._run_qmp(binaries)
        self._mark_fixture_tests(tests)

        smoke_gate = self._gate(self.cfg["required_tests"]["harness_smoke"], tests)
        production_gate = self._gate(self.cfg["required_tests"]["production"], tests)
        requested_gate = smoke_gate if mode == str(self.cfg["modes"]["smoke"]) else production_gate
        payload = {
            "schema": 1,
            "version": "0.15.0",
            "generated_at_utc": datetime.now(timezone.utc).isoformat(),
            "session_id": session_id,
            "mode": mode,
            "host": host,
            "binaries": binaries,
            "tests": tests,
            "smoke_gate": smoke_gate,
            "production_gate": production_gate,
            "requested_gate": deepcopy(requested_gate),
        }
        evidence_path = self.repository.write_evidence(payload)
        summary = {
            "schema": 1,
            "version": "0.15.0",
            "session_id": session_id,
            "mode": mode,
            "smoke_gate": smoke_gate,
            "production_gate": production_gate,
            "requested_gate": deepcopy(requested_gate),
            "evidence": str(evidence_path),
        }
        summary_path = self.repository.write_summary(summary)
        return {**summary, "summary": str(summary_path), "tests": tests, "host": host, "binaries": binaries}

    def _initial_tests(self) -> dict[str, dict[str, Any]]:
        status = self.config.statuses["not_run"]
        return {
            str(name): {"status": status, "code": self.config.codes["runtime_fixture_required"], "details": {}}
            for name in self.cfg["required_tests"]["production"]
        }

    def _run_qemu_img(self, session_dir: Path, binaries: dict[str, dict[str, Any]]) -> dict[str, Any]:
        if not binaries["qemu_img"]["available"]:
            return self._blocked_dependency("qemu_img")
        try:
            details = self.fixture.qemu_img_overlay(session_dir)
        except RuntimeValidationFixtureError as exc:
            return self._failed(self.config.codes["qemu_img_real_failed"], str(exc))
        return self._passed(details)

    def _run_qmp(self, binaries: dict[str, dict[str, Any]]) -> dict[str, Any]:
        qemu_system = binaries["qemu_system"]
        if not qemu_system["available"] or not qemu_system["resolved"]:
            return self._blocked_dependency("qemu_system")
        try:
            details = self.qemu_runtime.qmp_control_plane(str(qemu_system["resolved"]))
        except QemuRuntimeProbeError as exc:
            return self._failed(self.config.codes["qmp_probe_failed"], str(exc))
        return self._passed(details)

    def _mark_fixture_tests(self, tests: dict[str, dict[str, Any]]) -> None:
        fixture_requirements = self.cfg["fixture_requirements"]
        smoke_names = set(str(name) for name in self.cfg["required_tests"]["harness_smoke"])
        for name, result in tests.items():
            if name in smoke_names or result["status"] != self.config.statuses["not_run"]:
                continue
            requirements = fixture_requirements.get(name, [])
            tests[name] = {
                "status": self.config.statuses["blocked_fixture_required"],
                "code": self.config.codes["runtime_fixture_required"],
                "details": {"required_fixtures": list(requirements)},
            }

    def _gate(self, required: list[str], tests: dict[str, dict[str, Any]]) -> dict[str, Any]:
        statuses = [str(tests[str(name)]["status"]) for name in required]
        if all(status == self.config.statuses["pass"] for status in statuses):
            return {"status": self.config.statuses["pass"], "code": self.config.codes["pass"]}
        if any(status == self.config.statuses["fail"] for status in statuses):
            return {"status": self.config.statuses["fail"], "code": self.config.codes["production_blocked"]}
        blocked = [str(name) for name in required if tests[str(name)]["status"] != self.config.statuses["pass"]]
        if any(status == self.config.statuses["blocked_missing_dependency"] for status in statuses):
            gate_status = self.config.statuses["blocked_missing_dependency"]
        elif any(status == self.config.statuses["blocked_unsupported_host"] for status in statuses):
            gate_status = self.config.statuses["blocked_unsupported_host"]
        else:
            gate_status = self.config.statuses["blocked_fixture_required"]
        return {
            "status": gate_status,
            "code": self.config.codes["production_blocked"],
            "blocked_tests": blocked,
        }

    def _passed(self, details: dict[str, Any]) -> dict[str, Any]:
        return {"status": self.config.statuses["pass"], "code": self.config.codes["pass"], "details": details}

    def _failed(self, code: str, message: str) -> dict[str, Any]:
        return {"status": self.config.statuses["fail"], "code": code, "details": {"message": message}}

    def _blocked_dependency(self, dependency: str) -> dict[str, Any]:
        return {
            "status": self.config.statuses["blocked_missing_dependency"],
            "code": self.config.codes["dependency_missing"],
            "details": {"dependency": dependency},
        }
