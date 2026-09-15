# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0410_vm_control_center_contract.py
# 📌 Amac: v0.41.0 VM Kontrol Merkezi DOM, aksiyon ve responsive stil kontratini fail-closed dogrular
# 📌 Modul - Python
# Version: 0.41.0
# Aciklama: Kontrol-oncelikli VM detail, hizli islem kartlari, hazirlik paneli ve gorunur VM durum etiketlerinin gerilemesini engeller
# Bagimli Oldugu Katman: View | Tool

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def require(condition: bool, code: str) -> None:
    if not condition:
        raise AssertionError(code)


def main() -> None:
    view = read("apps/desktop/ui/views/vm_workspace_view.js")
    app = read("apps/desktop/ui/app.js")
    styles = read("apps/desktop/ui/styles.css")
    workspace_styles = read("apps/desktop/ui/views/workspace.css")
    index = read("apps/desktop/ui/index.html")

    for token in (
        'class="vm-detail-header vm-command-center"',
        'class="vm-primary-controls"',
        'class="vm-fact-strip"',
        'class="vm-command-grid"',
        'class="vm-readiness-card"',
        'data-action="connect"',
        'data-action="add-disk"',
        'data-action="network"',
        'data-action="snapshots"',
        'VM Yonetimi',
    ):
        require(token in view, f"VM_CONTROL_VIEW_TOKEN_MISSING:{token}")

    require('class="vm-overview-grid"' not in view, "LEGACY_OVERVIEW_PREVIEW_RETURNED")
    require('vm-list-state-label' in app, "VM_LIST_STATE_LABEL_MISSING")
    require('VM Kontrol Merkezi' in index, "VM_CONTROL_CENTER_HEADING_MISSING")

    for token in (
        ".vm-detail-header.vm-command-center",
        ".vm-primary-controls",
        ".vm-control-button",
        ".vm-fact-strip",
        ".vm-list-state-label",
    ):
        require(token in styles, f"VM_CONTROL_SHELL_STYLE_MISSING:{token}")

    for token in (
        ".vm-overview-control-layout",
        ".vm-command-grid",
        ".vm-command-card",
        ".vm-readiness-card",
        ".vm-overview-danger",
    ):
        require(token in workspace_styles, f"VM_CONTROL_VIEW_STYLE_MISSING:{token}")

    for breakpoint in ("1180px", "860px", "620px"):
        require(breakpoint in styles, f"VM_CONTROL_RESPONSIVE_BREAKPOINT_MISSING:{breakpoint}")

    print("V0410_VM_CONTROL_CENTER_CONTRACT=PASS")


if __name__ == "__main__":
    main()
