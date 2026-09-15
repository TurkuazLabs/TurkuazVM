# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v030_create_wizard_premium_contract.py
# 📌 Amac: Premium ServBay create wizard gorunum detaylarinin step-by-step akista korunmasini dogrular
# 📌 Modul - Python
# Version: 0.31.0
# Aciklama: Ikonlu bolum basliklari, sticky action bar ve kompakt kaynak kartlarini regression olarak korur
# Bagimli Oldugu Katman: Tool | View

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
HTML = (ROOT / "apps" / "desktop" / "ui" / "index.html").read_text(encoding="utf-8")
CSS = (ROOT / "apps" / "desktop" / "ui" / "styles.css").read_text(encoding="utf-8")
JS = (ROOT / "apps" / "desktop" / "ui" / "app.js").read_text(encoding="utf-8")

for token in (
    'class="create-heading-icon">OS</span>',
    'class="create-heading-icon">ID</span>',
    'class="create-heading-icon">CPU</span>',
    'class="create-heading-icon">OK</span>',
    'class="modal-actions create-servbay-actions create-step-actions"',
):
    if token not in HTML:
        raise SystemExit(f"V030_CREATE_HTML_MISSING: {token}")
for token in ('.create-heading-icon', '.create-step-actions', 'position: sticky;', 'backdrop-filter: blur(10px);'):
    if token not in CSS:
        raise SystemExit(f"V030_CREATE_CSS_MISSING: {token}")
for token in ('function wireCreateFlowNavigation', 'wireCreateFlowNavigation();', 'showCreateStep("catalog")'):
    if token not in JS:
        raise SystemExit(f"V030_CREATE_JS_MISSING: {token}")
print("V030_CREATE_WIZARD_PREMIUM_CONTRACT=PASS")
