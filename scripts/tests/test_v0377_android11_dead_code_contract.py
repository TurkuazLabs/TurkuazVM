# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0377_android11_dead_code_contract.py
# 📌 Amac: Android BUILD_INFO discovery verisinin gereksiz Distribution struct alanina geri sizmasini engeller
# 📌 Modul - Python
# Version: 0.40.0
# Aciklama: BUILD_INFO parserinin Provider Tool icinde lokal kalmasini ve BuildDiscovery yapisinin yalniz indirilecek resolved metadata tasimasini dogrular
# Bagimli Oldugu Katman: Tool | Service

from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[2]
DIST = (ROOT / "crates/guest/src/tools/android_ci_distribution_tool.rs").read_text(encoding="utf-8")
PROVIDER = (ROOT / "crates/guest/src/tools/android_ci_source_provider_tool.rs").read_text(encoding="utf-8")


def main() -> None:
    match = re.search(r"struct BuildDiscovery \{(?P<body>.*?)\n\}", DIST, re.S)
    assert match, "BUILD_DISCOVERY_STRUCT_MISSING"
    body = match.group("body")
    assert "build_info:" not in body, "BUILD_DISCOVERY_UNUSED_BUILD_INFO_FIELD_RETURNED"
    assert "let build_info =" in PROVIDER, "BUILD_INFO_LOCAL_PROVIDER_VALUE_MISSING"
    assert 'parse_build_info_string(&build_info, "build_version_sdk")' in PROVIDER, "BUILD_INFO_SDK_PARSE_MISSING"
    assert 'parse_build_info_string(&build_info, "build_version_release")' in PROVIDER, "BUILD_INFO_RELEASE_PARSE_MISSING"
    assert "status.json" not in DIST, "DISTRIBUTION_TOOL_DISCOVERY_DEAD_CODE_RETURNED"
    assert "#[allow(dead_code)]" not in DIST and "#[allow(dead_code)]" not in PROVIDER, "DEAD_CODE_LINT_SUPPRESSION_NOT_ALLOWED"
    print("V0377_ANDROID11_DEAD_CODE_CONTRACT=PASS")


if __name__ == "__main__":
    main()
