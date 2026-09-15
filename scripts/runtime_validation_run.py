# 📄 Dosya Yolu: /turkuazvm/scripts/runtime_validation_run.py
# 📌 Amac: L runtime validation harness icin CLI composition root saglar ve smoke veya production gate'i calistirir
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Config, Repo, Tool, Service ve Controller katmanlarini baglar; requested gate PASS degilse fail-closed exit verir
# Bagimli Oldugu Katman: Controller | Service | Repo | Tool | View

from __future__ import annotations

import argparse
import sys
import uuid
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from controllers.runtime_validation_controller import RuntimeValidationController
from repositories.runtime_validation_repository import RuntimeValidationRepository
from services.runtime_validation_service import RuntimeValidationService
from tools.qemu_img_tool import QemuImgTool
from tools.qemu_runtime_probe_tool import QemuRuntimeProbeTool
from tools.runtime_config import load_runtime_config
from tools.runtime_validation_config import load_runtime_validation_config
from tools.runtime_validation_fixture_tool import RuntimeValidationFixtureTool
from tools.runtime_validation_probe_tool import RuntimeValidationProbeTool
from views.runtime_validation_view import RuntimeValidationView


def build_controller() -> tuple[RuntimeValidationController, RuntimeValidationView, object]:
    validation_config = load_runtime_validation_config()
    runtime_config = load_runtime_config()
    repository = RuntimeValidationRepository(validation_config)
    probe = RuntimeValidationProbeTool(validation_config)
    qemu_img = QemuImgTool(runtime_config)
    fixture = RuntimeValidationFixtureTool(validation_config, qemu_img)
    qemu_runtime = QemuRuntimeProbeTool(validation_config, runtime_config)
    service = RuntimeValidationService(
        validation_config,
        repository=repository,
        probe=probe,
        fixture=fixture,
        qemu_runtime=qemu_runtime,
    )
    return RuntimeValidationController(service), RuntimeValidationView(), validation_config


def main() -> int:
    controller, view, config = build_controller()
    parser = argparse.ArgumentParser()
    parser.add_argument("--mode", choices=sorted(str(value) for value in config.validation["modes"].values()), required=True)
    parser.add_argument("--session-id")
    args = parser.parse_args()
    session_id = args.session_id or _session_id()
    result = controller.run({"session_id": session_id, "mode": args.mode})
    rendered = view.render(result)
    print(rendered)
    exit_codes = config.validation["exit_codes"]
    return int(exit_codes["pass"]) if result["requested_gate"]["status"] == config.statuses["pass"] else int(exit_codes["blocked"])


def _session_id() -> str:
    timestamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    return f"l-{timestamp}-{uuid.uuid4().hex[:8]}"


if __name__ == "__main__":
    raise SystemExit(main())
