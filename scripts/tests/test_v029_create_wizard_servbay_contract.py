# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v029_create_wizard_servbay_contract.py
# 📌 Amac: ServBay create wizard temel gorunum kontratinin yeni step wizard icinde korunmasini dogrular
# 📌 Modul - Python
# Version: 0.31.0
# Aciklama: Canli profil ozeti, kaynak kartlari ve create summary baglantilarinin v0.31 step wizard tarafinda korunmasini denetler
# Bagimli Oldugu Katman: Tool | View

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
HTML = (ROOT / "apps" / "desktop" / "ui" / "index.html").read_text(encoding="utf-8")
CSS = (ROOT / "apps" / "desktop" / "ui" / "styles.css").read_text(encoding="utf-8")
JS = (ROOT / "apps" / "desktop" / "ui" / "app.js").read_text(encoding="utf-8")

for token in (
    'id="create-selected-family"', 'id="create-selected-product"', 'id="create-selected-release"',
    'id="create-selected-profile"', 'id="create-selected-architecture"', 'id="create-selected-firmware"',
    'id="create-recommended-disk"', 'id="create-next-disk"', 'id="create-next-media"', 'id="create-next-network"',
):
    if token not in HTML:
        raise SystemExit(f"V029_CREATE_WIZARD_HTML_MISSING: {token}")
for token in ('.create-resource-grid', '.create-next-plan-grid', '.create-servbay-section'):
    if token not in CSS:
        raise SystemExit(f"V029_CREATE_WIZARD_CSS_MISSING: {token}")
for token in ('updateCreateServbaySummary', 'createSelectedFamily', 'createRecommendedDisk', 'createNextMedia'):
    if token not in JS:
        raise SystemExit(f"V029_CREATE_WIZARD_JS_MISSING: {token}")
print("V029_CREATE_WIZARD_SERVBAY_CONTRACT=PASS")
