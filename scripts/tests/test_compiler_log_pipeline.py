# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_compiler_log_pipeline.py
# 📌 Amac: v0.13.2 compiler evidence fail-closed davranislarini regression testleriyle dogrular
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Missing raw, Cargo nonzero, fingerprint/platform/provenance uyusmazligi ve zero-error PASS senaryolarini test eder
# Bagimli Oldugu Katman: Tool

from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = ROOT / "scripts"
sys.path.insert(0, str(SCRIPTS))

from compiler_gate_policy import fingerprint_for, load_policy

POLICY = load_policy(ROOT)
PASS = POLICY["status_codes"]["pass"]


def load_module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


normalizer = load_module("compiler_log_normalize", SCRIPTS / "compiler_log_normalize.py")


def record(code: str, path: str, line: int, message: str) -> dict:
    return {
        "reason": "compiler-message",
        "package_id": "path+file:///repo#turkuazvm-engine@0.13.2",
        "target": {"name": "turkuazvm_engine"},
        "message": {
            "level": "error",
            "message": message,
            "code": {"code": code, "explanation": None},
            "spans": [{"file_name": path, "line_start": line, "column_start": 9, "is_primary": True}],
            "rendered": "\u001b[31merror\u001b[0m",
        },
    }


def make_error(code: str = "E0308") -> dict:
    error = {
        "code": code,
        "message": "mismatched types",
        "file": "apps/engine/src/tools/x.rs",
        "line": 42,
        "column": 9,
        "crate": "turkuazvm_engine",
        "package_id": "pkg",
        "rendered": "error",
    }
    error["fingerprint"] = fingerprint_for(POLICY, error)
    return error


def write_platform(
    root: Path,
    platform_name: str,
    errors: list[dict],
    *,
    collector_status: str = PASS,
    cargo_exit: int | None = 0,
    git_head: str = "test-head",
    rustc_version: str = f"rustc {POLICY['runtime']['rust_toolchain']} (test)",
    summary_status: str = PASS,
) -> None:
    path = root / platform_name
    path.mkdir(parents=True, exist_ok=True)
    host_os = POLICY["platforms"][platform_name]["host_os"]
    (path / POLICY["artifacts"]["normalized_summary"]).write_text(
        json.dumps({
            "schema": POLICY["validation"]["normalized_schema"],
            "policy_version": POLICY["_version"],
            "platform": platform_name,
            "status_code": summary_status,
            "collector_status_code": collector_status,
            "cargo_exit_code": cargo_exit,
            "cargo_version": "cargo test",
            "rustc_version": rustc_version,
            "configured_rust_toolchain": POLICY["runtime"]["rust_toolchain"],
            "git_head": git_head,
            "expected_commit": git_head,
            "host_os": host_os,
            "expected_host_os": host_os,
            "raw_present": True,
            "raw_records": 1,
            "compiler_messages": len(errors),
            "malformed_json_lines": 0,
            "unique_errors": len(errors),
            "by_code": {},
            "by_file": {},
        }),
        encoding="utf-8",
    )
    (path / POLICY["artifacts"]["normalized_errors"]).write_text(json.dumps(errors), encoding="utf-8")


def run_gate(
    normalized: Path,
    output: Path,
    expected_head: str = "test-head",
    policy_result: str | None = None,
    collect_result: str | None = None,
) -> subprocess.CompletedProcess[str]:
    command = [
            sys.executable,
            str(SCRIPTS / "compiler_error_gate.py"),
            "--input-dir",
            str(normalized),
            "--output",
            str(output),
            "--expected-git-head",
            expected_head,
        ]
    if policy_result is not None:
        command.extend(["--policy-job-result", policy_result])
    if collect_result is not None:
        command.extend(["--collect-job-result", collect_result])
    return subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )


def test_windows_path_normalization() -> None:
    item = normalizer.normalize_error(
        POLICY,
        record("E0308", r"E:\\repo\\apps\\engine\\src\\tools\\x.rs", 42, " mismatched   types "),
        "windows",
        Path(r"E:\\repo"),
    )
    assert item["file"] == "apps/engine/src/tools/x.rs"
    assert item["message"] == "mismatched types"


def test_same_error_has_same_fingerprint_across_platforms() -> None:
    win = normalizer.normalize_error(
        POLICY,
        record("E0425", r"C:\\repo\\crates\\guest\\src\\lib.rs", 7, "cannot find value `x`"),
        "windows",
        Path(r"C:\\repo"),
    )
    linux = normalizer.normalize_error(
        POLICY,
        record("E0425", "/home/runner/repo/crates/guest/src/lib.rs", 7, "cannot find value `x`"),
        "linux",
        Path("/home/runner/repo"),
    )
    assert win["fingerprint"] == linux["fingerprint"]


def test_missing_raw_log_is_blocked() -> None:
    script = SCRIPTS / "compiler_log_normalize.py"
    with tempfile.TemporaryDirectory() as temp:
        base = Path(temp)
        input_root = base / "raw"
        output_root = base / "normalized"
        platform_dir = input_root / "linux"
        platform_dir.mkdir(parents=True)
        (platform_dir / POLICY["artifacts"]["collector_metadata"]).write_text(
            json.dumps({"platform": "linux", "status_code": PASS, "cargo_exit_code": 0}),
            encoding="utf-8",
        )
        result = subprocess.run(
            [sys.executable, str(script), "--platform", "linux", "--input-dir", str(input_root), "--output-dir", str(output_root)],
            cwd=ROOT,
            check=False,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
        )
        assert result.returncode == 2, result.stdout
        summary = json.loads((output_root / "linux" / POLICY["artifacts"]["normalized_summary"]).read_text(encoding="utf-8"))
        assert summary["status_code"] == POLICY["status_codes"]["raw_log_missing"]
        assert summary["raw_present"] is False


def test_zero_error_gate_passes() -> None:
    with tempfile.TemporaryDirectory() as temp:
        base = Path(temp)
        normalized = base / "normalized"
        write_platform(normalized, "linux", [])
        write_platform(normalized, "windows", [])
        output = base / "summary.json"
        result = run_gate(normalized, output, policy_result="success", collect_result="success")
        assert result.returncode == 0, result.stdout
        assert json.loads(output.read_text(encoding="utf-8"))["status_code"] == PASS


def test_cargo_nonzero_with_zero_compiler_errors_is_blocked() -> None:
    with tempfile.TemporaryDirectory() as temp:
        base = Path(temp)
        normalized = base / "normalized"
        write_platform(
            normalized,
            "linux",
            [],
            collector_status=POLICY["status_codes"]["cargo_failed_with_log"],
            cargo_exit=101,
        )
        write_platform(normalized, "windows", [])
        output = base / "summary.json"
        result = run_gate(normalized, output)
        summary = json.loads(output.read_text(encoding="utf-8"))
        assert result.returncode == 1, result.stdout
        codes = {item["code"] for item in summary["blockers"]}
        assert POLICY["status_codes"]["collector_failed"] in codes
        assert POLICY["status_codes"]["cargo_failed"] in codes


def test_invalid_fingerprint_is_blocked() -> None:
    with tempfile.TemporaryDirectory() as temp:
        base = Path(temp)
        normalized = base / "normalized"
        bad = make_error()
        bad["fingerprint"] = "forged"
        write_platform(normalized, "linux", [bad])
        write_platform(normalized, "windows", [])
        output = base / "summary.json"
        result = run_gate(normalized, output)
        summary = json.loads(output.read_text(encoding="utf-8"))
        assert result.returncode == 1, result.stdout
        assert POLICY["status_codes"]["fingerprint_invalid"] in {item["code"] for item in summary["blockers"]}


def test_configured_rust_toolchain_mismatch_is_blocked() -> None:
    with tempfile.TemporaryDirectory() as temp:
        base = Path(temp)
        normalized = base / "normalized"
        write_platform(normalized, "linux", [], rustc_version="rustc 1.97.1 (test)")
        write_platform(normalized, "windows", [], rustc_version="rustc 1.97.1 (test)")
        output = base / "summary.json"
        result = run_gate(normalized, output)
        merged = json.loads(output.read_text(encoding="utf-8"))
        assert result.returncode == 1, result.stdout
        assert POLICY["status_codes"]["toolchain_mismatch"] in {item["code"] for item in merged["blockers"]}


def test_platform_identity_mismatch_is_blocked() -> None:
    with tempfile.TemporaryDirectory() as temp:
        base = Path(temp)
        normalized = base / "normalized"
        write_platform(normalized, "linux", [])
        write_platform(normalized, "windows", [])
        summary_path = normalized / "windows" / POLICY["artifacts"]["normalized_summary"]
        summary = json.loads(summary_path.read_text(encoding="utf-8"))
        summary["host_os"] = "Linux"
        summary_path.write_text(json.dumps(summary), encoding="utf-8")
        output = base / "summary.json"
        result = run_gate(normalized, output)
        merged = json.loads(output.read_text(encoding="utf-8"))
        assert result.returncode == 1, result.stdout
        assert POLICY["status_codes"]["platform_identity_mismatch"] in {item["code"] for item in merged["blockers"]}


def test_provenance_mismatch_is_blocked() -> None:
    with tempfile.TemporaryDirectory() as temp:
        base = Path(temp)
        normalized = base / "normalized"
        write_platform(normalized, "linux", [], git_head="head-a")
        write_platform(normalized, "windows", [], git_head="head-b")
        output = base / "summary.json"
        result = run_gate(normalized, output, expected_head="head-a")
        merged = json.loads(output.read_text(encoding="utf-8"))
        assert result.returncode == 1, result.stdout
        assert POLICY["status_codes"]["provenance_mismatch"] in {item["code"] for item in merged["blockers"]}


def test_malformed_errors_json_still_writes_gate_summary() -> None:
    with tempfile.TemporaryDirectory() as temp:
        base = Path(temp)
        normalized = base / "normalized"
        write_platform(normalized, "linux", [])
        write_platform(normalized, "windows", [])
        (normalized / "windows" / POLICY["artifacts"]["normalized_errors"]).write_text("{bad", encoding="utf-8")
        output = base / "summary.json"
        result = run_gate(normalized, output)
        merged = json.loads(output.read_text(encoding="utf-8"))
        assert result.returncode == 1, result.stdout
        assert POLICY["status_codes"]["errors_payload_invalid"] in {item["code"] for item in merged["blockers"]}


def test_upstream_collect_failure_is_blocked() -> None:
    with tempfile.TemporaryDirectory() as temp:
        base = Path(temp)
        normalized = base / "normalized"
        write_platform(normalized, "linux", [])
        write_platform(normalized, "windows", [])
        output = base / "summary.json"
        result = run_gate(normalized, output, policy_result="success", collect_result="failure")
        merged = json.loads(output.read_text(encoding="utf-8"))
        assert result.returncode == 1, result.stdout
        assert POLICY["status_codes"]["upstream_job_failed"] in {item["code"] for item in merged["blockers"]}


def test_same_error_dedupes_across_platforms() -> None:
    with tempfile.TemporaryDirectory() as temp:
        base = Path(temp)
        normalized = base / "normalized"
        error = make_error()
        write_platform(normalized, "linux", [error])
        write_platform(normalized, "windows", [error])
        output = base / "summary.json"
        result = run_gate(normalized, output)
        merged = json.loads(output.read_text(encoding="utf-8"))
        assert result.returncode == 1, result.stdout
        assert merged["unique_errors"] == 1
        assert merged["errors"][0]["platforms"] == ["linux", "windows"]
        assert POLICY["status_codes"]["compiler_errors_present"] in {item["code"] for item in merged["blockers"]}


if __name__ == "__main__":
    test_windows_path_normalization()
    test_same_error_has_same_fingerprint_across_platforms()
    test_missing_raw_log_is_blocked()
    test_zero_error_gate_passes()
    test_cargo_nonzero_with_zero_compiler_errors_is_blocked()
    test_invalid_fingerprint_is_blocked()
    test_configured_rust_toolchain_mismatch_is_blocked()
    test_platform_identity_mismatch_is_blocked()
    test_provenance_mismatch_is_blocked()
    test_malformed_errors_json_still_writes_gate_summary()
    test_upstream_collect_failure_is_blocked()
    test_same_error_dedupes_across_platforms()
    print("compiler evidence pipeline tests: PASS")
