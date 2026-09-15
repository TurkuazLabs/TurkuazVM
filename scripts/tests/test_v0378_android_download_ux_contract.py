# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0378_android_download_ux_contract.py
# 📌 Amac: Yeni VM wizard Android image indirme telemetry, animasyon ve gercek progress kontratini korur
# 📌 Modul - Python
# Version: 0.37.8
# Aciklama: Engine progress alanlarinin yuzde, byte, hiz, ETA, asama, iptal ve log aksiyonlariyla View katmanina baglandigini fail-closed dogrular
# Bagimli Oldugu Katman: Service | Tool | View

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
HTML = (ROOT / "apps/desktop/ui/index.html").read_text(encoding="utf-8")
JS = (ROOT / "apps/desktop/ui/app.js").read_text(encoding="utf-8")
CSS = (ROOT / "apps/desktop/ui/styles.css").read_text(encoding="utf-8")
DOWNLOAD_VIEW = (ROOT / "apps/desktop/ui/views/download_progress_view.js").read_text(encoding="utf-8") if (ROOT / "apps/desktop/ui/views/download_progress_view.js").is_file() else ""
API = (ROOT / "crates/engine-api/src/lib.rs").read_text(encoding="utf-8")


def main() -> None:
    for token in (
        'id="create-android-download-progress"',
        'id="create-android-progress-track"',
        'id="create-android-progress-bar"',
        'id="create-android-progress-percent"',
        'id="create-android-progress-bytes"',
        'id="create-android-progress-speed"',
        'id="create-android-progress-eta"',
        'id="create-android-progress-elapsed"',
        'id="create-android-progress-cancel"',
        'id="create-android-progress-log"',
    ):
        assert token in HTML, f"ANDROID_DOWNLOAD_UX_HTML_MISSING: {token}"

    for token in (
        "androidInstallProgressMetrics",
        "renderCreateAndroidImageProgress",
        "progress?.downloaded_bytes",
        "progress?.total_bytes",
        "progress?.bytes_per_second",
        "progress?.eta_seconds",
        "progress?.elapsed_seconds",
        'invoke("cancel_android_image_distribution"',
        'invoke("open_android_image_install_log"',
        "ANDROID_INSTALL_ACTIVE_STAGE_ORDER",
        "1000);",
    ):
        assert token in JS, f"ANDROID_DOWNLOAD_UX_JS_MISSING: {token}"

    for token in (
        ".download-progress-track.is-indeterminate > span",
        "@keyframes download-progress-scan",
        ".download-progress-stats",
        ".ghost-button.is-busy::after",
        "prefers-reduced-motion",
    ):
        assert token in CSS, f"ANDROID_DOWNLOAD_UX_CSS_MISSING: {token}"

    assert "TurkuazVmDownloadProgressView" in DOWNLOAD_VIEW, "ANDROID_DOWNLOAD_SHARED_VIEW_MISSING"

    assert 'track.setAttribute("aria-valuenow"' in DOWNLOAD_VIEW, "ANDROID_DOWNLOAD_ARIA_PROGRESS_MISSING"

    for token in (
        "pub downloaded_bytes: u64",
        "pub total_bytes: Option<u64>",
        "pub bytes_per_second: Option<u64>",
        "pub eta_seconds: Option<u64>",
        "pub elapsed_seconds: u64",
        "pub log_path: String",
    ):
        assert token in API, f"ANDROID_PROGRESS_API_FIELD_MISSING: {token}"

    assert 'percent = 45' not in JS[JS.index('function androidInstallProgressMetrics'):JS.index('async function refreshCreateAndroidImageOptions')], "ANDROID_PROGRESS_FAKE_PERCENT_NOT_ALLOWED"
    print("V0378_ANDROID_DOWNLOAD_UX_CONTRACT=PASS")


if __name__ == "__main__":
    main()
