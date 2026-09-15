# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v027_display_ui_contract.py
# 📌 Amac: v0.27.0 console display bootstrap ve kompakt Desktop UX kontratini dogrular
# 📌 Modul - Python
# Version: 0.33.0
# Aciklama: virtio-vga bootstrap, Konsol/Baglan aksiyonlari, guncel navigation ve guest catalog secim zincirini statik test eder
# Bagimli Oldugu Katman: Tool | View

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
QEMU = (ROOT / "crates/qemu/src/tools/qemu_command_builder.rs").read_text(encoding="utf-8")
PROBE = (ROOT / "crates/qemu/src/tools/qemu_gpu_probe_tool.rs").read_text(encoding="utf-8")
HTML = (ROOT / "apps/desktop/ui/index.html").read_text(encoding="utf-8")
JS = (ROOT / "apps/desktop/ui/app.js").read_text(encoding="utf-8")
WORKSPACE_VIEW = (ROOT / "apps/desktop/ui/views/vm_workspace_view.js").read_text(encoding="utf-8")
UI_SOURCE = JS + "\n" + WORKSPACE_VIEW

assert 'const DEVICE_VIRTIO_GPU: &str = "virtio-vga";' in QEMU
assert 'const DEVICE_VIRTIO_GPU_GL: &str = "virtio-vga-gl";' in QEMU
assert 'DEVICE_VIRTIO_VGA' in PROBE and 'DEVICE_VIRTIO_VGA_GL' in PROBE
for element_id in (
    "guest-family-select", "guest-product-select", "guest-release-select", "guest-profile-select", "guest-architecture",
    "connection-modal", "network-profile",
):
    assert f'id="{element_id}"' in HTML, element_id
for label in ("Sanal Makineler", "Depolama", "Ayarlar", "Konsol", "Baglanti Merkezi"):
    assert label in HTML or label in JS, label
assert "guest-family-grid" not in HTML
assert 'data-action="display"' in UI_SOURCE and ('>Konsol</button>' in UI_SOURCE or '>Konsolu Ac</button>' in UI_SOURCE)
assert 'data-action="connect"' in UI_SOURCE and '>Baglan</button>' in UI_SOURCE
assert "function openConnectionModal" in JS
assert "function updateNetworkProfileFields" in JS
print("V027_DISPLAY_UI_CONTRACT_OK")
