# 📄 Dosya Yolu: /turkuazvm/scripts/compiler_gate_policy.py
# 📌 Amac: Compiler gate YAML politikasini yukler, dogrular ve ortak runtime fonksiyonlarini saglar
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Status, platform, artifact, fingerprint ve GitHub matrix degerleri icin tek kaynak adaptorudur
# Bagimli Oldugu Katman: Tool

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform as host_platform
import re
import subprocess
from typing import Any

import yaml

BOOTSTRAP_CONFIG_MISSING = "CI-I-101"
BOOTSTRAP_CONFIG_INVALID = "CI-I-102"
DEFAULT_CONFIG_PATH = Path("config/compiler-log-gate.yml")


class PolicyError(RuntimeError):
    def __init__(self, code: str, detail: str) -> None:
        super().__init__(detail)
        self.code = code
        self.detail = detail


def load_policy(root: Path, config_path: str | Path | None = None) -> dict[str, Any]:
    path = Path(config_path) if config_path else DEFAULT_CONFIG_PATH
    if not path.is_absolute():
        path = root / path
    if not path.is_file():
        raise PolicyError(BOOTSTRAP_CONFIG_MISSING, str(path))
    try:
        raw = yaml.safe_load(path.read_text(encoding="utf-8"))
    except (OSError, yaml.YAMLError) as error:
        raise PolicyError(BOOTSTRAP_CONFIG_INVALID, str(error)) from error
    try:
        gate = raw["compiler_log_gate"]
        required = gate["required_platforms"]
        platforms = gate["platforms"]
        statuses = gate["status_codes"]
        artifacts = gate["artifacts"]
        validation = gate["validation"]
        runtime = gate["runtime"]
        cargo_command = gate["cargo_command"]
    except (TypeError, KeyError) as error:
        raise PolicyError(BOOTSTRAP_CONFIG_INVALID, f"missing key: {error}") from error

    if not isinstance(required, list) or not required or not all(isinstance(item, str) for item in required):
        raise PolicyError(BOOTSTRAP_CONFIG_INVALID, "required_platforms invalid")
    if len(set(required)) != len(required):
        raise PolicyError(BOOTSTRAP_CONFIG_INVALID, "required_platforms duplicate")
    if not isinstance(platforms, dict) or any(item not in platforms for item in required):
        raise PolicyError(BOOTSTRAP_CONFIG_INVALID, "platform map invalid")
    if not isinstance(statuses, dict) or statuses.get("pass") is None:
        raise PolicyError(BOOTSTRAP_CONFIG_INVALID, "status_codes invalid")
    if not isinstance(artifacts, dict) or not isinstance(validation, dict) or not isinstance(runtime, dict):
        raise PolicyError(BOOTSTRAP_CONFIG_INVALID, "policy section invalid")
    if not isinstance(cargo_command, list) or not cargo_command or not all(isinstance(item, str) for item in cargo_command):
        raise PolicyError(BOOTSTRAP_CONFIG_INVALID, "cargo_command invalid")

    gate["_schema"] = raw.get("schema")
    gate["_version"] = raw.get("version")
    return gate


def status(policy: dict[str, Any], key: str) -> str:
    try:
        value = policy["status_codes"][key]
    except KeyError as error:
        raise PolicyError(BOOTSTRAP_CONFIG_INVALID, f"status code missing: {key}") from error
    if not isinstance(value, str) or not value:
        raise PolicyError(BOOTSTRAP_CONFIG_INVALID, f"status code invalid: {key}")
    return value


def required_platforms(policy: dict[str, Any]) -> list[str]:
    return list(policy["required_platforms"])


def ensure_platform(policy: dict[str, Any], platform_name: str) -> None:
    if platform_name not in required_platforms(policy):
        raise PolicyError(status(policy, "platform_invalid"), platform_name)


def expected_host_os(policy: dict[str, Any], platform_name: str) -> str:
    ensure_platform(policy, platform_name)
    value = policy["platforms"][platform_name].get("host_os")
    if not isinstance(value, str) or not value:
        raise PolicyError(BOOTSTRAP_CONFIG_INVALID, f"host_os invalid: {platform_name}")
    return value


def actual_host_os() -> str:
    return host_platform.system()


def artifact_value(policy: dict[str, Any], key: str) -> str:
    value = policy["artifacts"].get(key)
    if not isinstance(value, str) or not value:
        raise PolicyError(BOOTSTRAP_CONFIG_INVALID, f"artifact key invalid: {key}")
    return value


def validation_value(policy: dict[str, Any], key: str) -> Any:
    if key not in policy["validation"]:
        raise PolicyError(BOOTSTRAP_CONFIG_INVALID, f"validation key missing: {key}")
    return policy["validation"][key]


def runtime_value(policy: dict[str, Any], key: str) -> Any:
    if key not in policy["runtime"]:
        raise PolicyError(BOOTSTRAP_CONFIG_INVALID, f"runtime key missing: {key}")
    return policy["runtime"][key]


def cargo_command(policy: dict[str, Any], cargo_binary: str, target: str | None = None) -> list[str]:
    command = [cargo_binary, *policy["cargo_command"]]
    if target:
        command.extend(["--target", target])
    return command


def normalize_path(value: Any, root: Path) -> str | None:
    if not isinstance(value, str) or not value:
        return None
    normalized = re.sub(r"/+", "/", value.replace("\\", "/"))
    root_text = re.sub(r"/+", "/", str(root).replace("\\", "/")).rstrip("/") + "/"
    if normalized.lower().startswith(root_text.lower()):
        normalized = normalized[len(root_text):]
    match = re.search(r"(?:^|/)(apps|crates|scripts|config)/(.+)$", normalized, re.IGNORECASE)
    if match:
        normalized = match.group(1).lower() + "/" + match.group(2)
    return normalized


def collapse_space(value: str) -> str:
    return re.sub(r"\s+", " ", value).strip()


def strip_ansi(value: str) -> str:
    return re.sub(r"\x1b\[[0-9;]*m", "", value)


def fingerprint_for(policy: dict[str, Any], error: dict[str, Any]) -> str:
    fields = validation_value(policy, "fingerprint_fields")
    if not isinstance(fields, list) or not fields:
        raise PolicyError(BOOTSTRAP_CONFIG_INVALID, "fingerprint_fields invalid")
    values: list[str] = []
    for field in fields:
        value = error.get(field)
        if value is None:
            value = 0 if field in {"line", "column"} else "<unknown>"
        values.append(str(value))
    source = "|".join(values)
    return hashlib.sha256(source.encode("utf-8")).hexdigest()[:24]


def git_head(root: Path) -> str | None:
    try:
        result = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=root,
            check=False,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
        )
    except OSError:
        return None
    value = result.stdout.strip()
    return value if result.returncode == 0 and value else None


def github_matrix(policy: dict[str, Any]) -> dict[str, Any]:
    include = []
    for platform_name in required_platforms(policy):
        runner = policy["platforms"][platform_name].get("runner")
        if not isinstance(runner, str) or not runner:
            raise PolicyError(BOOTSTRAP_CONFIG_INVALID, f"runner invalid: {platform_name}")
        include.append({"platform": platform_name, "os": runner})
    return {"include": include}


def write_github_outputs(policy: dict[str, Any]) -> None:
    values = {
        "matrix": json.dumps(github_matrix(policy), separators=(",", ":")),
        "retention_days": str(runtime_value(policy, "artifact_retention_days")),
        "rust_toolchain": str(runtime_value(policy, "rust_toolchain")),
    }
    output_path = os.environ.get("GITHUB_OUTPUT")
    if output_path:
        with Path(output_path).open("a", encoding="utf-8", newline="\n") as handle:
            for key, value in values.items():
                handle.write(f"{key}={value}\n")
    else:
        for key, value in values.items():
            print(f"{key}={value}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("command", choices=["github-outputs", "validate"])
    parser.add_argument("--config", default=None)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    try:
        policy = load_policy(root, args.config)
        if args.command == "github-outputs":
            write_github_outputs(policy)
        else:
            print(f"COMPILER GATE POLICY: PASS version={policy.get('_version')}")
        return 0
    except PolicyError as error:
        print(f"COMPILER GATE POLICY: BLOCKED {error.code} {error.detail}")
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
