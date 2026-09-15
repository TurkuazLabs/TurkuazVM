# 📄 Dosya Yolu: /turkuazvm/scripts/host_resilience_policy.py
# 📌 Amac: J host resilience policy dosyasini fail-closed invariant setine gore dogrular
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Image rescue, network recovery, drag-drop ve shared-folder guvenlik kurallarinin config drift yasamasini engeller
# Bagimli Oldugu Katman: Tool | View

from __future__ import annotations

import sys
from pathlib import Path
from typing import Any

try:
    import yaml
except ImportError as exc:
    raise SystemExit("TVM-J-102 PyYAML dependency missing") from exc

ROOT = Path(__file__).resolve().parents[1]
CONFIG_PATH = ROOT / "config" / "host-resilience.yml"
MODULE_VERSION = "0.14.0"
BOOTSTRAP_CONFIG_MISSING = "TVM-J-101"
BOOTSTRAP_CONFIG_INVALID = "TVM-J-102"
SUMMARY_PATH = ROOT / "artifacts" / "host-resilience-policy-summary.yml"


class PolicyError(RuntimeError):
    pass


def require(condition: bool, code: str, message: str) -> None:
    if not condition:
        raise PolicyError(f"{code} {message}")


def load_policy(path: Path = CONFIG_PATH) -> dict[str, Any]:
    require(path.is_file(), BOOTSTRAP_CONFIG_MISSING, f"config missing: {path}")
    try:
        payload = yaml.safe_load(path.read_text(encoding="utf-8"))
    except Exception as exc:
        raise PolicyError(f"{BOOTSTRAP_CONFIG_INVALID} config parse failed: {exc}") from exc
    require(isinstance(payload, dict), BOOTSTRAP_CONFIG_INVALID, "config root must be a mapping")
    require(payload.get("schema") == 1, BOOTSTRAP_CONFIG_INVALID, "unsupported schema")
    require(payload.get("version") == MODULE_VERSION, BOOTSTRAP_CONFIG_INVALID, "unexpected version")
    section = payload.get("host_resilience")
    require(isinstance(section, dict), BOOTSTRAP_CONFIG_INVALID, "host_resilience section missing")
    return payload


def validate_policy(payload: dict[str, Any]) -> list[str]:
    policy = payload["host_resilience"]
    image = policy["image_recovery"]
    network = policy["network_recovery"]
    guest = policy["guest_integration"]
    release = policy["release_gate"]
    codes = policy["status_codes"]

    require(image["immutable_source_required"] is True, codes["source_mutation_risk"], "source image must remain immutable")
    require(image["rescue_overlay_required"] is True, codes["rescue_overlay_missing"], "rescue must use an overlay or clone")
    require(image["automatic_repair_allowed"] is False, codes["automatic_repair_forbidden"], "automatic disk repair must stay disabled")
    require(image["qemu_img_check_mode"] == "read_only", codes["automatic_repair_forbidden"], "qemu-img check must be read-only")
    require(image["copy_host_bound_secret_blob"] is False, codes["host_secret_copy_forbidden"], "host-bound secret blobs must not be copied")
    require(image["rewrap_host_bound_secret_on_move"] is True, codes["host_secret_copy_forbidden"], "moved guest secrets must be rewrapped")
    require(image["preserve_firmware_mode"] is True, codes["firmware_change_blocked"], "firmware mode must be preserved")
    require(image["hardware_fingerprint_boot_lock"] is False, codes["source_mutation_risk"], "hardware fingerprint must not become a boot lock")

    require(network["preserve_guest_mac"] is True, codes["guest_mac_change_blocked"], "network recovery must preserve guest MAC")
    for key in ("automatic_host_bridge_mutation", "automatic_host_route_mutation", "automatic_firewall_mutation"):
        require(network[key] is False, codes["host_network_mutation_blocked"], f"{key} must stay disabled")
    require(1 <= int(network["max_recovery_attempts"]) <= 8, codes["network_policy_unsafe"], "recovery attempts must be bounded")
    require(network["bridge_or_tap_setup"] == "explicit_only", codes["host_network_mutation_blocked"], "bridge/TAP setup must require explicit action")
    require(network["inbound_port_forwarding"] == "explicit_only", codes["network_policy_unsafe"], "port forwarding must require explicit action")

    drag = guest["drag_drop"]
    share = guest["shared_folder"]
    require(guest["capability_negotiation_required"] is True, codes["guest_agent_untrusted"], "guest capabilities must be negotiated")
    require(guest["signed_guest_agent_required"] is True, codes["guest_agent_untrusted"], "guest agent must be signed")
    require(drag["transport"] == "tvgb_authenticated_file_transfer", codes["drag_drop_integrity_required"], "drag-drop must use TVGB authenticated transfer")
    require(drag["shared_folder_dependency"] is False, codes["drag_drop_integrity_required"], "drag-drop must not depend on shared folders")
    require(drag["overwrite_existing"] is False, codes["drag_drop_integrity_required"], "drag-drop overwrite must be rejected")
    require(drag["sha256_required"] is True and drag["resume_required"] is True, codes["drag_drop_integrity_required"], "drag-drop integrity and resume are required")
    require(share["default_enabled"] is False, codes["shared_folder_unsupported"], "shared folders must be opt-in")
    require(share["default_access"] == "read_only", codes["shared_folder_path_rejected"], "shared folders must default read-only")
    require(share["host_path_allowlist_required"] is True, codes["shared_folder_path_rejected"], "host share allowlist is required")
    require(share["canonical_path_confinement_required"] is True, codes["shared_folder_path_rejected"], "canonical path confinement is required")
    require(share["symlink_escape_reject"] is True, codes["shared_folder_path_rejected"], "symlink escape must be rejected")
    require(share["network_share_fallback_allowed"] is False, codes["shared_folder_unsupported"], "implicit SMB/network share fallback is forbidden")
    require(share["unsupported_fallback"] == "tvgb_on_demand_copy", codes["shared_folder_unsupported"], "unsupported share fallback must be TVGB copy")
    require(share["backends"]["windows"]["maturity"] == "experimental", codes["windows_virtiofs_unverified"], "Windows virtiofs must remain experimental until runtime validation")

    require(release["production_status"] == "BLOCKED_UNTIL_RUNTIME_TESTS_PASS", codes["runtime_validation_required"], "production gate must remain blocked before runtime tests")

    return [
        "immutable image rescue",
        "host-bound secret rewrap",
        "firmware preservation",
        "bounded network recovery",
        "guest MAC preservation",
        "no implicit host network mutation",
        "TVGB drag-drop integrity",
        "opt-in confined shared folders",
        "Windows virtiofs experimental gate",
        "runtime validation release block",
    ]


def write_summary(status: str, checks: list[str], error: str | None = None) -> None:
    SUMMARY_PATH.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema": 1,
        "version": MODULE_VERSION,
        "module": "host-resilience-j",
        "status": status,
        "checks": checks,
        "error": error,
    }
    header = (
        "# 📄 Dosya Yolu: /turkuazvm/artifacts/host-resilience-policy-summary.yml\n"
        "# 📌 Amac: J host resilience policy gate sonucunu makine-okunur evidence olarak kaydeder\n"
        "# 📌 Modul - YAML\n"
        f"# Version: {MODULE_VERSION}\n"
        "# Aciklama: Policy invariant kontrollerinin PASS veya BLOCKED sonucunu ve hata kodunu saklar\n"
        "# Bagimli Oldugu Katman: Tool | View\n\n"
    )
    SUMMARY_PATH.write_text(header + yaml.safe_dump(payload, sort_keys=False), encoding="utf-8")


def main() -> int:
    try:
        payload = load_policy()
        checks = validate_policy(payload)
        pass_code = payload["host_resilience"]["status_codes"]["pass"]
    except PolicyError as exc:
        write_summary("BLOCKED", [], str(exc))
        print(str(exc), file=sys.stderr)
        return 2

    write_summary("PASS", checks)
    print(f"{pass_code} PASS ({len(checks)} invariants)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
