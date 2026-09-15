# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0406_release_metadata_contract.py
# 📌 Amac: Aktif release metadata alanlarini ve Python FULL recovery gate capraz dogrulamasini fail-closed kilitler
# 📌 Modul - Python
# Version: 0.40.6
# Aciklama: Engine API, guest-agent protocol, manifest ve release status surumlerinin paketlemede tekrar kaybolmasini engeller
# Bagimli Oldugu Katman: Tool | Config | Service

from __future__ import annotations

import re
import tomllib
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> None:
    with (ROOT / "Cargo.toml").open("rb") as handle:
        cargo = tomllib.load(handle)
    version = str(cargo["workspace"]["package"]["version"])

    status_path = ROOT / f"RELEASE_STATUS_v{version}.yml"
    require(status_path.is_file(), "RELEASE_STATUS_CURRENT_VERSION_MISSING")
    status = yaml.safe_load(status_path.read_text(encoding="utf-8"))
    compatibility = status.get("compatibility") or {}

    require(str(status.get("version")) == version, "RELEASE_STATUS_VERSION_MISMATCH")
    require(status.get("package") == f"TurkuazVM-v{version}-FULL", "RELEASE_PACKAGE_NAME_MISMATCH")
    require(
        status.get("authoritative_manifest") == f"MANIFEST_SHA256_v{version}.txt",
        "RELEASE_MANIFEST_NAME_MISMATCH",
    )
    require(compatibility.get("engine_api_version") == 24, "RELEASE_ENGINE_API_VERSION_MISSING_OR_INVALID")
    require(
        compatibility.get("guest_agent_protocol_version") == 2,
        "RELEASE_GUEST_AGENT_PROTOCOL_VERSION_MISSING_OR_INVALID",
    )

    engine_api = read("crates/engine-api/src/lib.rs")
    engine_match = re.search(r"ENGINE_API_VERSION:\s*u16\s*=\s*(\d+)\s*;", engine_api)
    require(engine_match is not None, "ENGINE_API_SOURCE_VERSION_MISSING")
    require(int(engine_match.group(1)) == compatibility["engine_api_version"], "ENGINE_API_RELEASE_MISMATCH")

    guest_protocol = read("guest/android-agent/res/values/config.xml")
    guest_match = re.search(r'<integer\s+name="guest_agent_protocol_version">(\d+)</integer>', guest_protocol)
    require(guest_match is not None, "GUEST_AGENT_PROTOCOL_SOURCE_VERSION_MISSING")
    require(
        int(guest_match.group(1)) == compatibility["guest_agent_protocol_version"],
        "GUEST_AGENT_PROTOCOL_RELEASE_MISMATCH",
    )

    gate = read("scripts/full_recovery_gate.py")
    for token in (
        "validate_release_metadata",
        "RELEASE_STATUS_v{workspace_version}.yml",
        "engine_api_version",
        "guest_agent_protocol_version",
        "authoritative_manifest",
    ):
        require(token in gate, f"FULL_RECOVERY_RELEASE_METADATA_GATE_MISSING:{token}")

    print("V0406_RELEASE_METADATA_CONTRACT=PASS")


if __name__ == "__main__":
    main()
