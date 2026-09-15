# 📄 Dosya Yolu: /turkuazvm/scripts/host_resilience_runtime_gate.py
# 📌 Amac: K runtime wiring icin layered architecture, config, language, header ve unsafe primitive driftini fail-closed dogrular
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Controller-Service-Repo-Tool sinirlarini, merkezi status kodlarini ve runtime artifact evidence uretimini CI gate olarak uygular
# Bagimli Oldugu Katman: Tool | View

from __future__ import annotations

import ast
import sys
from pathlib import Path

from typing import Any

import yaml

ROOT = Path(__file__).resolve().parents[1]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from tools.project_version_tool import project_version

from tools.runtime_config import RuntimeConfigError, load_runtime_config

SUMMARY_PATH = ROOT / "artifacts" / "host-resilience-runtime-summary.yml"
IMPLEMENTATION_DIRS = ("controllers", "services", "repositories", "tools", "views")
HEADER_PREFIXES = (
    "# 📄 Dosya Yolu:",
    "# 📌 Amac:",
    "# 📌 Modul -",
    "# Version:",
    "# Aciklama:",
    "# Bagimli Oldugu Katman:",
)


class RuntimeGateError(RuntimeError):
    pass


def require(condition: bool, code: str, message: str) -> None:
    if not condition:
        raise RuntimeGateError(f"{code} {message}")


def _python_files(directory: str) -> list[Path]:
    return sorted(path for path in (ROOT / directory).glob("*.py") if path.is_file())


def validate_headers(config: Any) -> list[str]:
    files: list[Path] = []
    for directory in IMPLEMENTATION_DIRS:
        files.extend(_python_files(directory))
    files.extend(
        [
            ROOT / "config" / "host-resilience-runtime.yml",
            ROOT / "language" / "tr.yml",
            ROOT / "language" / "en.yml",
            ROOT / "scripts" / "host_resilience_runtime_gate.py",
            ROOT / "scripts" / "tests" / "test_host_resilience_runtime.py",
        ]
    )
    for path in files:
        require(path.is_file(), config.codes["config_missing"], f"required K file missing: {path.relative_to(ROOT)}")
        raw = path.read_bytes()
        require(b"\x00" not in raw, config.codes["config_invalid"], f"NUL byte found: {path.relative_to(ROOT)}")
        lines = raw.decode("utf-8").splitlines()
        require(len(lines) >= len(HEADER_PREFIXES), config.codes["config_invalid"], f"header missing: {path.relative_to(ROOT)}")
        for index, prefix in enumerate(HEADER_PREFIXES):
            require(lines[index].startswith(prefix), config.codes["config_invalid"], f"header line {index + 1} invalid: {path.relative_to(ROOT)}")
    return ["technical headers", "UTF-8/NUL integrity"]


def validate_layer_imports(config: Any) -> list[str]:
    forbidden_by_layer = {
        "controllers": ("tools", "repositories", "views", "language"),
        "services": ("controllers", "views", "language"),
        "repositories": ("controllers", "services", "views", "language"),
        "tools": ("controllers", "services", "repositories", "views", "language"),
        "views": ("controllers", "services", "repositories", "tools"),
    }
    for layer, forbidden in forbidden_by_layer.items():
        for path in _python_files(layer):
            tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
            for node in ast.walk(tree):
                module = ""
                if isinstance(node, ast.ImportFrom):
                    module = node.module or ""
                elif isinstance(node, ast.Import) and node.names:
                    for alias in node.names:
                        top = alias.name.split(".", 1)[0]
                        require(top not in forbidden, config.codes["config_invalid"], f"layer import violation {layer}: {path.name} -> {top}")
                    continue
                if module:
                    top = module.split(".", 1)[0]
                    require(top not in forbidden, config.codes["config_invalid"], f"layer import violation {layer}: {path.name} -> {top}")
    return ["Controller -> Service boundary", "Service/Repo/Tool dependency direction"]


def validate_status_localization(config: Any) -> list[str]:
    codes = set(config.codes.values())
    for language_file in (ROOT / "language" / "tr.yml", ROOT / "language" / "en.yml"):
        payload = yaml.safe_load(language_file.read_text(encoding="utf-8"))
        require(isinstance(payload, dict), config.codes["config_invalid"], f"language file invalid: {language_file.name}")
        localized_k_codes = {str(key) for key in payload if str(key).startswith("TVM-K-")}
        missing = sorted(codes - localized_k_codes)
        extra = sorted(localized_k_codes - codes)
        require(not missing, config.codes["config_invalid"], f"language codes missing in {language_file.name}: {missing}")
        require(not extra, config.codes["config_invalid"], f"K language codes unknown in {language_file.name}: {extra}")
    return ["TR/EN status localization parity"]


def validate_magic_status_strings(config: Any) -> list[str]:
    for directory in IMPLEMENTATION_DIRS:
        for path in _python_files(directory):
            source = path.read_text(encoding="utf-8")
            require("TVM-K-" not in source, config.codes["config_invalid"], f"inline K status literal found: {path.relative_to(ROOT)}")
    return ["centralized K status codes"]


def validate_runtime_primitives(config: Any) -> list[str]:
    controller = (ROOT / "controllers" / "host_resilience_controller.py").read_text(encoding="utf-8")
    for token in ("subprocess", "socket.", "shutil", "yaml.", "qemu-img", "virtiofsd"):
        require(token not in controller, config.codes["config_invalid"], f"controller logic primitive found: {token}")

    all_python = []
    for directory in IMPLEMENTATION_DIRS:
        all_python.extend(_python_files(directory))
    for path in all_python:
        source = path.read_text(encoding="utf-8")
        require("shell=True" not in source, config.codes["unsafe_command_blocked"], f"shell=True forbidden: {path.relative_to(ROOT)}")
        if "subprocess" in source:
            require(path.parent.name == "tools", config.codes["config_invalid"], f"subprocess outside Tool layer: {path.relative_to(ROOT)}")

    qemu_source = (ROOT / "tools" / "qemu_img_tool.py").read_text(encoding="utf-8")
    repair_flag = str(config.runtime["image"]["forbidden_repair_argument"])
    require(f'"{repair_flag}"' not in qemu_source and f"'{repair_flag}'" not in qemu_source, config.codes["unsafe_command_blocked"], "qemu-img repair flag embedded in runtime tool")
    return ["no controller runtime logic", "subprocess Tool-only", "shell disabled", "qemu-img repair flag absent"]


def validate_release_policy(config: Any) -> list[str]:
    release = config.policy["release_gate"]
    require(release["production_status"] == "BLOCKED_UNTIL_RUNTIME_TESTS_PASS", config.codes["runtime_test_required"], "J production block drift")
    require(config.policy["guest_integration"]["shared_folder"]["backends"]["windows"]["maturity"] == "experimental", config.codes["windows_virtiofs_blocked"], "Windows VirtioFS maturity drift")
    return ["runtime release remains blocked", "Windows VirtioFS remains experimental"]


def write_summary(status: str, checks: list[str], error: str | None = None) -> None:
    SUMMARY_PATH.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema": 1,
        "version": project_version(),
        "module": "host-resilience-k-runtime-wiring",
        "status": status,
        "checks": checks,
        "error": error,
    }
    header = (
        "# 📄 Dosya Yolu: /turkuazvm/artifacts/host-resilience-runtime-summary.yml\n"
        "# 📌 Amac: K runtime wiring static gate sonucunu makine-okunur evidence olarak kaydeder\n"
        "# 📌 Modul - YAML\n"
        f"# Version: {project_version()}\n"
        "# Aciklama: Layer, config, localization ve unsafe primitive kontrollerinin PASS veya BLOCKED sonucunu saklar\n"
        "# Bagimli Oldugu Katman: Tool | View\n\n"
    )
    SUMMARY_PATH.write_text(header + yaml.safe_dump(payload, sort_keys=False), encoding="utf-8")


def main() -> int:
    checks: list[str] = []
    try:
        config = load_runtime_config()
        checks.extend(validate_headers(config))
        checks.extend(validate_layer_imports(config))
        checks.extend(validate_status_localization(config))
        checks.extend(validate_magic_status_strings(config))
        checks.extend(validate_runtime_primitives(config))
        checks.extend(validate_release_policy(config))
    except (RuntimeConfigError, RuntimeGateError, SyntaxError) as exc:
        write_summary("BLOCKED", checks, str(exc))
        print(str(exc), file=sys.stderr)
        return 2

    write_summary("PASS", checks)
    print(f"{config.codes['pass']} PASS ({len(checks)} static runtime checks)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
