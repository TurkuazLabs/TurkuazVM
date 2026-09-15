# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_host_resilience_policy.py
# 📌 Amac: J host resilience policy icin pozitif ve fail-closed negatif regression testlerini calistirir
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Source mutation, host network mutation, secret copy ve guvensiz shared-folder driftlerinin gate tarafindan reddedildigini kanitlar
# Bagimli Oldugu Katman: Tool | View

from __future__ import annotations

import copy
import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = ROOT / "scripts" / "host_resilience_policy.py"
SPEC = importlib.util.spec_from_file_location("host_resilience_policy", MODULE_PATH)
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def expect_block(payload: dict, expected_code: str) -> None:
    try:
        MODULE.validate_policy(payload)
    except MODULE.PolicyError as exc:
        assert expected_code in str(exc), str(exc)
        return
    raise AssertionError(f"expected policy block: {expected_code}")


def main() -> int:
    baseline = MODULE.load_policy()
    checks = MODULE.validate_policy(baseline)
    assert len(checks) >= 10
    codes = baseline["host_resilience"]["status_codes"]

    case = copy.deepcopy(baseline)
    case["host_resilience"]["image_recovery"]["immutable_source_required"] = False
    expect_block(case, codes["source_mutation_risk"])

    case = copy.deepcopy(baseline)
    case["host_resilience"]["image_recovery"]["copy_host_bound_secret_blob"] = True
    expect_block(case, codes["host_secret_copy_forbidden"])

    case = copy.deepcopy(baseline)
    case["host_resilience"]["network_recovery"]["automatic_host_bridge_mutation"] = True
    expect_block(case, codes["host_network_mutation_blocked"])

    case = copy.deepcopy(baseline)
    case["host_resilience"]["network_recovery"]["preserve_guest_mac"] = False
    expect_block(case, codes["guest_mac_change_blocked"])

    case = copy.deepcopy(baseline)
    case["host_resilience"]["guest_integration"]["drag_drop"]["transport"] = "shared_folder"
    expect_block(case, codes["drag_drop_integrity_required"])

    case = copy.deepcopy(baseline)
    case["host_resilience"]["guest_integration"]["shared_folder"]["default_enabled"] = True
    expect_block(case, codes["shared_folder_unsupported"])

    case = copy.deepcopy(baseline)
    case["host_resilience"]["guest_integration"]["shared_folder"]["network_share_fallback_allowed"] = True
    expect_block(case, codes["shared_folder_unsupported"])

    case = copy.deepcopy(baseline)
    case["host_resilience"]["guest_integration"]["shared_folder"]["backends"]["windows"]["maturity"] = "stable"
    expect_block(case, codes["windows_virtiofs_unverified"])

    case = copy.deepcopy(baseline)
    case["host_resilience"]["release_gate"]["production_status"] = "PASS"
    expect_block(case, codes["runtime_validation_required"])

    print(f"{codes['pass']} PASS (baseline + 9 negative scenarios)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
