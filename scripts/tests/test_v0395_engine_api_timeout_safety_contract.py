# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0395_engine_api_timeout_safety_contract.py
# 📌 Amac: Engine API timeout sonrasi mutating request retry ve false startup regresyonunu engeller
# 📌 Modul - Python
# Version: 0.39.5
# Aciklama: Client hata asamalari, Ping readiness, long wizard timeout ve silent curl kontratlarini statik dogrular
# Bagimli Oldugu Katman: Service | Tool | Config

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
client = (ROOT / "apps/desktop/src-tauri/src/tools/engine_api_client_tool.rs").read_text(encoding="utf-8")
desktop = (ROOT / "apps/desktop/src-tauri/src/services/desktop_service.rs").read_text(encoding="utf-8")
download = (ROOT / "apps/engine/src/services/installer_media_download_application_service.rs").read_text(encoding="utf-8")
http = (ROOT / "crates/guest/src/tools/http_download_tool.rs").read_text(encoding="utf-8")
config = (ROOT / "config/turkuazvm.yml").read_text(encoding="utf-8")

assert "Connect(String)" in client
assert "Write(String)" in client
assert "Read(String)" in client
assert "action: EngineAction::Ping" in desktop
assert "send_after_readiness" in desktop
assert "Request tekrar gonderilmedi" in desktop
assert "EngineAction::CreateVmDisk" in desktop
assert "EngineAction::AttachNetworkProfile" in desktop
assert "EngineAction::AttachDownloadedInstallerMedia" in desktop
assert '.arg("--silent")' in http
status_block = download.split("pub fn status", 1)[1].split("pub fn ready_path", 1)[0]
assert status_block.index(".jobs") < status_block.index("self.source_request")
assert "request_timeout_ms: 5000" in config
assert "long_request_timeout_ms: 300000" in config
print("v0.39.5 engine API timeout safety contract: PASS")
