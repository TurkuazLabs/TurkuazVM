# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_local_log_viewer_contract.py
# 📌 Amac: Local log viewer ve Android install log katalog kontratlarini regression olarak dogrular
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Generic viewer ile Android install log ownership kontrolunun yanlis dosyada birlesmesini engeller
# Bagimli Oldugu Katman: Tool | View

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
VIEWER = ROOT / "apps/desktop/src-tauri/src/tools/local_file_viewer_tool.rs"
CATALOG = ROOT / "apps/desktop/src-tauri/src/tools/local_log_catalog_tool.rs"
VERIFIER = ROOT / "scripts/verify_structure.ps1"


def require(content: str, tokens: tuple[str, ...], label: str) -> None:
    missing = [token for token in tokens if token not in content]
    if missing:
        raise SystemExit(f"{label}_MISSING: {missing}")


def main() -> None:
    viewer = VIEWER.read_text(encoding="utf-8")
    catalog = CATALOG.read_text(encoding="utf-8")
    verifier = VERIFIER.read_text(encoding="utf-8")

    require(
        viewer,
        (
            "LocalFileViewerTool",
            "open_data_log",
            "DATA_DIRECTORY",
            "fs::canonicalize",
            "requested.starts_with(&data_root)",
            "requested.is_file()",
            '"log" | "txt"',
            "Command::new(WINDOWS_TEXT_VIEWER)",
        ),
        "LOCAL_LOG_VIEWER",
    )
    require(
        catalog,
        (
            "LocalLogCatalogTool",
            'data_root.join("logs")',
            "walk_install_logs",
            'Some("install.log")',
            '"android_image"',
        ),
        "LOCAL_LOG_CATALOG",
    )
    require(
        verifier,
        (
            "$LocalLogViewerContent",
            "LOCAL_LOG_VIEWER_CONTRACT_MISSING",
            "$LocalLogCatalogContent",
            "LOCAL_LOG_CATALOG_CONTRACT_MISSING",
        ),
        "STRUCTURE_VERIFIER",
    )

    viewer_gate_start = verifier.index("$LocalLogViewerContent")
    catalog_gate_start = verifier.index("$LocalLogCatalogContent")
    viewer_gate = verifier[viewer_gate_start:catalog_gate_start]
    if "install.log" in viewer_gate:
        raise SystemExit("LOCAL_LOG_VIEWER_STALE_INSTALL_LOG_LITERAL_PRESENT")

    print("LOCAL_LOG_VIEWER_CONTRACT_OK")


if __name__ == "__main__":
    main()
