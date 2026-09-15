# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0394_linux_media_policy_contract.py
# 📌 Amac: Linux Desktop ve Server installer medya secim politikasinin dagitimlar arasinda dogru varsayilan medyayi sectigini denetler
# 📌 Modul - Python
# Version: 0.39.4
# Aciklama: Fedora netinstall, Debian netinst, Rocky boot, Ubuntu live-server ve Desktop live medya kurallarini fail-closed dogrular
# Bagimli Oldugu Katman: Service | Repo | Tool | View

from pathlib import Path
import sys
import yaml

SERVER_LIGHT_KINDS = {"network_install", "boot", "minimal", "server_standard"}
DESKTOP_KINDS = {"desktop_live"}
KNOWN_KINDS = SERVER_LIGHT_KINDS | DESKTOP_KINDS | {"dvd", "official_page", "unknown"}


def require(errors, condition, message):
    if not condition:
        errors.append(message)


def main():
    root = Path(__file__).resolve().parents[2]
    errors = []
    catalog = yaml.safe_load((root / "config/guest-catalog.yml").read_text(encoding="utf-8"))
    require(errors, catalog.get("schema_version") == 4, "guest catalog schema 4 required")
    templates = catalog.get("templates") or []
    policies = catalog.get("linux_media_policies") or []
    policy_map = {(item.get("product_id"), item.get("profile_id")): item for item in policies}

    linux_templates = [item for item in templates if item.get("family") == "linux"]
    for template in linux_templates:
        key = (template.get("product_id"), template.get("profile_id"))
        policy = policy_map.get(key)
        require(errors, policy is not None, f"Linux media policy missing: {template.get('id')}")
        primary = template.get("installer_media") or {}
        options = template.get("installer_media_options") or []
        sources = [primary, *options]
        kinds = [source.get("media_kind") for source in sources if source]
        require(errors, all(kind in KNOWN_KINDS for kind in kinds), f"unknown media kind: {template.get('id')}")
        require(errors, primary.get("recommended") is True, f"primary media must be recommended: {template.get('id')}")
        require(errors, all(source.get("recommended") is not True for source in options), f"only primary media may be recommended: {template.get('id')}")
        if policy:
            preferred = policy.get("preferred_media_kinds") or []
            require(errors, bool(preferred), f"preferred media kinds missing: {template.get('id')}")
            require(errors, primary.get("media_kind") == preferred[0], f"primary media does not match policy: {template.get('id')}")

    expected = {
        "fedora-44-server": ("fedora-server-44-netinst-x86_64", "network_install"),
        "debian-13-server": ("debian-13.6-netinst-amd64", "network_install"),
        "rocky-10-server": ("rocky-10-latest-boot-x86_64", "boot"),
        "ubuntu-26.04-lts-server": ("ubuntu-26.04.1-server-amd64", "server_standard"),
        "fedora-44-workstation": ("fedora-workstation-44-1.7-x86_64", "desktop_live"),
        "debian-13-desktop-gnome": ("debian-live-13.6-gnome-amd64", "desktop_live"),
    }
    by_id = {item.get("id"): item for item in templates}
    for template_id, (media_id, media_kind) in expected.items():
        template = by_id.get(template_id)
        require(errors, template is not None, f"template missing: {template_id}")
        if template:
            primary = template.get("installer_media") or {}
            require(errors, primary.get("id") == media_id, f"wrong primary media: {template_id}")
            require(errors, primary.get("media_kind") == media_kind, f"wrong primary media kind: {template_id}")

    for template in linux_templates:
        primary = template.get("installer_media") or {}
        profile = str(template.get("profile_id") or "")
        if profile == "server":
            options = template.get("installer_media_options") or []
            has_light = primary.get("media_kind") in SERVER_LIGHT_KINDS or any(
                source.get("media_kind") in SERVER_LIGHT_KINDS for source in options
            )
            if has_light:
                require(errors, primary.get("media_kind") != "dvd", f"server DVD selected despite lighter official media: {template.get('id')}")

    service = (root / "crates/guest-catalog/src/services/guest_catalog_service.rs").read_text(encoding="utf-8")
    repository = (root / "crates/repositories/src/repositories/yaml_guest_catalog_repository.rs").read_text(encoding="utf-8")
    domain = (root / "crates/guest-catalog/src/domain/guest_template.rs").read_text(encoding="utf-8")
    for token in ("apply_linux_media_policy", "policy.rank", "selected.recommended = true", "GuestFamily::Linux"):
        require(errors, token in service, f"Linux media policy service token missing: {token}")
    for token in ("CATALOG_SCHEMA_VERSION: u16 = 4", "linux_media_policies", "InstallerMediaKindManifest", "to_linux_media_policy"):
        require(errors, token in repository, f"Linux media policy repository token missing: {token}")
    for token in ("InstallerMediaKind", "NetworkInstall", "ServerStandard", "DesktopLive"):
        require(errors, token in domain, f"installer media kind domain token missing: {token}")

    if errors:
        for error in errors:
            print(f"FAIL: {error}")
        return 1
    print("V0394_LINUX_MEDIA_POLICY_OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
