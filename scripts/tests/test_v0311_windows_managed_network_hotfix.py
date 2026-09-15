# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0311_windows_managed_network_hotfix.py
# 📌 Amac: Windows managed network helper parse, bridge discovery ve diagnostic hotfix kontratini regression olarak dogrular
# 📌 Modul - Python
# Version: 0.31.2
# Aciklama: PowerShell degisken-colon parse bugini, GUID bridge retry zincirini ve Rust stderr/stdout diagnostic aktarimini fail-closed denetler
# Bagimli Oldugu Katman: Tool | Service

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
HELPER = (ROOT / "scripts" / "network_windows_managed.ps1").read_text(encoding="utf-8")
RUST = (ROOT / "crates" / "platform" / "src" / "tools" / "native_network_tool.rs").read_text(encoding="utf-8")

required_helper = (
    'Version: 0.31.2',
    'BridgeDiscoveryAttempts = 40',
    'Get-TurkuazBridgeAdapterByGuid',
    'Wait-TurkuazBridgeAdapter',
    'InterfaceGuid',
    'TURKUAZ_BRIDGE_ADAPTER_TIMEOUT',
    'TURKUAZ_WINNAT_CONFLICT',
    'windows-managed-network.log',
    '${Protocol}:${HostPort}',
    'Get-NetAdapterBinding',
    'ms_bridge',
)
for token in required_helper:
    if token not in HELPER:
        raise SystemExit(f"V0311_NETWORK_HELPER_MISSING: {token}")

required_rust = (
    'MAX_WINDOWS_HELPER_DIAGNOSTIC_CHARS',
    'String::from_utf8_lossy(&output.stdout)',
    'String::from_utf8_lossy(&output.stderr)',
    'detail={diagnostics}',
    '.output()',
)
for token in required_rust:
    if token not in RUST:
        raise SystemExit(f"V0311_NETWORK_RUST_DIAGNOSTIC_MISSING: {token}")

# PowerShell double-quoted strings cannot safely use an unbraced ordinary variable
# immediately followed by ':' because it can be parsed as scoped-variable syntax.
# Scope variables such as $Script:Name and $env:Path are valid and excluded.
string_literals = re.findall(r'"(?:[^"`]|`.)*"', HELPER)
allowed_scopes = {"Script", "Global", "Local", "Private", "env"}
violations: list[str] = []
for literal in string_literals:
    for match in re.finditer(r'\$([A-Za-z_][A-Za-z0-9_]*):', literal):
        if match.group(1) not in allowed_scopes:
            violations.append(literal)
if violations:
    raise SystemExit(f"V0311_POWERSHELL_UNBRACED_COLON_VARIABLE: {violations[:5]}")

print("V0311_WINDOWS_MANAGED_NETWORK_HOTFIX=PASS")
