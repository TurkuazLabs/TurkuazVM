# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0312_windows_network_transaction_contract.py
# 📌 Amac: v0.31.2 Windows managed network transactional rollback, stale recovery ve TAP runtime probe kontratini dogrular
# 📌 Modul - Python
# Version: 0.31.2
# Aciklama: Partial ensure kalintilarinin tekrar VM start zincirini kilitlemesini engelleyen runtime kontratini fail-closed test eder
# Bagimli Oldugu Katman: Tool | View

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
HELPER = (ROOT / "scripts" / "network_windows_managed.ps1").read_text(encoding="utf-8")
LAUNCHER = (ROOT / "scripts" / "start_turkuazvm.ps1").read_text(encoding="utf-8")

required_helper = (
    "Version: 0.31.2",
    "Test-TurkuazTapDriverProbe",
    "TURKUAZ_TAP_DRIVER_PROBE_FAILED",
    "ensure_rollback_begin",
    "ensure_rollback_complete",
    "stale_bridge_recovered_by_name",
    "stale_bridge_recovered_by_members",
    "bridge_create_retry_compat",
    "forcecompatmode=enable",
    '"bridge", "destroy"',
    "Remove-TurkuazTapByName",
)
for token in required_helper:
    if token not in HELPER:
        raise SystemExit(f"V0312_NETWORK_TRANSACTION_MISSING: {token}")

required_launcher = (
    'Write-Status -Name "Turkuaz NAT Runtime"',
    "QEMU User NAT hazir; TAP/OpenVPN gerekmez.",
    "Advanced TAP tapctl",
)
for token in required_launcher:
    if token not in LAUNCHER:
        raise SystemExit(f"V0312_LAUNCHER_PROBE_MISSING: {token}")

print("V0312_WINDOWS_NETWORK_TRANSACTION_CONTRACT=PASS")
