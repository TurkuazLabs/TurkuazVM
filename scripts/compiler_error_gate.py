# 📄 Dosya Yolu: /turkuazvm/scripts/compiler_error_gate.py
# 📌 Amac: Platform compiler artifactlarini butunluk, provenance ve sifir-hata politikasi ile release gate'e baglar
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Cargo nonzero, missing raw, bozuk JSON, sahte fingerprint, platform/provenance/toolchain uyusmazliginda fail-closed karar verir
# Bagimli Oldugu Katman: Tool | View

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

from compiler_gate_policy import (
    BOOTSTRAP_CONFIG_INVALID,
    PolicyError,
    artifact_value,
    expected_host_os,
    fingerprint_for,
    load_policy,
    required_platforms,
    status,
    validation_value,
)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input-dir", default=None)
    parser.add_argument("--output", default=None)
    parser.add_argument("--expected-git-head", default=None)
    parser.add_argument("--policy-job-result", default=None)
    parser.add_argument("--collect-job-result", default=None)
    parser.add_argument("--config", default=None)
    args = parser.parse_args()

    root = Path(__file__).resolve().parents[1]
    try:
        policy = load_policy(root, args.config)
    except PolicyError as error:
        output = Path(args.output) if args.output else root / "artifacts/compiler-gate-summary.json"
        if not output.is_absolute():
            output = root / output
        write_json(output, bootstrap_failure(error.code, error.detail))
        print(f"COMPILER ERROR GATE: BLOCKED {error.code} {error.detail}")
        return 1

    input_dir = Path(args.input_dir) if args.input_dir else Path(artifact_value(policy, "normalized_root"))
    output = Path(args.output) if args.output else Path(artifact_value(policy, "gate_summary"))
    if not input_dir.is_absolute():
        input_dir = root / input_dir
    if not output.is_absolute():
        output = root / output

    blockers: list[dict[str, str]] = []
    validate_upstream_results(policy, blockers, args.policy_job_result, args.collect_job_result)
    platform_summaries: dict[str, dict[str, Any]] = {}
    combined_errors: dict[str, dict[str, Any]] = {}
    pass_code = status(policy, "pass")

    for platform_name in required_platforms(policy):
        summary_path = input_dir / platform_name / artifact_value(policy, "normalized_summary")
        errors_path = input_dir / platform_name / artifact_value(policy, "normalized_errors")
        if not summary_path.is_file() or not errors_path.is_file():
            add_blocker(blockers, status(policy, "platform_missing"), platform_name)
            continue

        summary = read_json_dict(summary_path)
        if summary is None:
            add_blocker(blockers, status(policy, "normalized_summary_invalid"), platform_name)
            continue
        platform_summaries[platform_name] = summary

        validate_summary(policy, platform_name, summary, blockers, pass_code)
        errors = read_json_list(errors_path)
        if errors is None:
            add_blocker(blockers, status(policy, "errors_payload_invalid"), platform_name)
            continue
        if summary.get("unique_errors") != len(errors):
            add_blocker(blockers, status(policy, "errors_payload_invalid"), f"{platform_name}:count")

        for error in errors:
            if not isinstance(error, dict):
                add_blocker(blockers, status(policy, "errors_payload_invalid"), f"{platform_name}:item")
                continue
            if not valid_error_payload(error):
                add_blocker(blockers, status(policy, "errors_payload_invalid"), f"{platform_name}:schema")
                continue
            expected_fingerprint = fingerprint_for(policy, error)
            actual_fingerprint = error.get("fingerprint")
            if actual_fingerprint != expected_fingerprint:
                add_blocker(blockers, status(policy, "fingerprint_invalid"), f"{platform_name}:{actual_fingerprint}")
                continue
            existing = combined_errors.get(expected_fingerprint)
            if existing is None:
                combined_errors[expected_fingerprint] = {**error, "platforms": [platform_name]}
            else:
                if canonical_error(policy, existing) != canonical_error(policy, error):
                    add_blocker(blockers, status(policy, "fingerprint_invalid"), f"collision:{expected_fingerprint}")
                    continue
                if platform_name not in existing["platforms"]:
                    existing["platforms"].append(platform_name)
                    existing["platforms"].sort()

    validate_cross_platform(policy, platform_summaries, blockers, args.expected_git_head)

    max_errors = int(validation_value(policy, "max_unique_errors"))
    if len(combined_errors) > max_errors:
        add_blocker(blockers, status(policy, "compiler_errors_present"), str(len(combined_errors)))

    result = {
        "schema": 2,
        "policy_version": policy.get("_version"),
        "status_code": blockers[0]["code"] if blockers else pass_code,
        "required_platforms": required_platforms(policy),
        "platforms_present": sorted(platform_summaries),
        "expected_git_head": args.expected_git_head,
        "unique_errors": len(combined_errors),
        "blockers": blockers,
        "errors": sorted(
            combined_errors.values(),
            key=lambda item: (
                item.get("code") or "",
                item.get("file") or "",
                item.get("line") or 0,
                item.get("column") or 0,
            ),
        ),
    }
    write_json(output, result)

    if blockers:
        print(f"COMPILER ERROR GATE: BLOCKED {result['status_code']}")
        for blocker in blockers:
            print(f" - {blocker['code']} {blocker['detail']}")
        for error in result["errors"][:50]:
            location = f"{error.get('file')}:{error.get('line')}:{error.get('column')}"
            print(f" - {error.get('code') or 'NO_CODE'} {location} {error.get('message')}")
        return 1

    print("COMPILER ERROR GATE: PASS unique_errors=0")
    return 0


def validate_summary(
    policy: dict[str, Any],
    platform_name: str,
    summary: dict[str, Any],
    blockers: list[dict[str, str]],
    pass_code: str,
) -> None:
    expected_schema = validation_value(policy, "normalized_schema")
    if summary.get("schema") != expected_schema or summary.get("status_code") != pass_code:
        add_blocker(blockers, status(policy, "normalized_summary_invalid"), f"{platform_name}:{summary.get('status_code')}")
    if validation_value(policy, "require_platform_identity"):
        if summary.get("platform") != platform_name:
            add_blocker(blockers, status(policy, "platform_identity_mismatch"), f"{platform_name}:{summary.get('platform')}")
        if summary.get("host_os") != expected_host_os(policy, platform_name):
            add_blocker(blockers, status(policy, "platform_identity_mismatch"), f"{platform_name}:{summary.get('host_os')}")
        if summary.get("expected_host_os") != expected_host_os(policy, platform_name):
            add_blocker(blockers, status(policy, "platform_identity_mismatch"), f"{platform_name}:expected-host")
    if validation_value(policy, "require_raw_log") and summary.get("raw_present") is not True:
        add_blocker(blockers, status(policy, "normalized_summary_invalid"), f"{platform_name}:raw")
    if validation_value(policy, "require_nonempty_raw_when_cargo_success") and summary.get("cargo_exit_code") == 0:
        raw_records = summary.get("raw_records")
        if not isinstance(raw_records, int) or raw_records <= 0:
            add_blocker(blockers, status(policy, "normalized_summary_invalid"), f"{platform_name}:empty-raw")
    if validation_value(policy, "require_valid_json") and summary.get("malformed_json_lines") != 0:
        add_blocker(blockers, status(policy, "normalized_summary_invalid"), f"{platform_name}:json")
    if validation_value(policy, "require_collector_status_pass") and summary.get("collector_status_code") != pass_code:
        add_blocker(blockers, status(policy, "collector_failed"), f"{platform_name}:{summary.get('collector_status_code')}")
    if validation_value(policy, "require_cargo_exit_zero") and summary.get("cargo_exit_code") != 0:
        add_blocker(blockers, status(policy, "cargo_failed"), f"{platform_name}:{summary.get('cargo_exit_code')}")
    if validation_value(policy, "require_configured_rust_toolchain"):
        configured = str(policy["runtime"].get("rust_toolchain") or "")
        rustc_version = summary.get("rustc_version")
        if not isinstance(rustc_version, str) or not rustc_version.startswith(f"rustc {configured} "):
            add_blocker(blockers, status(policy, "toolchain_mismatch"), f"{platform_name}:configured-rustc")



def validate_upstream_results(
    policy: dict[str, Any],
    blockers: list[dict[str, str]],
    policy_job_result: str | None,
    collect_job_result: str | None,
) -> None:
    if not validation_value(policy, "require_upstream_jobs_success"):
        return
    for job_name, result in (("policy", policy_job_result), ("collect", collect_job_result)):
        if result is not None and result != "success":
            add_blocker(blockers, status(policy, "upstream_job_failed"), f"{job_name}:{result}")


def valid_error_payload(error: dict[str, Any]) -> bool:
    if not isinstance(error.get("fingerprint"), str):
        return False
    if not isinstance(error.get("message"), str):
        return False
    for key in ("code", "file", "crate"):
        if error.get(key) is not None and not isinstance(error.get(key), str):
            return False
    for key in ("line", "column"):
        if error.get(key) is not None and not isinstance(error.get(key), int):
            return False
    return True

def validate_cross_platform(
    policy: dict[str, Any],
    summaries: dict[str, dict[str, Any]],
    blockers: list[dict[str, str]],
    expected_git_head: str | None,
) -> None:
    if not summaries:
        return
    if validation_value(policy, "require_same_git_head"):
        heads = {summary.get("git_head") for summary in summaries.values() if summary.get("git_head")}
        missing_head = any(not summary.get("git_head") for summary in summaries.values())
        if missing_head or len(heads) != 1:
            add_blocker(blockers, status(policy, "provenance_mismatch"), "cross-platform-git-head")
        elif expected_git_head and next(iter(heads)) != expected_git_head:
            add_blocker(blockers, status(policy, "provenance_mismatch"), "expected-git-head")
        expected_commits = {summary.get("expected_commit") for summary in summaries.values() if summary.get("expected_commit")}
        if len(expected_commits) > 1:
            add_blocker(blockers, status(policy, "provenance_mismatch"), "collector-expected-commit")
        if expected_git_head and expected_commits and expected_commits != {expected_git_head}:
            add_blocker(blockers, status(policy, "provenance_mismatch"), "github-sha")
    if validation_value(policy, "require_same_rustc_version"):
        versions = {summary.get("rustc_version") for summary in summaries.values() if summary.get("rustc_version")}
        missing_version = any(not summary.get("rustc_version") for summary in summaries.values())
        if missing_version or len(versions) != 1:
            add_blocker(blockers, status(policy, "toolchain_mismatch"), "rustc-version")


def canonical_error(policy: dict[str, Any], error: dict[str, Any]) -> tuple[Any, ...]:
    fields = validation_value(policy, "fingerprint_fields")
    return tuple(error.get(field) for field in fields)


def read_json_dict(path: Path) -> dict[str, Any] | None:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None
    return value if isinstance(value, dict) else None


def read_json_list(path: Path) -> list[Any] | None:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None
    return value if isinstance(value, list) else None


def add_blocker(blockers: list[dict[str, str]], code: str, detail: str) -> None:
    item = {"code": code, "detail": detail}
    if item not in blockers:
        blockers.append(item)


def bootstrap_failure(code: str, detail: str) -> dict[str, Any]:
    return {
        "schema": 2,
        "status_code": code or BOOTSTRAP_CONFIG_INVALID,
        "unique_errors": 0,
        "blockers": [{"code": code or BOOTSTRAP_CONFIG_INVALID, "detail": detail}],
        "errors": [],
    }


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


if __name__ == "__main__":
    raise SystemExit(main())
