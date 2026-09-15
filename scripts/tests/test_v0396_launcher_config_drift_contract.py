# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0396_launcher_config_drift_contract.py
# 📌 Amac: Windows launcher structure gate icinde stale timeout magic-value regresyonunu engeller
# 📌 Modul - Python
# Version: 0.39.6
# Aciklama: Main config timeout semantigini ve verifierin exact release degerine bagli olmadigini statik dogrular
# Bagimli Oldugu Katman: Tool | Config

from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[2]
config = (ROOT / "config/turkuazvm.yml").read_text(encoding="utf-8")
verifier = (ROOT / "scripts/verify_structure.ps1").read_text(encoding="utf-8")

request_match = re.search(r"(?m)^\s*request_timeout_ms:\s*(\d+)\s*$", config)
long_match = re.search(r"(?m)^\s*long_request_timeout_ms:\s*(\d+)\s*$", config)
assert request_match is not None
assert long_match is not None
request_timeout_ms = int(request_match.group(1))
long_request_timeout_ms = int(long_match.group(1))
assert request_timeout_ms > 0
assert long_request_timeout_ms >= 30_000
assert long_request_timeout_ms >= request_timeout_ms
assert request_timeout_ms == 5_000
assert long_request_timeout_ms == 300_000

assert '"long_request_timeout_ms: 180000"' not in verifier
assert '"long_request_timeout_ms: 300000"' not in verifier
assert "DESKTOP_LONG_TIMEOUT_MAIN_CONFIG_TOO_SMALL" in verifier
assert "DESKTOP_LONG_TIMEOUT_MAIN_CONFIG_LT_REQUEST" in verifier
assert "[regex]::Match" in verifier
print("v0.39.6 launcher config drift contract: PASS")
