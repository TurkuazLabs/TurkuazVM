# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0411_vm_workspace_usability_contract.py
# 📌 Amac: v0.41.1 bos-filo onboarding, buyuk ekran olcekleme ve Goruntu Merkezi alan kullanim kontratini fail-closed dogrular
# 📌 Modul - Python
# Version: 0.41.1
# Aciklama: 1920x1080 ekranlarda gereksiz bos alanin ve tekrar eden bos VM panellerinin geri donmesini engeller
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
    index = read("apps/desktop/ui/index.html")
    app = read("apps/desktop/ui/app.js")
    view = read("apps/desktop/ui/views/vm_workspace_view.js")
    styles = read("apps/desktop/ui/styles.css")
    workspace_styles = read("apps/desktop/ui/views/workspace.css")

    for token in (
        'id="vm-workspace-shell"',
        'id="fleet-empty-workspace"',
        'data-empty-create',
        'data-empty-images',
        'Kontrol merkezi kullanima hazir',
    ):
        require(token in index, f"VM_USABILITY_INDEX_TOKEN_MISSING:{token}")

    for token in (
        'classList.toggle("is-empty-fleet", !hasFleet)',
        'querySelectorAll("[data-empty-create]")',
        'querySelector("[data-empty-images]")',
        'dataset.homeAction === "create"',
        'dataset.homeAction === "machines"',
    ):
        require(token in app, f"VM_USABILITY_APP_TOKEN_MISSING:{token}")

    for token in (
        'class="home-action-strip"',
        'data-home-action="create"',
        'data-home-action="images"',
        'data-home-action="machines"',
    ):
        require(token in view, f"VM_USABILITY_HOME_TOKEN_MISSING:{token}")

    for token in (
        ".vm-workspace-shell.is-empty-fleet",
        ".fleet-empty-workspace",
        ".fleet-empty-steps",
        "body:not(.expert-mode) .image-center-standard-iso",
        "@media (min-width: 1500px)",
    ):
        require(token in styles, f"VM_USABILITY_STYLE_TOKEN_MISSING:{token}")

    for token in (
        ".home-action-strip",
        ".home-action-card",
        "@media (min-width: 1500px)",
        ".vm-command-card",
    ):
        require(token in workspace_styles, f"VM_USABILITY_WORKSPACE_STYLE_MISSING:{token}")

    require("width: min(1040px, 100%);" in styles, "IMAGE_CENTER_STANDARD_WIDTH_NOT_EXPANDED")
    require("grid-template-columns: 360px minmax(0,1fr);" in styles, "LARGE_SCREEN_VM_LIBRARY_WIDTH_MISSING")

    print("V0411_VM_WORKSPACE_USABILITY_CONTRACT=PASS")


if __name__ == "__main__":
    main()
