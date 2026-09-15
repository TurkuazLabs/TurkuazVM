# 📄 Dosya Yolu: /turkuazvm/scripts/compiler_log_collect.py
# 📌 Amac: Cargo JSON compiler kanitini config kontrollu metadata ve provenance ile toplar
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Host kimligi, Git commit, toolchain ve Cargo exit-code bilgisini raw logdan ayirmadan fail-closed saklar
# Bagimli Oldugu Katman: Tool

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import time

from compiler_gate_policy import (
    PolicyError,
    actual_host_os,
    artifact_value,
    cargo_command,
    ensure_platform,
    expected_host_os,
    git_head,
    load_policy,
    status,
)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--platform", required=True)
    parser.add_argument("--output-dir", default=None)
    parser.add_argument("--target", default="")
    parser.add_argument("--config", default=None)
    args = parser.parse_args()

    root = Path(__file__).resolve().parents[1]
    try:
        policy = load_policy(root, args.config)
        ensure_platform(policy, args.platform)
    except PolicyError as error:
        print(f"COMPILER COLLECTOR: BLOCKED {error.code} {error.detail}")
        return 2

    output_root = Path(args.output_dir) if args.output_dir else Path(artifact_value(policy, "raw_root"))
    if not output_root.is_absolute():
        output_root = root / output_root
    output_dir = output_root / args.platform
    output_dir.mkdir(parents=True, exist_ok=True)

    raw_path = output_dir / artifact_value(policy, "raw_log")
    stderr_path = output_dir / artifact_value(policy, "stderr_log")
    meta_path = output_dir / artifact_value(policy, "collector_metadata")

    cargo = shutil.which("cargo")
    rustc = shutil.which("rustc")
    host_os = actual_host_os()
    current_head = git_head(root)
    expected_commit = os.environ.get("GITHUB_SHA") or None

    metadata = {
        "schema": 2,
        "policy_version": policy.get("_version"),
        "platform": args.platform,
        "host_os": host_os,
        "expected_host_os": expected_host_os(policy, args.platform),
        "target": args.target or None,
        "started_unix": int(time.time()),
        "cargo_available": bool(cargo),
        "rustc_available": bool(rustc),
        "status_code": status(policy, "pass"),
        "cargo_exit_code": None,
        "cargo_version": None,
        "rustc_version": None,
        "git_head": current_head,
        "expected_commit": expected_commit,
        "command": None,
    }

    if host_os != metadata["expected_host_os"]:
        return block_before_run(policy, metadata, meta_path, raw_path, stderr_path, "host_mismatch", host_os)
    if not current_head:
        return block_before_run(policy, metadata, meta_path, raw_path, stderr_path, "provenance_unavailable", "git HEAD missing")
    if expected_commit and expected_commit != current_head:
        return block_before_run(policy, metadata, meta_path, raw_path, stderr_path, "provenance_unavailable", "GITHUB_SHA mismatch")
    if not cargo:
        return block_before_run(policy, metadata, meta_path, raw_path, stderr_path, "cargo_missing", "cargo missing")
    if not rustc:
        return block_before_run(policy, metadata, meta_path, raw_path, stderr_path, "rustc_missing", "rustc missing")

    metadata["cargo_version"] = tool_version(cargo, root)
    metadata["rustc_version"] = tool_version(rustc, root)
    command = cargo_command(policy, cargo, args.target or None)
    metadata["command"] = command

    try:
        with (
            raw_path.open("w", encoding="utf-8", newline="\n") as output,
            stderr_path.open("w", encoding="utf-8", newline="\n") as stderr_output,
        ):
            process = subprocess.Popen(
                command,
                cwd=root,
                text=True,
                stdout=subprocess.PIPE,
                stderr=stderr_output,
            )
            assert process.stdout is not None
            for line in process.stdout:
                output.write(line)
                output.flush()
                try:
                    record = json.loads(line)
                    if record.get("reason") == "compiler-message":
                        message = record.get("message") or {}
                        rendered = message.get("rendered")
                        if rendered:
                            print(rendered.rstrip())
                except json.JSONDecodeError:
                    print(line.rstrip())
            exit_code = process.wait()
    except OSError as error:
        metadata["status_code"] = status(policy, "log_write_failed")
        metadata["error"] = str(error)
        safe_write_json(meta_path, metadata)
        print(f"COMPILER COLLECTOR: BLOCKED {metadata['status_code']} {error}")
        return 2

    metadata["cargo_exit_code"] = exit_code
    metadata["completed_unix"] = int(time.time())
    if exit_code != 0:
        metadata["status_code"] = status(policy, "cargo_failed_with_log")

    safe_write_json(meta_path, metadata)
    print(
        "COMPILER COLLECTOR: "
        + ("PASS" if exit_code == 0 else "CAPTURED_FAILURE")
        + f" platform={args.platform} cargo_exit={exit_code} git_head={current_head}"
    )
    return 0


def block_before_run(
    policy: dict,
    metadata: dict,
    meta_path: Path,
    raw_path: Path,
    stderr_path: Path,
    status_key: str,
    detail: str,
) -> int:
    metadata["status_code"] = status(policy, status_key)
    metadata["error"] = detail
    raw_path.write_text("", encoding="utf-8")
    stderr_path.write_text("", encoding="utf-8")
    safe_write_json(meta_path, metadata)
    print(f"COMPILER COLLECTOR: BLOCKED {metadata['status_code']} {detail}")
    return 2


def tool_version(binary: str, root: Path) -> str:
    result = subprocess.run(
        [binary, "--version"],
        cwd=root,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    return result.stdout.strip()


def safe_write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


if __name__ == "__main__":
    raise SystemExit(main())
