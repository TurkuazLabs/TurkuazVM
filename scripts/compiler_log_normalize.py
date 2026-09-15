# 📄 Dosya Yolu: /turkuazvm/scripts/compiler_log_normalize.py
# 📌 Amac: Raw Cargo JSON compiler kanitini deterministic ve dogrulanabilir hata kayitlarina donusturur
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Missing raw log ve bozuk metadata durumlarini sifir hata gibi yorumlamaz; fingerprint ortak policy ile uretilir
# Bagimli Oldugu Katman: Tool

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

from compiler_gate_policy import (
    PolicyError,
    artifact_value,
    collapse_space,
    ensure_platform,
    fingerprint_for,
    load_policy,
    normalize_path,
    status,
    strip_ansi,
)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--platform", required=True)
    parser.add_argument("--input-dir", default=None)
    parser.add_argument("--output-dir", default=None)
    parser.add_argument("--config", default=None)
    args = parser.parse_args()

    root = Path(__file__).resolve().parents[1]
    try:
        policy = load_policy(root, args.config)
        ensure_platform(policy, args.platform)
    except PolicyError as error:
        print(f"COMPILER NORMALIZER: BLOCKED {error.code} {error.detail}")
        return 2

    input_root = Path(args.input_dir) if args.input_dir else Path(artifact_value(policy, "raw_root"))
    output_root = Path(args.output_dir) if args.output_dir else Path(artifact_value(policy, "normalized_root"))
    if not input_root.is_absolute():
        input_root = root / input_root
    if not output_root.is_absolute():
        output_root = root / output_root

    input_dir = input_root / args.platform
    output_dir = output_root / args.platform
    output_dir.mkdir(parents=True, exist_ok=True)

    raw_path = input_dir / artifact_value(policy, "raw_log")
    meta_path = input_dir / artifact_value(policy, "collector_metadata")
    errors_path = output_dir / artifact_value(policy, "normalized_errors")
    summary_path = output_dir / artifact_value(policy, "normalized_summary")

    if not meta_path.is_file():
        return write_blocked(policy, args.platform, errors_path, summary_path, "metadata_missing", "metadata missing")
    try:
        meta = json.loads(meta_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        return write_blocked(policy, args.platform, errors_path, summary_path, "metadata_invalid", str(error))
    if not isinstance(meta, dict) or meta.get("platform") != args.platform or not isinstance(meta.get("status_code"), str):
        return write_blocked(policy, args.platform, errors_path, summary_path, "metadata_invalid", "metadata identity/status invalid")
    if not raw_path.is_file():
        return write_blocked(policy, args.platform, errors_path, summary_path, "raw_log_missing", "raw log missing", meta)

    errors: dict[str, dict[str, Any]] = {}
    malformed_lines = 0
    compiler_messages = 0
    raw_records = 0

    try:
        raw_text = raw_path.read_text(encoding="utf-8", errors="replace")
    except OSError as error:
        return write_blocked(policy, args.platform, errors_path, summary_path, "raw_log_missing", str(error), meta)

    for raw_line in raw_text.splitlines():
        if not raw_line.strip():
            continue
        try:
            record = json.loads(raw_line)
        except json.JSONDecodeError:
            malformed_lines += 1
            continue
        raw_records += 1
        if record.get("reason") != "compiler-message":
            continue
        compiler_messages += 1
        message = record.get("message") or {}
        if message.get("level") != "error":
            continue
        normalized = normalize_error(policy, record, args.platform, root)
        errors.setdefault(normalized["fingerprint"], normalized)

    ordered = sorted(
        errors.values(),
        key=lambda item: (
            item.get("code") or "",
            item.get("file") or "",
            item.get("line") or 0,
            item.get("column") or 0,
            item.get("message") or "",
        ),
    )
    by_code: dict[str, int] = {}
    by_file: dict[str, int] = {}
    for item in ordered:
        code = item.get("code") or "NO_CODE"
        by_code[code] = by_code.get(code, 0) + 1
        file_name = item.get("file") or "<unknown>"
        by_file[file_name] = by_file.get(file_name, 0) + 1

    status_code = status(policy, "invalid_json") if malformed_lines else status(policy, "pass")
    summary = {
        "schema": policy["validation"]["normalized_schema"],
        "policy_version": policy.get("_version"),
        "platform": args.platform,
        "status_code": status_code,
        "collector_status_code": meta.get("status_code"),
        "cargo_exit_code": meta.get("cargo_exit_code"),
        "cargo_version": meta.get("cargo_version"),
        "rustc_version": meta.get("rustc_version"),
        "git_head": meta.get("git_head"),
        "expected_commit": meta.get("expected_commit"),
        "host_os": meta.get("host_os"),
        "expected_host_os": meta.get("expected_host_os"),
        "configured_rust_toolchain": policy["runtime"].get("rust_toolchain"),
        "raw_present": True,
        "raw_records": raw_records,
        "compiler_messages": compiler_messages,
        "malformed_json_lines": malformed_lines,
        "unique_errors": len(ordered),
        "by_code": dict(sorted(by_code.items())),
        "by_file": dict(sorted(by_file.items())),
    }
    write_json(errors_path, ordered)
    write_json(summary_path, summary)
    print(
        f"COMPILER NORMALIZER: platform={args.platform} unique_errors={len(ordered)} "
        f"malformed={malformed_lines} cargo_exit={meta.get('cargo_exit_code')}"
    )
    return 0 if malformed_lines == 0 else 2


def normalize_error(policy: dict[str, Any], record: dict[str, Any], platform_name: str, root: Path) -> dict[str, Any]:
    message = record.get("message") or {}
    spans = message.get("spans") or []
    primary = next((span for span in spans if span.get("is_primary")), spans[0] if spans else {})
    code_obj = message.get("code") or {}
    target = record.get("target") or {}
    item = {
        "platform": platform_name,
        "code": code_obj.get("code"),
        "message": collapse_space(message.get("message") or ""),
        "file": normalize_path(primary.get("file_name"), root),
        "line": primary.get("line_start"),
        "column": primary.get("column_start"),
        "crate": target.get("name"),
        "package_id": record.get("package_id"),
        "rendered": strip_ansi(message.get("rendered") or "").strip() or None,
    }
    item["fingerprint"] = fingerprint_for(policy, item)
    return item


def write_blocked(
    policy: dict[str, Any],
    platform_name: str,
    errors_path: Path,
    summary_path: Path,
    status_key: str,
    detail: str,
    meta: dict[str, Any] | None = None,
) -> int:
    summary = {
        "schema": policy["validation"]["normalized_schema"],
        "policy_version": policy.get("_version"),
        "platform": platform_name,
        "status_code": status(policy, status_key),
        "collector_status_code": (meta or {}).get("status_code"),
        "cargo_exit_code": (meta or {}).get("cargo_exit_code"),
        "cargo_version": (meta or {}).get("cargo_version"),
        "rustc_version": (meta or {}).get("rustc_version"),
        "git_head": (meta or {}).get("git_head"),
        "expected_commit": (meta or {}).get("expected_commit"),
        "host_os": (meta or {}).get("host_os"),
        "expected_host_os": (meta or {}).get("expected_host_os"),
        "configured_rust_toolchain": policy["runtime"].get("rust_toolchain"),
        "raw_present": False,
        "raw_records": 0,
        "compiler_messages": 0,
        "malformed_json_lines": 0,
        "unique_errors": 0,
        "by_code": {},
        "by_file": {},
        "detail": detail,
    }
    write_json(errors_path, [])
    write_json(summary_path, summary)
    print(f"COMPILER NORMALIZER: BLOCKED {summary['status_code']} {detail}")
    return 2


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


if __name__ == "__main__":
    raise SystemExit(main())
