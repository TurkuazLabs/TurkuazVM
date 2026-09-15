# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v033_workspace_contract.py
# 📌 Amac: v0.33.0 Desktop Workspace Redesign kontratini fail-closed dogrular
# 📌 Modul - Python
# Version: 0.34.0
# Aciklama: Tek navigation, VM Library, VM Detail Workspace, kalici gorev cubugu ve uc adimli VM wizard yapisini test eder
# Bagimli Oldugu Katman: View | Tool

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
HTML = ROOT / "apps" / "desktop" / "ui" / "index.html"
JS = ROOT / "apps" / "desktop" / "ui" / "app.js"
CSS = ROOT / "apps" / "desktop" / "ui" / "styles.css"


def require(text: str, tokens: tuple[str, ...], label: str) -> None:
    missing = [token for token in tokens if token not in text]
    if missing:
        raise AssertionError(f"{label} missing: {missing}")


def forbid(text: str, tokens: tuple[str, ...], label: str) -> None:
    found = [token for token in tokens if token in text]
    if found:
        raise AssertionError(f"{label} forbidden: {found}")


def main() -> int:
    html = HTML.read_text(encoding="utf-8")
    js = JS.read_text(encoding="utf-8")
    css = CSS.read_text(encoding="utf-8")

    require(
        html,
        (
            'class="app-shell workspace-v2"',
            'class="vm-library-pane"',
            'id="vm-detail-panel"',
            'class="task-dock"',
            'class="modal-backdrop workspace-page-modal hidden" role="region"',
            'id="expert-mode-button"',
            'data-create-progress="catalog"',
            'data-create-progress="resource"',
            'data-create-progress="summary"',
            '>1 Sistem<',
            '>2 Kaynaklar<',
            '>3 Olustur<',
        ),
        "desktop html",
    )
    forbid(
        html,
        (
            'class="resource-sidebar"',
            'data-create-progress="identity"',
            'data-create-progress="disk"',
            'data-create-progress="media"',
            'data-create-progress="network"',
        ),
        "legacy desktop html",
    )
    require(
        js,
        (
            'const CREATE_STEP_ORDER = Object.freeze(["catalog", "resource", "summary"]);',
            'selectedVmTab: "overview"',
            'expertMode: false',
            'function setExpertMode(enabled, persist = true)',
            'const workspacePageMap = new Map([',
            "function renderVmDetail()",
            'action === "select"',
            'data-detail-tab',
            'function renderTaskDockStatus()',
        ),
        "desktop javascript",
    )
    require(
        css,
        (
            ".workspace-v2",
            ".vm-workspace-shell",
            ".vm-library-list .vm-card",
            ".vm-detail-panel",
            ".detail-tabs",
            ".taskbar-main",
            ".create-wizard-v2",
            ".create-progress-v2",
            ".workspace-v2:not(.expert-mode) .expert-only",
            ".workspace-page-modal",
        ),
        "desktop css",
    )

    print("V033_WORKSPACE_CONTRACT=PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
