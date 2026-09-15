# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_installer_media_reconfigure_contract.py
# 📌 Amac: Installer ISO yeniden baglama ve guvenli replace kontratini fail-closed dogrular
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Ayni ISO no-op, farkli ISO backup replace ve rollback davranislarinin kaynak kodda mevcut oldugunu dogrular
# Bagimli Oldugu Katman: Service | Tool

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def require(path: str, tokens: tuple[str, ...]) -> None:
    text = (ROOT / path).read_text(encoding="utf-8")
    missing = [token for token in tokens if token not in text]
    if missing:
        raise AssertionError(f"{path}: missing tokens: {missing}")


def main() -> int:
    require(
        "crates/core/src/ports/media_port.rs",
        (
            "pub enum MediaImportState",
            "Created",
            "Unchanged",
            "Replaced",
            "fn commit_iso",
            "fn rollback_iso",
        ),
    )
    require(
        "crates/guest/src/tools/local_guest_media_tool.rs",
        (
            "Self::sha256(source_path)? == Self::sha256(&target)?",
            "MediaImportState::Unchanged",
            "MediaImportState::Replaced",
            "BACKUP_SUFFIX",
            "TEMP_SUFFIX",
            "fs::rename(&target, &backup)",
            "fs::rename(&backup, &target)",
            "same_iso_is_idempotent",
            "replacement_rollback_restores_previous_iso",
        ),
    )
    require(
        "crates/core/src/services/guest_boot_service.rs",
        (
            "media_import_state",
            "rollback_iso",
            "commit_iso",
        ),
    )
    print("INSTALLER_MEDIA_RECONFIGURE_CONTRACT_OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
