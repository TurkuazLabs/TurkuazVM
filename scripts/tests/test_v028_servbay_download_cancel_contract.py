# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v028_servbay_download_cancel_contract.py
# 📌 Amac: Installer ISO download cancel zincirini ve guncel Desktop workspace entegrasyonunu fail-closed dogrular
# 📌 Modul - Python
# Version: 0.39.7
# Aciklama: Engine API v24 cancel action, curl child iptali, part cleanup ve guncel workspace yapisini olcu magic-value olmadan test eder
# Bagimli Oldugu Katman: Controller | Service | Tool | View

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def require(text: str, tokens: tuple[str, ...], label: str) -> None:
    missing = [token for token in tokens if token not in text]
    if missing:
        raise AssertionError(f"{label} missing: {missing}")


def main() -> int:
    api = (ROOT / "crates/engine-api/src/lib.rs").read_text(encoding="utf-8")
    engine_service = (ROOT / "apps/engine/src/services/engine_application_service.rs").read_text(encoding="utf-8")
    download_service = (ROOT / "apps/engine/src/services/installer_media_download_application_service.rs").read_text(encoding="utf-8")
    desktop_service = (ROOT / "apps/desktop/src-tauri/src/services/desktop_service.rs").read_text(encoding="utf-8")
    controller = (ROOT / "apps/desktop/src-tauri/src/controllers/desktop_controller.rs").read_text(encoding="utf-8")
    main_rs = (ROOT / "apps/desktop/src-tauri/src/main.rs").read_text(encoding="utf-8")
    html = (ROOT / "apps/desktop/ui/index.html").read_text(encoding="utf-8")
    js = (ROOT / "apps/desktop/ui/app.js").read_text(encoding="utf-8")
    css = (ROOT / "apps/desktop/ui/styles.css").read_text(encoding="utf-8")

    require(api, ("ENGINE_API_VERSION: u16 = 24", "CancelInstallerMediaDownload"), "engine api")
    require(engine_service, ("EngineAction::CancelInstallerMediaDownload", ".cancel(&guest_template_id, &media_id)"), "engine service")
    require(
        download_service,
        (
            "InstallerMediaDownloadState::Cancelling",
            "InstallerMediaDownloadState::Cancelled",
            "cancel_requested",
            "child.kill()",
            "mark_cancelled",
            "fs::remove_file(part_path)",
            "sha256_file_cancelable",
        ),
        "download service",
    )
    require(desktop_service, ("cancel_installer_media_download", "EngineAction::CancelInstallerMediaDownload"), "desktop service")
    require(controller, ("pub fn cancel_installer_media_download", ".cancel_installer_media_download"), "desktop controller")
    require(main_rs, ("cancel_installer_media_download,",), "desktop composition")

    require(
        html,
        (
            'class="app-shell workspace-v2"',
            'id="vm-detail-panel"',
            'id="resource-engine-status"',
            'id="installer-media-cancel-download-button"',
            "Indirmeyi Durdur",
        ),
        "desktop html",
    )
    require(
        js,
        (
            'invoke("cancel_installer_media_download"',
            "cancelInstallerMediaDownload",
            'cancelling: "Indirme durduruluyor"',
            'cancelled: "Indirme durduruldu"',
            "vmViewMode: VM_VIEW_COMPACT",
        ),
        "desktop javascript",
    )
    require(
        css,
        (
            ".vm-workspace-shell",
            ".vm-library-list",
            ".danger-button",
            ".content-shell",
            ".machine-grid.compact-view",
        ),
        "desktop css",
    )

    print("V028_SERVBAY_DOWNLOAD_CANCEL_CONTRACT_OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
