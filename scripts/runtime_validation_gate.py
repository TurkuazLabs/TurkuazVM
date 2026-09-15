# 📄 Dosya Yolu: /turkuazvm/scripts/runtime_validation_gate.py
# 📌 Amac: Guncel runtime validation harness config, katman, qemu-img ve language kontratini statik olarak fail-closed dogrular
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Tarihsel K-L release artefactlarina baglanmadan aktif runtime invariantlarini ve katman sinirlarini dogrular
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

from tools.runtime_validation_config import load_runtime_validation_config

SUMMARY_PATH = ROOT / "artifacts" / "runtime-validation" / "runtime-validation-static-summary.yml"
HEADER_PREFIXES = (
    "# 📄 Dosya Yolu:",
    "# 📌 Amac:",
    "# 📌 Modul -",
    "# Version:",
    "# Aciklama:",
    "# Bagimli Oldugu Katman:",
)
CURRENT_RUNTIME_FILES = (
    "config/runtime-validation.yml",
    "config/requirements-runtime-validation.txt",
    "controllers/runtime_validation_controller.py",
    "services/runtime_validation_service.py",
    "repositories/runtime_validation_repository.py",
    "tools/runtime_validation_config.py",
    "tools/runtime_validation_probe_tool.py",
    "tools/runtime_validation_fixture_tool.py",
    "tools/qemu_runtime_probe_tool.py",
    "views/runtime_validation_view.py",
    "scripts/runtime_validation_run.py",
    "scripts/runtime_validation_gate.py",
    "scripts/tests/test_runtime_validation_harness.py",
    ".github/workflows/runtime-validation-gate.yml",
    "docs/VALIDATION.md",
)


class RuntimeValidationGateError(RuntimeError):
    pass


def require(condition: bool, code: str, message: str) -> None:
    if not condition:
        raise RuntimeValidationGateError(f"{code} {message}")


def validate_headers(config: Any) -> list[str]:
    for relative in CURRENT_RUNTIME_FILES:
        path = ROOT / relative
        require(path.is_file(), config.codes["config_missing"], f"required L file missing: {relative}")
        raw = path.read_bytes()
        require(b"\x00" not in raw, config.codes["config_invalid"], f"NUL byte found: {relative}")
        lines = raw.decode("utf-8").splitlines()
        for index, prefix in enumerate(HEADER_PREFIXES):
            require(index < len(lines) and lines[index].startswith(prefix), config.codes["config_invalid"], f"header invalid: {relative}:{index + 1}")
    return ["technical headers", "UTF-8/NUL integrity"]


def validate_controller_boundary(config: Any) -> list[str]:
    path = ROOT / "controllers" / "runtime_validation_controller.py"
    tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
    for node in ast.walk(tree):
        names: list[str] = []
        if isinstance(node, ast.ImportFrom) and node.module:
            names.append(node.module.split(".", 1)[0])
        elif isinstance(node, ast.Import):
            names.extend(alias.name.split(".", 1)[0] for alias in node.names)
        for name in names:
            require(name not in {"tools", "repositories", "views"}, config.codes["config_invalid"], f"Controller layer violation: {name}")
    return ["Controller -> Service boundary"]


def validate_qemu_img_hotfix(config: Any) -> list[str]:
    tool_path = ROOT / "tools" / "qemu_img_tool.py"
    text = tool_path.read_text(encoding="utf-8")
    require('"-B"' not in text and "'-B'" not in text, config.codes["config_invalid"], "legacy qemu-img create -B option detected")
    runtime = yaml.safe_load((ROOT / "config" / "host-resilience-runtime.yml").read_text(encoding="utf-8"))
    image_cfg = runtime.get("runtime", {}).get("image", {})
    require(image_cfg.get("backing_format_option") == "-F", config.codes["config_invalid"], "qemu-img backing format option must be -F")
    require(image_cfg.get("backing_file_option") == "-b", config.codes["config_invalid"], "qemu-img backing file option must be -b")
    return ["qemu-img create -F/-b contract"]


def validate_required_test_coverage(config: Any) -> list[str]:
    production = set(str(item) for item in config.validation["required_tests"]["production"])
    smoke = set(str(item) for item in config.validation["required_tests"]["harness_smoke"])
    fixture = set(str(item) for item in config.validation["fixture_requirements"])
    require(smoke.issubset(production), config.codes["config_invalid"], "smoke tests outside production list")
    require(production - smoke == fixture, config.codes["config_invalid"], "fixture requirements do not cover production-only tests")
    return ["production test coverage", "fixture coverage"]


def validate_language(config: Any) -> list[str]:
    codes = set(str(value) for value in config.codes.values())
    for name in ("tr.yml", "en.yml"):
        payload = yaml.safe_load((ROOT / "language" / name).read_text(encoding="utf-8"))
        require(isinstance(payload, dict), config.codes["config_invalid"], f"language invalid: {name}")
        missing = sorted(codes - set(payload))
        require(not missing, config.codes["config_invalid"], f"language codes missing in {name}: {missing}")
    return ["TR/EN status localization"]


def validate_subprocess_safety(config: Any) -> list[str]:
    paths = [ROOT / relative for relative in CURRENT_RUNTIME_FILES if relative.endswith(".py")]
    paths.append(ROOT / "tools" / "qemu_img_tool.py")
    for path in paths:
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
        for node in ast.walk(tree):
            if not isinstance(node, ast.Call):
                continue
            for keyword in node.keywords:
                if keyword.arg != "shell":
                    continue
                is_true = isinstance(keyword.value, ast.Constant) and keyword.value.value is True
                require(not is_true, config.codes["config_invalid"], f"unsafe subprocess shell mode: {path.relative_to(ROOT)}")
    return ["shell-disabled command boundary"]



def validate_current_release_artifacts(config: Any) -> list[str]:
    cargo_text = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    workspace_version = None
    in_workspace_package = False
    for raw in cargo_text.splitlines():
        line = raw.strip()
        if line == "[workspace.package]":
            in_workspace_package = True
            continue
        if in_workspace_package and line.startswith("["):
            break
        if in_workspace_package and line.startswith("version") and "=" in line:
            workspace_version = line.split("=", 1)[1].strip().strip('"')
            break
    require(bool(workspace_version), config.codes["config_invalid"], "workspace version missing")
    release_status = ROOT / f"RELEASE_STATUS_v{workspace_version}.yml"
    require(release_status.is_file(), config.codes["config_missing"], f"current release status missing: {release_status.name}")
    payload = yaml.safe_load(release_status.read_text(encoding="utf-8"))
    require(isinstance(payload, dict), config.codes["config_invalid"], "current release status invalid")
    require(str(payload.get("version")) == str(workspace_version), config.codes["config_invalid"], "release status version drift")
    return ["current release metadata"]


def write_summary(config: Any, checks: list[str]) -> None:
    SUMMARY_PATH.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema": 1,
        "version": project_version(),
        "status": config.statuses["pass"],
        "code": config.codes["pass"],
        "checks": checks,
    }
    header = (
        "# 📄 Dosya Yolu: /turkuazvm/artifacts/runtime-validation/runtime-validation-static-summary.yml\n"
        "# 📌 Amac: Guncel runtime validation static gate sonucunu makine-okunur olarak saklar\n"
        "# 📌 Modul - YAML\n"
        f"# Version: {project_version()}\n"
        "# Aciklama: Aktif katman, config, qemu-img ve release metadata kontrollerinin evidence artifactidir\n"
        "# Bagimli Oldugu Katman: Tool | View\n\n"
    )
    SUMMARY_PATH.write_text(header + yaml.safe_dump(payload, sort_keys=False, allow_unicode=False), encoding="utf-8")


def main() -> int:
    config = load_runtime_validation_config()
    checks: list[str] = []
    for validator in (
        validate_headers,
        validate_controller_boundary,
        validate_qemu_img_hotfix,
        validate_required_test_coverage,
        validate_language,
        validate_subprocess_safety,
        validate_current_release_artifacts,
    ):
        checks.extend(validator(config))
    write_summary(config, checks)
    print(f"{config.codes['pass']} runtime validation static gate PASS ({len(checks)} checks)")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except RuntimeValidationGateError as exc:
        print(str(exc), file=sys.stderr)
        raise SystemExit(1)
