# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0271_launcher_network_verifier_contract.py
# 📌 Amac: Launcher structure verifier icindeki v0.26 stale network kimligi beklentisinin geri gelmesini engeller
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Main config ile verify_structure managed network tokenlarini ayni sozlesmede fail-closed dogrular
# Bagimli Oldugu Katman: Tool | View

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def main() -> int:
    config = (ROOT / "config/turkuazvm.yml").read_text(encoding="utf-8")
    verifier = (ROOT / "scripts/verify_structure.ps1").read_text(encoding="utf-8")

    required = (
        "default_network_id: turkuaz-net-01",
        "private_network_id: turkuaz-private-01",
        "default_profile: managed_nat",
        "managed_helper_path: scripts/network_windows_managed.ps1",
    )
    for token in required:
        assert token in config, f"main config missing: {token}"
        assert token in verifier, f"structure verifier missing: {token}"

    assert "default_network_id: default" not in verifier, "stale default network verifier token returned"
    assert '"openNetworkModal", "attach_default_network"' not in verifier, "stale desktop network action returned"
    assert '"openNetworkModal", "attach_network_profile"' in verifier, "managed profile desktop verifier action missing"
    print("V0271_LAUNCHER_NETWORK_VERIFIER_CONTRACT_OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
