# 📄 Dosya Yolu: /turkuazvm/scripts/full_recovery_gate.py
# 📌 Amac: Kurtarilmis FULL TurkuazVM workspace'inin source, config, launcher ve regression butunlugunu dogrular
# 📌 Modul - Python
# Version: 0.41.2
# Aciklama: Workspace ve release metadata ile Baglanti Merkezi calisan-VM erisim, VM workspace, Android SDK ve legacy regression kontratlarini tek static gate altinda dogrular
# Bagimli Oldugu Katman: Tool | View

from __future__ import annotations

import argparse
import compileall
import re
import shutil
import subprocess
import sys
import tomllib
from pathlib import Path

import yaml

STATUS_PASS = "PASS"
STATUS_BLOCKED = "BLOCKED"
STATUS_UNVERIFIED = "UNVERIFIED"
ROOT_MARKER = "Cargo.toml"
SUMMARY_PATH = Path("artifacts/full-recovery-gate-summary.yml")
MIN_RUST_FILES = 236
MIN_CARGO_MANIFESTS = 19
REQUIRED_PATHS = (
    Path("Cargo.toml"),
    Path("TurkuazVM-Start.cmd"),
    Path("scripts/verify.ps1"),
    Path("apps/engine/Cargo.toml"),
    Path("apps/desktop/src-tauri/Cargo.toml"),
    Path("apps/desktop/src-tauri/icons/icon.ico"),
    Path("crates/core/Cargo.toml"),
    Path("crates/qemu/Cargo.toml"),
    Path("crates/guest/Cargo.toml"),
    Path("docs/recovery/FULL_RECOVERY_STATUS.yml"),
    Path("docs/recovery/A_G_RECOVERY_MATRIX.md"),
    Path(".github/workflows/artifact-cache-agent-gate.yml"),
    Path("packages/guest-agent/manifest.example.yml"),
    Path("scripts/release_guest_agent_update.py"),
    Path("scripts/artifact_cache_agent_gate.py"),
)
REGRESSION_TESTS = (
    Path("scripts/tests/test_compiler_log_pipeline.py"),
    Path("scripts/tests/test_host_resilience_policy.py"),
    Path("scripts/tests/test_host_resilience_runtime.py"),
    Path("scripts/tests/test_runtime_validation_harness.py"),
    Path("scripts/tests/test_artifact_cache_agent.py"),
    Path("scripts/tests/test_artifact_cache_http_revalidation.py"),
    Path("scripts/tests/test_artifact_cache_v21_contract.py"),
    Path("scripts/tests/test_release_root_cleanliness.py"),
    Path("scripts/tests/test_v028_servbay_download_cancel_contract.py"),
    Path("scripts/tests/test_v033_workspace_contract.py"),
    Path("scripts/tests/test_v0331_launcher_workspace_gate_contract.py"),
    Path("scripts/tests/test_v034_android_release_taskbar_contract.py"),
    Path("scripts/tests/test_v035_android_catalog_standard_mode_contract.py"),
    Path("scripts/tests/test_v036_image_center_download_sources_contract.py"),
    Path("scripts/tests/test_v0361_launcher_android_schema_gate_contract.py"),
    Path("scripts/tests/test_v0362_launcher_android_ci_source_gate_contract.py"),
    Path("scripts/tests/test_v0363_rust_android_ci_compile_contract.py"),
    Path("scripts/tests/test_v0364_engine_download_config_compile_contract.py"),
    Path("scripts/tests/test_v0365_launcher_download_sources_path_gate_contract.py"),
    Path("scripts/tests/test_v037_workspace_hardening_contract.py"),
    Path("scripts/tests/test_v0379_android10_legacy_ci_status_contract.py"),
    Path("scripts/tests/test_v0380_unified_download_ux_contract.py"),
    Path("scripts/tests/test_v0400_android_source_resolver_contract.py"),
    Path("scripts/tests/test_v0401_installer_managed_download_contract.py"),
    Path("scripts/tests/test_v0402_native_display_telemetry_contract.py"),
    Path("scripts/tests/test_v0403_installer_session_media_contract.py"),
    Path("scripts/tests/test_v0404_launcher_softbuffer_contract.py"),
    Path("scripts/tests/test_v0405_android_ci_dynamic_target_contract.py"),
    Path("scripts/tests/test_v0406_release_metadata_contract.py"),
    Path("scripts/tests/test_v0407_android_ci_dead_code_contract.py"),
    Path("scripts/tests/test_v0408_android_ci_launcher_verifier_contract.py"),
    Path("scripts/tests/test_v0409_android_ci_numeric_build_compile_contract.py"),
    Path("scripts/tests/test_v0410_engine_start_crash_diagnostics_contract.py"),
    Path("scripts/tests/test_v0411_windows_runtime_binary_lock_contract.py"),
    Path("scripts/tests/test_v0412_windows_ps51_process_collection_contract.py"),
    Path("scripts/tests/test_v0413_android_sdk_windows_provider_contract.py"),
    Path("scripts/tests/test_v0414_android_emulator_port_config_wiring_contract.py"),
    Path("scripts/tests/test_v0410_vm_control_center_contract.py"),
    Path("scripts/tests/test_v0411_vm_workspace_usability_contract.py"),
    Path("scripts/tests/test_v0412_connection_running_vm_access_contract.py"),
)
TEXT_SUFFIXES = {
    ".rs", ".toml", ".yml", ".yaml", ".py", ".ps1", ".cmd", ".md", ".txt", ".json", ".json5", ".sh"
}


def find_root() -> Path:
    candidate = Path(__file__).resolve().parent.parent
    if not (candidate / ROOT_MARKER).is_file():
        raise RuntimeError("TurkuazVM workspace root bulunamadi")
    return candidate


def parse_workspace(root: Path) -> tuple[list[str], dict]:
    with (root / "Cargo.toml").open("rb") as handle:
        document = tomllib.load(handle)
    members = document.get("workspace", {}).get("members", [])
    if not isinstance(members, list) or not members:
        raise RuntimeError("Cargo workspace member listesi bos")
    return [str(item) for item in members], document


def validate_required_paths(root: Path) -> list[str]:
    errors: list[str] = []
    for relative in REQUIRED_PATHS:
        if not (root / relative).exists():
            errors.append(f"required path missing: {relative.as_posix()}")
    return errors


def validate_release_metadata(root: Path) -> list[str]:
    errors: list[str] = []
    try:
        _, workspace_document = parse_workspace(root)
        workspace_version = str(workspace_document.get("workspace", {}).get("package", {}).get("version", "")).strip()
    except Exception as error:  # noqa: BLE001
        return [f"release metadata workspace parse failed: {error}"]

    if not workspace_version:
        return ["release metadata workspace version missing"]

    release_status_name = f"RELEASE_STATUS_v{workspace_version}.yml"
    release_status_path = root / release_status_name
    expected_manifest_name = f"MANIFEST_SHA256_v{workspace_version}.txt"

    if not release_status_path.is_file():
        errors.append(f"release status missing: {release_status_name}")
        return errors

    try:
        with release_status_path.open("r", encoding="utf-8") as handle:
            release_status = yaml.safe_load(handle) or {}
    except Exception as error:  # noqa: BLE001
        errors.append(f"release status YAML invalid: {error}")
        return errors

    if not isinstance(release_status, dict):
        errors.append("release status root must be mapping")
        return errors

    if str(release_status.get("version", "")).strip() != workspace_version:
        errors.append(
            f"release status version mismatch: workspace={workspace_version} release={release_status.get('version')}"
        )

    expected_package = f"TurkuazVM-v{workspace_version}-FULL"
    if release_status.get("package") != expected_package:
        errors.append(
            f"release package mismatch: expected={expected_package} actual={release_status.get('package')}"
        )

    if release_status.get("authoritative_manifest") != expected_manifest_name:
        errors.append(
            "release authoritative manifest mismatch: "
            f"expected={expected_manifest_name} actual={release_status.get('authoritative_manifest')}"
        )

    compatibility = release_status.get("compatibility")
    if not isinstance(compatibility, dict):
        errors.append("release compatibility mapping missing")
        compatibility = {}

    release_engine_api = compatibility.get("engine_api_version")
    if not isinstance(release_engine_api, int) or release_engine_api <= 0:
        errors.append("release engine_api_version missing or invalid")

    release_guest_protocol = compatibility.get("guest_agent_protocol_version")
    if not isinstance(release_guest_protocol, int) or release_guest_protocol <= 0:
        errors.append("release guest_agent_protocol_version missing or invalid")

    engine_api_path = root / "crates/engine-api/src/lib.rs"
    if engine_api_path.is_file():
        engine_api_text = engine_api_path.read_text(encoding="utf-8")
        match = re.search(r"ENGINE_API_VERSION:\s*u16\s*=\s*(\d+)\s*;", engine_api_text)
        if match is None:
            errors.append("engine API source version missing")
        elif isinstance(release_engine_api, int) and int(match.group(1)) != release_engine_api:
            errors.append(
                f"engine API version mismatch: source={match.group(1)} release={release_engine_api}"
            )
    else:
        errors.append("engine API source missing")

    guest_protocol_path = root / "guest/android-agent/res/values/config.xml"
    if guest_protocol_path.is_file():
        guest_protocol_text = guest_protocol_path.read_text(encoding="utf-8")
        match = re.search(
            r'<integer\s+name="guest_agent_protocol_version">(\d+)</integer>',
            guest_protocol_text,
        )
        if match is None:
            errors.append("guest-agent protocol source version missing")
        elif isinstance(release_guest_protocol, int) and int(match.group(1)) != release_guest_protocol:
            errors.append(
                "guest-agent protocol version mismatch: "
                f"source={match.group(1)} release={release_guest_protocol}"
            )
    else:
        errors.append("guest-agent protocol source missing")

    stale_release_status = sorted(
        path.name for path in root.glob("RELEASE_STATUS_v*.yml") if path.name != release_status_name
    )
    if stale_release_status:
        errors.append(f"stale release status files in root: {', '.join(stale_release_status)}")

    stale_manifest = sorted(
        path.name for path in root.glob("MANIFEST_SHA256_v*.txt") if path.name != expected_manifest_name
    )
    if stale_manifest:
        errors.append(f"stale release manifest files in root: {', '.join(stale_manifest)}")

    return errors


def validate_cargo(root: Path) -> tuple[list[str], int, int]:
    errors: list[str] = []
    members, _ = parse_workspace(root)
    for member in members:
        manifest = root / member / "Cargo.toml"
        if not manifest.is_file():
            errors.append(f"workspace member manifest missing: {member}/Cargo.toml")
            continue
        try:
            with manifest.open("rb") as handle:
                tomllib.load(handle)
        except Exception as error:  # noqa: BLE001
            errors.append(f"invalid Cargo manifest {manifest.relative_to(root).as_posix()}: {error}")
    manifests = list(root.rglob("Cargo.toml"))
    rust_files = list(root.rglob("*.rs"))
    if len(manifests) < MIN_CARGO_MANIFESTS:
        errors.append(f"Cargo manifest count too low: {len(manifests)} < {MIN_CARGO_MANIFESTS}")
    if len(rust_files) < MIN_RUST_FILES:
        errors.append(f"Rust file count too low: {len(rust_files)} < {MIN_RUST_FILES}")
    return errors, len(manifests), len(rust_files)


def validate_yaml_toml(root: Path) -> list[str]:
    errors: list[str] = []
    for path in root.rglob("*.toml"):
        try:
            with path.open("rb") as handle:
                tomllib.load(handle)
        except Exception as error:  # noqa: BLE001
            errors.append(f"invalid TOML {path.relative_to(root).as_posix()}: {error}")
    for pattern in ("*.yml", "*.yaml"):
        for path in root.rglob(pattern):
            try:
                with path.open("r", encoding="utf-8") as handle:
                    yaml.safe_load(handle)
            except Exception as error:  # noqa: BLE001
                errors.append(f"invalid YAML {path.relative_to(root).as_posix()}: {error}")
    return errors


def validate_text_files(root: Path) -> list[str]:
    errors: list[str] = []
    ignored_parts = {"target", ".git", "__pycache__"}
    for path in root.rglob("*"):
        if not path.is_file() or path.suffix.lower() not in TEXT_SUFFIXES:
            continue
        if any(part in ignored_parts for part in path.parts):
            continue
        data = path.read_bytes()
        if b"\x00" in data:
            errors.append(f"NUL byte found: {path.relative_to(root).as_posix()}")
    return errors


def validate_python(root: Path) -> list[str]:
    errors: list[str] = []
    if not compileall.compile_dir(root / "scripts", quiet=1):
        errors.append("Python compileall failed under scripts/")
    for test in REGRESSION_TESTS:
        target = root / test
        if not target.is_file():
            errors.append(f"regression test missing: {test.as_posix()}")
            continue
        completed = subprocess.run(
            [sys.executable, str(target)],
            cwd=root,
            capture_output=True,
            text=True,
            check=False,
        )
        if completed.returncode != 0:
            detail = (completed.stderr or completed.stdout).strip()
            errors.append(f"regression failed {test.as_posix()}: {detail}")
    return errors


def run_cargo(root: Path, enabled: bool) -> tuple[str, list[str]]:
    if not enabled:
        return STATUS_UNVERIFIED, []
    cargo = shutil.which("cargo")
    rustc = shutil.which("rustc")
    if cargo is None or rustc is None:
        return STATUS_BLOCKED, ["cargo/rustc missing"]
    errors: list[str] = []
    for command in (
        [cargo, "check", "--workspace", "--all-targets"],
        [cargo, "test", "--workspace"],
    ):
        completed = subprocess.run(command, cwd=root, check=False)
        if completed.returncode != 0:
            errors.append(f"command failed: {' '.join(command)} exit={completed.returncode}")
            break
    return (STATUS_PASS if not errors else STATUS_BLOCKED), errors


def write_summary(
    root: Path,
    static_errors: list[str],
    cargo_status: str,
    cargo_errors: list[str],
    cargo_manifests: int,
    rust_files: int,
) -> None:
    summary = root / SUMMARY_PATH
    summary.parent.mkdir(parents=True, exist_ok=True)
    static_status = STATUS_PASS if not static_errors else STATUS_BLOCKED
    _, workspace_document = parse_workspace(root)
    workspace_version = str(workspace_document.get("workspace", {}).get("package", {}).get("version", "unknown"))
    payload = {
        "version": workspace_version,
        "package_kind": "FULL_SOURCE_RECOVERY",
        "static_status": static_status,
        "cargo_status": cargo_status,
        "counts": {
            "cargo_manifests": cargo_manifests,
            "rust_files": rust_files,
        },
        "static_errors": static_errors,
        "cargo_errors": cargo_errors,
    }
    header = (
        "# 📄 Dosya Yolu: /turkuazvm/artifacts/full-recovery-gate-summary.yml\n"
        "# 📌 Amac: FULL recovery gate sonucunu makine-okunur kaydeder\n"
        "# 📌 Modul - YAML\n"
        f"# Version: {workspace_version}\n"
        "# Aciklama: Static workspace ve opsiyonel Cargo gate sonucunu kaydeder\n"
        "# Bagimli Oldugu Katman: Tool | View\n\n"
    )
    summary.write_text(header + yaml.safe_dump(payload, sort_keys=False), encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description="TurkuazVM FULL recovery gate")
    parser.add_argument("--cargo", action="store_true", help="cargo check/test de calistir")
    args = parser.parse_args()
    root = find_root()
    errors: list[str] = []
    errors.extend(validate_required_paths(root))
    errors.extend(validate_release_metadata(root))
    cargo_shape_errors, cargo_manifests, rust_files = validate_cargo(root)
    errors.extend(cargo_shape_errors)
    errors.extend(validate_yaml_toml(root))
    errors.extend(validate_text_files(root))
    errors.extend(validate_python(root))
    cargo_status, cargo_errors = run_cargo(root, args.cargo)
    write_summary(root, errors, cargo_status, cargo_errors, cargo_manifests, rust_files)
    if errors:
        print(f"FULL_RECOVERY_GATE={STATUS_BLOCKED}")
        for error in errors:
            print(f"- {error}")
        return 2
    print(f"FULL_RECOVERY_GATE={STATUS_PASS}")
    print(f"CARGO_GATE={cargo_status}")
    print(f"CARGO_MANIFESTS={cargo_manifests}")
    print(f"RUST_FILES={rust_files}")
    return 0 if cargo_status != STATUS_BLOCKED else 3


if __name__ == "__main__":
    raise SystemExit(main())
