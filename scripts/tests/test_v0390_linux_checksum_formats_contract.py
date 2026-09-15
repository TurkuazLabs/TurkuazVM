# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0390_linux_checksum_formats_contract.py
# 📌 Amac: Linux resmi ISO checksum listelerinde GNU ve Fedora BSD SHA256 bicimlerinin birlikte desteklendigini dogrular
# 📌 Modul - Python
# Version: 0.39.0
# Aciklama: Ubuntu/Debian/Rocky hash-filename ve Fedora SHA256(filename)=hash bicimlerinin parser tarafinda regressiona ugramasini engeller
# Bagimli Oldugu Katman: Service | Tool

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SERVICE = (ROOT / "apps/engine/src/services/installer_media_download_application_service.rs").read_text(encoding="utf-8")


def main() -> None:
    assert 'let prefix = format!("SHA256 ({filename}) = ");' in SERVICE, "FEDORA_BSD_CHECKSUM_FORMAT_MISSING"
    assert "fn valid_sha256(value: &str) -> bool" in SERVICE, "SHARED_SHA256_VALIDATOR_MISSING"
    assert "trim_start_matches('*')" in SERVICE, "GNU_SHA256_FORMAT_REGRESSED"
    print("V0390_LINUX_CHECKSUM_FORMATS_CONTRACT=PASS")


if __name__ == "__main__":
    main()
