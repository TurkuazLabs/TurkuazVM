# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0397_workspace_layout_contract.py
# 📌 Amac: Yeni VM Workspace listesinin legacy compact-grid CSS kolonlarindan etkilenmemesini dogrular
# 📌 Modul - Python
# Version: 0.39.7
# Aciklama: VM kutuphanesi DOM siniflarini, View-mode izolasyonunu ve okunabilir layout olculerini statik regression olarak kilitler
# Bagimli Oldugu Katman: Controller | View

from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[2]
index_html = (ROOT / "apps/desktop/ui/index.html").read_text(encoding="utf-8")
app_js = (ROOT / "apps/desktop/ui/app.js").read_text(encoding="utf-8")
styles = (ROOT / "apps/desktop/ui/styles.css").read_text(encoding="utf-8")
workspace_styles = (ROOT / "apps/desktop/ui/views/workspace.css").read_text(encoding="utf-8")

machine_grid_tag = re.search(r'<div id="machine-grid" class="([^"]+)"', index_html)
assert machine_grid_tag is not None
classes = machine_grid_tag.group(1).split()
assert "vm-library-list" in classes
assert "machine-grid" not in classes

assert 'const usesWorkspaceLibrary = elements.machineGrid.classList.contains("vm-library-list");' in app_js
assert 'classList.toggle("compact-view", !usesWorkspaceLibrary && uiState.vmViewMode === VM_VIEW_COMPACT)' in app_js

assert "grid-template-columns: minmax(360px, 390px) minmax(0, 1fr);" in styles
assert ".vm-library-list .vm-card" in styles
assert "grid-template-columns: 40px minmax(0,1fr) auto;" in styles
assert ".vm-list-main strong { font-size: 14px;" in styles
assert ".vm-list-sub { display: flex; gap: 8px; color: #758999; font-size: 11px;" in styles
assert ".detail-tab { min-height: 40px;" in styles
assert ".vm-library-list.compact-view .vm-card" in styles
assert "repeat(auto-fit,minmax(260px,1fr))" in workspace_styles
assert ".detail-secondary { min-height: 36px;" in styles

print("v0.39.7 workspace layout contract: PASS")
