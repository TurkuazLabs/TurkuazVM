# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v034_android_release_taskbar_contract.py
# 📌 Amac: v0.34.0 Android Surum Merkezi ve alt Gorev Cubugu kontratini fail-closed dogrular
# 📌 Modul - Python
# Version: 0.34.0
# Aciklama: Android VM ISO yonlendirme, surum-uyumlu image atama, otomatik guncel image akisi ve taskbar katman kontratini test eder
# Bagimli Oldugu Katman: Tool | View | Service

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
HTML = ROOT / "apps" / "desktop" / "ui" / "index.html"
JS = ROOT / "apps" / "desktop" / "ui" / "app.js"
CSS = ROOT / "apps" / "desktop" / "ui" / "styles.css"
CATALOG = ROOT / "config" / "guest-catalog.yml"


def require(text: str, tokens: tuple[str, ...], label: str) -> None:
    missing = [token for token in tokens if token not in text]
    if missing:
        raise AssertionError(f"{label} missing: {missing}")


def main() -> int:
    html = HTML.read_text(encoding="utf-8")
    js = JS.read_text(encoding="utf-8")
    css = CSS.read_text(encoding="utf-8")
    catalog = CATALOG.read_text(encoding="utf-8")

    require(html, (
        'id="android-release-catalog"',
        'id="android-release-context"',
        'id="android-release-refresh"',
        'id="task-dock-toggle"',
        'id="task-dock-panel"',
        'id="task-active-count"',
        'id="task-error-count"',
        'id="task-latest-label"',
        'Android Surumleri',
        'Gelismis Image Registry',
    ), "desktop html")

    require(js, (
        'if (machine.guest_profile === "android") {',
        'openAndroidImagesForVm(vmId, { vmId, fromAndroid: true });',
        'function androidReleaseEntries()',
        'function renderAndroidReleaseCatalog(images = uiState.androidImages)',
        'function ensureAndroidRelease(releaseId)',
        'function completePendingAndroidAutoAssign(images = uiState.androidImages)',
        'function toggleTaskDock(forceOpen = null)',
        'function renderTaskDockStatus()',
        'data-android-release-action="auto"',
        'Otomatik Indir ve Ata',
        'selected.releaseId === release.releaseId',
        'Android image surumu VM profiliyle uyusmuyor.',
    ), "desktop javascript")

    require(css, (
        '.task-dock {',
        'left: 224px;',
        'z-index: 49;',
        'inset: 66px 0 44px 224px;',
        '.taskbar-main',
        '.task-dock-panel',
        '.android-release-grid',
        '.android-release-card.selected',
        '.android-version-badge',
    ), "desktop css")

    require(catalog, (
        'id: android-17-gaming-phone',
        'id: android-16-gaming-phone',
        'id: android-15-phone',
        'source_kind: android_image',
    ), "guest catalog")

    print("V034_ANDROID_RELEASE_TASKBAR_CONTRACT=PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
