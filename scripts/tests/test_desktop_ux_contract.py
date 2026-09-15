# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_desktop_ux_contract.py
# 📌 Amac: Desktop UX navigasyon, filtre, toplu islem ve feedback contractlarini regression olarak dogrular
# 📌 Modul - Python
# Version: 0.30.0
# Aciklama: HTML ID, JavaScript selector, UX token ve islevsiz kontrol geri donuslerini fail-closed kontrol eder
# Bagimli Oldugu Katman: Tool | View

from __future__ import annotations

import re
from html.parser import HTMLParser
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
HTML_PATH = ROOT / "apps" / "desktop" / "ui" / "index.html"
SCRIPT_PATH = ROOT / "apps" / "desktop" / "ui" / "app.js"
STYLE_PATH = ROOT / "apps" / "desktop" / "ui" / "styles.css"

REQUIRED_VIEW_IDS = {
    "vm-filter-group",
    "vm-sort-select",
    "card-view-button",
    "compact-view-button",
    "bulk-start-button",
    "bulk-stop-button",
    "activity-list",
    "toast-stack",
    "shortcuts-modal",
    "sidebar-collapse-button",
}

REQUIRED_SCRIPT_TOKENS = {
    "setVmQuickFilter",
    "setVmSortMode",
    "setVmViewMode",
    "handleBulkVmAction",
    "recordActivity",
    "showToast",
    "closeTopModal",
    "toggleSidebar",
    "DiskBootRequiresDisk",
    "add-disk",
}

REQUIRED_STYLE_TOKENS = {
    ".fleet-summary-grid",
    ".segmented-control",
    ".machine-grid.compact-view",
    ".activity-table",
    ".toast-stack",
    ".app-shell.sidebar-collapsed",
    ".vm-readiness-warning",
}

FORBIDDEN_DEAD_CONTROL_IDS = {
    "connect-button",
    "close-host-button",
    "fake-filter-button",
    "fake-view-button",
}


class IdCollector(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.ids: list[str] = []

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        del tag
        for key, value in attrs:
            if key == "id" and value:
                self.ids.append(value)


def main() -> int:
    html = HTML_PATH.read_text(encoding="utf-8")
    script = SCRIPT_PATH.read_text(encoding="utf-8")
    style = STYLE_PATH.read_text(encoding="utf-8")

    parser = IdCollector()
    parser.feed(html)
    ids = parser.ids
    unique_ids = set(ids)
    if len(ids) != len(unique_ids):
        raise AssertionError("desktop HTML duplicate id detected")

    missing_view = REQUIRED_VIEW_IDS - unique_ids
    if missing_view:
        raise AssertionError(f"desktop UX view ids missing: {sorted(missing_view)}")

    selectors = set(re.findall(r'document\.querySelector\("#([A-Za-z0-9_-]+)"\)', script))
    missing_selector_targets = selectors - unique_ids
    if missing_selector_targets:
        raise AssertionError(f"desktop selector targets missing: {sorted(missing_selector_targets)}")

    missing_script = {token for token in REQUIRED_SCRIPT_TOKENS if token not in script}
    if missing_script:
        raise AssertionError(f"desktop UX script tokens missing: {sorted(missing_script)}")

    missing_style = {token for token in REQUIRED_STYLE_TOKENS if token not in style}
    if missing_style:
        raise AssertionError(f"desktop UX style tokens missing: {sorted(missing_style)}")

    dead_controls = FORBIDDEN_DEAD_CONTROL_IDS & unique_ids
    if dead_controls:
        raise AssertionError(f"dead visual controls returned: {sorted(dead_controls)}")

    print(f"DESKTOP_UX_CONTRACT_GATE=PASS ids={len(unique_ids)} selectors={len(selectors)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
