# 📄 Dosya Yolu: /turkuazvm/scripts/test_guest_catalog_wizard.py
# 📌 Amac: Guest Catalog ve VM Creation Wizard v0.26.0 kontratlarini statik olarak dogrular
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: YAML katalog, Engine API, template persistence ve Desktop sirali wizard baglantilarini fail-closed kontrol eder
# Bagimli Oldugu Katman: Service | Repo | Tool | View

from __future__ import annotations

from pathlib import Path
import sys

import yaml


def require(condition: bool, message: str, errors: list[str]) -> None:
    if not condition:
        errors.append(message)


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    errors: list[str] = []

    catalog_path = root / "config/guest-catalog.yml"
    catalog = yaml.safe_load(catalog_path.read_text(encoding="utf-8"))
    require(catalog.get("schema_version") == 3, "guest catalog schema must be 3", errors)
    templates = catalog.get("templates") or []
    require(len(templates) >= 10, "guest catalog must provide a useful initial template set", errors)
    ids = [entry.get("id") for entry in templates]
    require(len(ids) == len(set(ids)), "guest catalog template ids must be unique", errors)
    families = {entry.get("family") for entry in templates}
    require({"windows", "linux", "android", "other"}.issubset(families), "all guest families must exist", errors)
    for entry in templates:
        recommended = entry.get("recommended") or {}
        require(int(recommended.get("vcpu_count", 0)) > 0, f"{entry.get('id')}: vcpu invalid", errors)
        require(int(recommended.get("memory_mib", 0)) >= 512, f"{entry.get('id')}: memory invalid", errors)
        require(int(recommended.get("disk_size_gib", 0)) > 0, f"{entry.get('id')}: disk invalid", errors)

    api = (root / "crates/engine-api/src/lib.rs").read_text(encoding="utf-8")
    for token in ("ENGINE_API_VERSION: u16 = 24", "ListGuestCatalog", "GuestCatalogList", "GuestTemplateDto", "guest_template_id"):
        require(token in api, f"Engine API missing {token}", errors)

    guest_boot = (root / "crates/core/src/domain/guest_boot.rs").read_text(encoding="utf-8")
    for token in ("catalog_template_id", "InvalidCatalogTemplateId"):
        require(token in guest_boot, f"guest boot persistence missing {token}", errors)

    repository = (root / "crates/repositories/src/repositories/yaml_guest_catalog_repository.rs").read_text(encoding="utf-8")
    require("impl GuestCatalogRepositoryPort" in repository, "guest catalog repository port adapter missing", errors)
    engine_app = (root / "apps/engine/src/services/engine_application_service.rs").read_text(encoding="utf-8")
    for token in ("guest_profile_from_hint", "guest_template_get_failed"):
        require(token in engine_app, f"server-side guest template validation missing {token}", errors)
    guest_catalog_application = (root / "apps/engine/src/services/guest_catalog_application_service.rs").read_text(encoding="utf-8")
    require("pub fn get(&self, id: &str)" in guest_catalog_application, "guest template server lookup missing", errors)

    html = (root / "apps/desktop/ui/index.html").read_text(encoding="utf-8")
    js = (root / "apps/desktop/ui/app.js").read_text(encoding="utf-8")
    for token in ("guest-family-select", "guest-product-select", "guest-release-select", "guest-profile-select", "create-add-disk", "installer-media-modal"):
        require(token in html, f"wizard view missing {token}", errors)
    for token in ("list_guest_catalog", "renderGuestCatalogSelectors", "guest_template_id", "handlePostCreateDisk", "openInstallerMediaModal", "openNetworkModal"):
        require(token in js, f"wizard script missing {token}", errors)
    for stale in ("create-assign-image", "create-add-network", "handlePostCreateImage", "handlePostCreateNetwork"):
        require(stale not in html and stale not in js, f"stale post-create bypass present {stale}", errors)

    if errors:
        for error in errors:
            print(f"FAIL: {error}")
        return 1
    print(f"GUEST_CATALOG_WIZARD_GATE=PASS templates={len(templates)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
