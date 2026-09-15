# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_resource_naming_contract.py
# 📌 Amac: VM, disk ve ag kaynak kimliklerinin benzersiz otomatik isimlendirme kontratini dogrular
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Desktop onerileri ile Engine network action zincirinin VM tabanli adlari korudugunu statik olarak test eder
# Bagimli Oldugu Katman: Controller | Service | Tool | View

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
APP_JS = (ROOT / "apps/desktop/ui/app.js").read_text(encoding="utf-8")
INDEX = (ROOT / "apps/desktop/ui/index.html").read_text(encoding="utf-8")
API = (ROOT / "crates/engine-api/src/lib.rs").read_text(encoding="utf-8")
ENGINE = (ROOT / "apps/engine/src/services/engine_application_service.rs").read_text(encoding="utf-8")
CONTROLLER = (ROOT / "apps/desktop/src-tauri/src/controllers/desktop_controller.rs").read_text(encoding="utf-8")
SERVICE = (ROOT / "apps/desktop/src-tauri/src/services/desktop_service.rs").read_text(encoding="utf-8")

required_js = (
    "function suggestUniqueVmId(value)",
    "function nextResourceId(machine, kind)",
    "RESOURCE_KIND_DISK",
    "RESOURCE_KIND_NETWORK",
    "refreshStorageDiskSuggestion()",
    "refreshNetworkIdSuggestion()",
    'request: { vm_id: vmId, network_id: networkId }',
)
for token in required_js:
    assert token in APP_JS, f"resource naming JS token missing: {token}"

assert 'id="network-id"' in INDEX, "network id input missing"
assert 'Disk Adi / ID' in INDEX, "disk naming label missing"
assert 'NIC / Baglanti ID' in INDEX, "network naming label missing"
assert "AttachDefaultNetwork { vm_id: String, network_id: String }" in API, "Engine API named network contract missing"
assert "EngineAction::AttachDefaultNetwork { vm_id, network_id }" in ENGINE, "Engine named network handler missing"
assert "AttachDefaultNetworkRequest" in CONTROLLER, "Desktop named network request missing"
assert "attach_default_network(&mut self, vm_id: String, network_id: String)" in SERVICE, "Desktop service named network contract missing"
assert "ENGINE_API_VERSION: u16 = 24" in API, "Engine API v24 missing"

print("RESOURCE_NAMING_CONTRACT_OK")
