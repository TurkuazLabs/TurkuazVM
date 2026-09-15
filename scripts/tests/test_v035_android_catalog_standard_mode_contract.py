# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v035_android_catalog_standard_mode_contract.py
# 📌 Amac: v0.35.0 tam Android release katalogu ve sade Standart Mod kontratini fail-closed dogrular
# 📌 Modul - Python
# Version: 0.35.0
# Aciklama: Android 10-17/12L katalog kapsamini, Standart Mod tek secici/aksiyonunu ve Uzman Mod teknik ayrimini test eder
# Bagimli Oldugu Katman: Tool | View | Repo

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

    require(catalog, (
        "id: android-17-gaming-phone", "id: android-16-gaming-phone", "id: android-15-phone",
        "id: android-14-phone", "id: android-13-phone", "id: android-12l-tablet",
        "id: android-12-phone", "id: android-11-phone", "id: android-10-phone",
    ), "android guest catalog")

    require(html, (
        'id="android-standard-release-select"', 'id="android-standard-release-action"',
        'id="android-standard-release-note"', 'class="android-release-grid expert-only"',
        'id="logs-button" class="nav-item expert-only"', 'host-picker expert-only',
        'id="refresh-button" class="toolbar-button expert-only"',
        'android-image-quick-assign expert-only', 'snapshot-list expert-only',
    ), "desktop html")

    require(js, (
        'const ANDROID_RELEASE_ORDER = Object.freeze(["17", "16", "15", "14", "13", "12L", "12", "11", "10"]);',
        "function normalizeAndroidRelease(value)", "function renderAndroidStandardRelease(releases, images, selected)",
        'document.body.classList.toggle("expert-mode", uiState.expertMode);',
        "function handleAndroidStandardReleaseChange()", "function handleAndroidStandardReleaseAction()",
        'elements.androidStandardReleaseSelect?.addEventListener("change", handleAndroidStandardReleaseChange);',
        'elements.androidStandardReleaseAction?.addEventListener("click", handleAndroidStandardReleaseAction);',
        "Android ${release.releaseId} icin surume sabitlenmis resmi kaynak kullanilir.",
    ), "desktop javascript")

    require(css, (
        "body:not(.expert-mode) .expert-only", "body.expert-mode .standard-only", ".android-standard-release {", ".android-standard-summary {",
        "body:not(.expert-mode) .android-release-center", "body:not(.expert-mode) #android-image-flow-hint",
        "body:not(.expert-mode) .task-dock:not(.has-active) #task-active-chip",
    ), "desktop css")

    print("V035_ANDROID_CATALOG_STANDARD_MODE_CONTRACT=PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
