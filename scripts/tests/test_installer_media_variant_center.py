# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_installer_media_variant_center.py
# 📌 Amac: Host uyumlu ISO onerisi ile coklu installer medya varyanti kontratini denetler
# 📌 Modul - Python
# Version: 0.39.4
# Aciklama: Debian direct indirme, alternatif surumler, mimari filtreleme ve media_id API akislarini fail-closed dogrular
# Bagimli Oldugu Katman: Service | Repo | Tool | View

from pathlib import Path
import sys
import yaml


def require(errors, condition, message):
    if not condition:
        errors.append(message)


def main():
    root = Path(__file__).resolve().parents[2]
    errors = []
    catalog = yaml.safe_load((root / "config/guest-catalog.yml").read_text(encoding="utf-8"))
    require(errors, catalog.get("schema_version") == 4, "guest catalog schema 4 required")
    templates = catalog.get("templates") or []

    for template in templates:
        sources = [template.get("installer_media"), *(template.get("installer_media_options") or [])]
        sources = [source for source in sources if source]
        ids = [source.get("id") for source in sources]
        require(errors, len(ids) == len(set(ids)), f"duplicate media id in {template.get('id')}")
        for source in sources:
            require(errors, bool(source.get("id")), f"media id missing in {template.get('id')}")
            require(errors, bool(source.get("architecture")), f"media architecture missing in {template.get('id')}")

    debian = next((item for item in templates if item.get("id") == "debian-13-server"), None)
    require(errors, debian is not None, "Debian 13 template missing")
    if debian:
        primary = debian.get("installer_media") or {}
        options = debian.get("installer_media_options") or []
        require(errors, primary.get("mode") == "direct", "Debian primary media must download directly")
        require(errors, primary.get("id") == "debian-13.6-netinst-amd64", "Debian stable recommended media id invalid")
        require(errors, primary.get("recommended") is True, "Debian stable media must be recommended")
        require(errors, "cdimage.debian.org/debian-cd/current/amd64/iso-cd/" in str(primary.get("url")), "Debian stable official current URL missing")
        require(errors, str(primary.get("checksum_url", "")).endswith("/SHA256SUMS"), "Debian stable SHA256SUMS missing")
        option_ids = {item.get("id") for item in options}
        require(errors, "debian-13.6-dvd1-amd64" in option_ids, "Debian DVD-1 option missing")
        require(errors, "debian-12.15-netinst-amd64" in option_ids, "Debian 12.15 oldstable option missing")

    domain = (root / "crates/guest-catalog/src/domain/guest_template.rs").read_text(encoding="utf-8")
    repository = (root / "crates/repositories/src/repositories/yaml_guest_catalog_repository.rs").read_text(encoding="utf-8")
    api = (root / "crates/engine-api/src/lib.rs").read_text(encoding="utf-8")
    downloader = (root / "apps/engine/src/services/installer_media_download_application_service.rs").read_text(encoding="utf-8")
    html = (root / "apps/desktop/ui/index.html").read_text(encoding="utf-8")
    js = (root / "apps/desktop/ui/app.js").read_text(encoding="utf-8")

    for token in ("installer_media_options", "installer_media_source", "pub architecture: String", "pub recommended: bool"):
        require(errors, token in domain, f"guest catalog media variant domain missing: {token}")
    for token in ("CATALOG_SCHEMA_VERSION: u16 = 4", "installer_media_options", "media.id", "media.architecture"):
        require(errors, token in repository, f"guest catalog repository variant mapping missing: {token}")
    for token in ("ENGINE_API_VERSION: u16 = 24", "media_id: String", "installer_media_options: Vec<InstallerMediaSourceDto>"):
        require(errors, token in api, f"Engine API media variant contract missing: {token}")
    for token in ("download_key", ".join(media_id)", "installer_media_source(media_id)"):
        require(errors, token in downloader, f"download isolation missing: {token}")
    for token in ("installer-media-options-toggle", "installer-media-source-select", "installer-media-host-architecture", "installer-media-recommended-label"):
        require(errors, token in html, f"installer media variant view missing: {token}")
    for token in ("normalizeInstallerArchitecture", "installerMediaSourceCompatible", "hostArchitecture", "Farkli Surum / Medya Sec", "media_id: mediaId"):
        require(errors, token in js, f"installer media variant behavior missing: {token}")

    if errors:
        for error in errors:
            print(f"FAIL: {error}")
        return 1
    print("INSTALLER_MEDIA_VARIANT_CENTER_OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
