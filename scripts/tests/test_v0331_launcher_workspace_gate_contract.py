# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0331_launcher_workspace_gate_contract.py
# 📌 Amac: v0.33.1 launcher structure gate ile v0.33 Workspace kontratinin ayni GUI mimarisini dogruladigini kontrol eder
# 📌 Modul - Python
# Version: 0.33.1
# Aciklama: Eski resource-sidebar zorunlulugunun launcher preflight akisini bloke etmesini engelleyen regression testidir
# Bagimli Oldugu Katman: Tool | View

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
VERIFY = ROOT / "scripts" / "verify_structure.ps1"
HTML = ROOT / "apps" / "desktop" / "ui" / "index.html"


def main() -> int:
    verify = VERIFY.read_text(encoding="utf-8")
    html = HTML.read_text(encoding="utf-8")

    legacy_installer_contract = (
        '"installer-media-recommended-label", "resource-sidebar"'
    )
    if legacy_installer_contract in verify:
        raise AssertionError("legacy resource-sidebar installer-media gate still present")

    required_workspace_tokens = (
        '"vm-library-pane", "vm-detail-panel", "task-dock", "expert-mode-button"',
        'DESKTOP_WORKSPACE_VIEW_MISSING',
        'LEGACY_RESOURCE_SIDEBAR_PRESENT',
    )
    missing = [token for token in required_workspace_tokens if token not in verify]
    if missing:
        raise AssertionError(f"workspace verifier missing: {missing}")

    if 'class="resource-sidebar"' in html:
        raise AssertionError("legacy resource sidebar returned to desktop html")

    for token in ('class="vm-library-pane"', 'id="vm-detail-panel"', 'class="task-dock"', 'id="expert-mode-button"'):
        if token not in html:
            raise AssertionError(f"desktop workspace token missing: {token}")

    print("V0331_LAUNCHER_WORKSPACE_GATE_CONTRACT=PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
