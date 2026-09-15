# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0380_unified_download_ux_contract.py
# 📌 Amac: Android image ve Windows/Linux ISO indirmelerinin tek ortak Download Progress View kontratini kullandigini dogrular
# 📌 Modul - Python
# Version: 0.38.0
# Aciklama: Wizard, Kurulum Medyasi ve Goruntu Merkezi indirmelerinde ortak animasyon, yuzde, byte, hiz, ETA, sure ve asama UI'sini fail-closed korur
# Bagimli Oldugu Katman: View | Controller | Service

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
HTML = (ROOT / "apps/desktop/ui/index.html").read_text(encoding="utf-8")
JS = (ROOT / "apps/desktop/ui/app.js").read_text(encoding="utf-8")
CSS = (ROOT / "apps/desktop/ui/styles.css").read_text(encoding="utf-8")
VIEW = (ROOT / "apps/desktop/ui/views/download_progress_view.js").read_text(encoding="utf-8")


def main() -> None:
    progress_roots = (
        "image-center-iso-download-progress",
        "android-standard-download-progress",
        "installer-media-download-progress",
        "create-media-download-progress",
        "create-android-download-progress",
    )
    for root_id in progress_roots:
        token = f'id="{root_id}" class="download-progress-card hidden"'
        assert token in HTML, f"UNIFIED_DOWNLOAD_ROOT_MISSING: {root_id}"

    for role in ("title", "stage", "percent", "track", "bar", "bytes", "speed", "eta", "elapsed", "detail"):
        assert f'data-download-role="{role}"' in HTML, f"UNIFIED_DOWNLOAD_ROLE_MISSING: {role}"

    assert '<script src="./views/download_progress_view.js"></script>' in HTML, "UNIFIED_DOWNLOAD_VIEW_SCRIPT_MISSING"
    assert HTML.index('./views/download_progress_view.js') < HTML.index('./app.js'), "UNIFIED_DOWNLOAD_VIEW_LOAD_ORDER_INVALID"

    for token in (
        "TurkuazVmDownloadProgressView",
        'root.classList.toggle("is-failed"',
        'track.classList.toggle("is-indeterminate"',
        'track.setAttribute("aria-valuenow"',
        'setText(root, "speed"',
        'setText(root, "eta"',
        'setText(root, "elapsed"',
    ):
        assert token in VIEW, f"UNIFIED_DOWNLOAD_VIEW_TOKEN_MISSING: {token}"

    for token in (
        "downloadProgressTelemetry",
        "installerMediaProgressModel",
        "androidDownloadProgressModel",
        "downloadProgressView?.render(elements.installerMediaDownloadProgress",
        "downloadProgressView?.render(elements.createMediaDownloadProgress",
        "downloadProgressView?.render(elements.createAndroidDownloadProgress",
        "renderImageCenterIsoDownloadProgress",
        "downloadProgressView?.render(elements.androidStandardDownloadProgress",
        "bytesPerSecond",
        "etaSeconds",
        "elapsedSeconds",
    ):
        assert token in JS, f"UNIFIED_DOWNLOAD_CONTROLLER_TOKEN_MISSING: {token}"

    for token in (
        ".download-progress-card",
        ".download-progress-head",
        ".download-progress-percent",
        ".download-progress-track.is-indeterminate > span",
        ".download-progress-stats",
        ".download-progress-detail",
        "@keyframes download-progress-scan",
        "prefers-reduced-motion",
    ):
        assert token in CSS, f"UNIFIED_DOWNLOAD_CSS_TOKEN_MISSING: {token}"

    assert ".android-create-progress" not in CSS, "ANDROID_SPECIFIC_PROGRESS_CSS_REMAINS"
    assert ".download-progress-meta" not in CSS, "LEGACY_ISO_PROGRESS_CSS_REMAINS"
    iso_block = JS[JS.index("function renderInstallerMediaDownload"):JS.index("function normalizeInstallerArchitecture")]
    create_iso_block = JS[JS.index("function renderCreateMediaDownload"):JS.index("async function refreshCreateMediaDownloadStatus")]
    assert "percent = 45" not in iso_block + create_iso_block, "UNIFIED_DOWNLOAD_FAKE_45_PERCENT_NOT_ALLOWED"
    assert "percent = 88" not in iso_block + create_iso_block, "UNIFIED_DOWNLOAD_FAKE_88_PERCENT_NOT_ALLOWED"

    print("V0380_UNIFIED_DOWNLOAD_UX_CONTRACT=PASS")


if __name__ == "__main__":
    main()
