# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v031_step_by_step_create_wizard_contract.py
# 📌 Amac: VM olusturma wizard orchestration kontratini guncel uc adimli UX ile regression olarak dogrular
# 📌 Modul - Python
# Version: 0.33.0
# Aciklama: Sistem Kaynaklar Olustur adimlarini, validasyonu ve final toplu create orchestration zincirini fail-closed denetler
# Bagimli Oldugu Katman: Controller | Service | Tool | View

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
HTML = (ROOT / "apps" / "desktop" / "ui" / "index.html").read_text(encoding="utf-8")
CSS = (ROOT / "apps" / "desktop" / "ui" / "styles.css").read_text(encoding="utf-8")
JS = (ROOT / "apps" / "desktop" / "ui" / "app.js").read_text(encoding="utf-8")

for token in (
    'data-create-progress="catalog">1 Sistem',
    'data-create-progress="resource">2 Kaynaklar',
    'data-create-progress="summary">3 Olustur',
    'id="create-step-catalog"',
    'id="create-step-resource"',
    'id="create-step-summary"',
    'id="create-back-button"',
    'id="create-next-button"',
    'id="create-submit" class="primary-button hidden"',
    'id="create-media-download-button"',
    'id="create-media-cancel-button"',
    'id="create-network-profile"',
    'class="create-advanced-panel"',
):
    if token not in HTML:
        raise SystemExit(f"V031_CREATE_HTML_MISSING: {token}")

for token in (
    '.create-page-step.hidden',
    '.create-step-actions-right',
    '.create-step-progress',
    '.create-disk-plan-card',
    '.create-media-panel',
    '.create-network-plan-grid',
    '.create-review-grid',
    '.create-wizard-v2',
    '.create-advanced-panel',
):
    if token not in CSS:
        raise SystemExit(f"V031_CREATE_CSS_MISSING: {token}")

for token in (
    'const CREATE_STEP_ORDER = Object.freeze(["catalog", "resource", "summary"])',
    'function showCreateStep(step)',
    'function validateCreateStep(step)',
    'async function handleCreateNext()',
    'function handleCreateBack()',
    'async function prepareCreateMediaStep()',
    'async function startCreateMediaDownload()',
    'async function cancelCreateMediaDownload()',
    'async function rollbackCreateWizardVm',
    'await invoke("create_vm", { request })',
    'await invoke("create_vm_disk"',
    'await invoke("attach_network_profile"',
    'await invoke("attach_downloaded_installer_media"',
    'await invoke("configure_installer_media"',
    'await invoke("assign_android_image"',
):
    if token not in JS:
        raise SystemExit(f"V031_CREATE_JS_MISSING: {token}")

for stale in (
    'data-create-progress="identity"',
    'data-create-progress="disk"',
    'data-create-progress="media"',
    'data-create-progress="network"',
    'class="create-flow-rail"',
    'function setCreateFlowFocus',
):
    if stale in HTML or stale in JS:
        raise SystemExit(f"V031_CREATE_STALE_WIZARD_PRESENT: {stale}")

print("V031_STEP_BY_STEP_CREATE_WIZARD_CONTRACT=PASS")
