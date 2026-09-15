# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_runtime_validation_harness.py
# 📌 Amac: L runtime validation harness icin config, service gate, evidence ve qemu-img CLI hotfix regressionlarini calistirir
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Gercek QEMU zorunlu olmadan orchestration ve fail-closed davranisini; K hotfixini ise argv seviyesinde dogrular
# Bagimli Oldugu Katman: Service | Repo | Tool | View

from __future__ import annotations

import json
import sys
import tempfile
from dataclasses import replace
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from repositories.runtime_validation_repository import RuntimeValidationRepository
from services.runtime_validation_service import RuntimeValidationService
from tools.command_runner import CommandResult
from tools.qemu_img_tool import QemuImgTool
from tools.qemu_runtime_probe_tool import QemuRuntimeProbeError, QemuRuntimeProbeTool
from tools.runtime_config import load_runtime_config
from tools.runtime_validation_config import RuntimeValidationConfig, load_runtime_validation_config
from tools.runtime_validation_fixture_tool import RuntimeValidationFixtureError


class FakeProbe:
    def __init__(self, *, qemu_img: bool = True, qemu_system: bool = True) -> None:
        self.qemu_img = qemu_img
        self.qemu_system = qemu_system

    def host(self) -> dict[str, Any]:
        return {
            "platform": "linux",
            "platform_raw": "linux",
            "machine": "x86_64",
            "architecture": "x86_64",
            "python": "3.13",
            "release": "test",
        }

    def binaries(self, architecture: str) -> dict[str, dict[str, Any]]:
        assert architecture == "x86_64"
        def item(name: str, available: bool) -> dict[str, Any]:
            return {
                "name": name,
                "configured": name,
                "resolved": f"/fake/{name}" if available else None,
                "available": available,
                "version_text": "fake 1.0" if available else None,
            }
        return {
            "qemu_img": item("qemu-img", self.qemu_img),
            "qemu_system": item("qemu-system-x86_64", self.qemu_system),
            "virtiofsd": item("virtiofsd", False),
            "swtpm": item("swtpm", False),
            "guest_bridge_cli": item("turkuazvm-guest-bridge", False),
            "secret_rewrap_cli": item("turkuazvm-secret-rewrap", False),
        }


class FakeFixture:
    def __init__(self, *, fail: bool = False) -> None:
        self.fail = fail

    def qemu_img_overlay(self, session_dir: Path) -> dict[str, Any]:
        assert session_dir.is_dir()
        if self.fail:
            raise RuntimeValidationFixtureError("TVM-L-201 fake qemu-img failure")
        return {
            "source_sha256": "abc",
            "overlay_format": "qcow2",
            "backing_format": "raw",
        }


class FakeQemuRuntime:
    def __init__(self, *, fail: bool = False) -> None:
        self.fail = fail

    def qmp_control_plane(self, qemu_binary: str) -> dict[str, Any]:
        assert qemu_binary.endswith("qemu-system-x86_64")
        if self.fail:
            raise QemuRuntimeProbeError("TVM-L-302 fake QMP failure")
        return {"schema_entries": 50, "endpoint_transport": "tcp"}


class FakeQemuRunner:
    def __init__(self) -> None:
        self.calls: list[tuple[str, ...]] = []

    def run(self, argv: Any, **kwargs: Any) -> CommandResult:
        args = tuple(str(item) for item in argv)
        self.calls.append(args)
        command = args[1]
        if command == "info":
            return CommandResult(args, 0, json.dumps({"format": "raw"}), "")
        if command == "create":
            Path(args[-1]).write_bytes(b"overlay")
            return CommandResult(args, 0, "", "")
        if command == "check":
            return CommandResult(args, 0, json.dumps({"corruptions": 0}), "")
        raise AssertionError(args)


def temp_config(temp: Path) -> RuntimeValidationConfig:
    loaded = load_runtime_validation_config()
    return replace(loaded, root=temp)


def test_smoke_pass_production_blocked(temp: Path) -> None:
    config = temp_config(temp)
    repo = RuntimeValidationRepository(config)
    service = RuntimeValidationService(
        config,
        repository=repo,
        probe=FakeProbe(),
        fixture=FakeFixture(),
        qemu_runtime=FakeQemuRuntime(),
    )
    result = service.run(session_id="smoke-pass", mode=str(config.validation["modes"]["smoke"]))
    assert result["smoke_gate"]["status"] == config.statuses["pass"]
    assert result["production_gate"]["status"] == config.statuses["blocked_fixture_required"]
    assert Path(result["summary"]).is_file()
    evidence = config.evidence_root() / str(config.validation["workspace"]["evidence_file"])
    assert evidence.is_file()
    assert evidence.read_text(encoding="utf-8").startswith("# 📄 Dosya Yolu:")


def test_missing_qemu_blocks_smoke(temp: Path) -> None:
    config = temp_config(temp)
    service = RuntimeValidationService(
        config,
        repository=RuntimeValidationRepository(config),
        probe=FakeProbe(qemu_img=False, qemu_system=False),
        fixture=FakeFixture(),
        qemu_runtime=FakeQemuRuntime(),
    )
    result = service.run(session_id="missing-qemu", mode=str(config.validation["modes"]["smoke"]))
    assert result["requested_gate"]["status"] != config.statuses["pass"]
    assert result["tests"]["qemu_img_overlay_real"]["status"] == config.statuses["blocked_missing_dependency"]
    assert result["tests"]["qmp_control_plane_real"]["status"] == config.statuses["blocked_missing_dependency"]


def test_real_failure_fails_gate(temp: Path) -> None:
    config = temp_config(temp)
    service = RuntimeValidationService(
        config,
        repository=RuntimeValidationRepository(config),
        probe=FakeProbe(),
        fixture=FakeFixture(fail=True),
        qemu_runtime=FakeQemuRuntime(),
    )
    result = service.run(session_id="qemu-img-fail", mode=str(config.validation["modes"]["smoke"]))
    assert result["requested_gate"]["status"] == config.statuses["fail"]


def test_qemu_img_create_uses_f_option(temp: Path) -> None:
    runtime_config = load_runtime_config()
    temp.mkdir(parents=True, exist_ok=True)
    source = temp / "base.raw"
    destination = temp / "rescue.qcow2"
    source.write_bytes(b"base")
    runner = FakeQemuRunner()
    tool = QemuImgTool(runtime_config, runner=runner)
    tool.create_rescue_overlay(source, destination)
    create_call = next(call for call in runner.calls if call[1] == "create")
    assert "-F" in create_call
    assert "-b" in create_call
    assert "-B" not in create_call


def test_qmp_schema_validation() -> None:
    validation = load_runtime_validation_config()
    runtime = load_runtime_config()
    tool = QemuRuntimeProbeTool(validation, runtime)
    tool._validate_schema([{"name": "query-status", "meta-type": "command"}, {"name": "query-qmp-schema", "meta-type": "command"}])
    try:
        tool._validate_schema([{"name": "other-command", "meta-type": "command"}])
    except QemuRuntimeProbeError as exc:
        assert validation.codes["qmp_schema_invalid"] in str(exc)
    else:
        raise AssertionError("missing QMP schema type was not rejected")


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="tvm-l-tests-") as value:
        temp = Path(value)
        test_smoke_pass_production_blocked(temp / "a")
        test_missing_qemu_blocks_smoke(temp / "b")
        test_real_failure_fails_gate(temp / "c")
        test_qemu_img_create_uses_f_option(temp / "d")
    test_qmp_schema_validation()
    print("TVM-L-000 L runtime validation regression PASS (5 groups)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
