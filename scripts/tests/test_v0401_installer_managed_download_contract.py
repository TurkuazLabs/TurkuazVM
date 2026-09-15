# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0401_installer_managed_download_contract.py
# 📌 Amac: Official-page resolver medya kaynaklarinin UI tarafinda yonetilen indirme olarak kullanilmasini ve guvenli mirror failover davranisini dogrular
# 📌 Modul - Python
# Version: 0.40.1
# Aciklama: Linux Mint resolver yolunun Desktop tarafinda erisilebilir olmasini, Windows official-page davranisinin korunmasini ve mirror gecisinde partial dosya temizligini kilitler
# Bagimli Oldugu Katman: Service | Tool | View | Config

from pathlib import Path
import yaml

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> None:
    catalog = yaml.safe_load(read("config/guest-catalog.yml"))
    templates = {item["id"]: item for item in catalog["templates"]}
    mint = templates["linux-mint-22.3-cinnamon"]["installer_media"]
    require(mint["mode"] == "official_page", "MINT_OFFICIAL_PAGE_MODE_CHANGED")
    require(mint["media_kind"] == "desktop_live", "MINT_TYPED_MEDIA_KIND_MISSING")

    for template_id in (
        "windows-11-25h2-pro",
        "windows-11-25h2-enterprise",
        "windows-11-24h2-pro",
        "windows-server-2025-standard",
        "windows-server-2022-standard",
    ):
        source = templates[template_id]["installer_media"]
        require(source["mode"] == "official_page", f"WINDOWS_OFFICIAL_PAGE_MODE_CHANGED:{template_id}")
        require("media_kind" not in source, f"WINDOWS_MANAGED_MEDIA_KIND_UNEXPECTED:{template_id}")

    domain = read("crates/guest-catalog/src/domain/guest_template.rs")
    api = read("crates/engine-api/src/lib.rs")
    engine = read("apps/engine/src/services/engine_application_service.rs")
    desktop = read("apps/desktop/ui/app.js")
    download = read("apps/engine/src/services/installer_media_download_application_service.rs")

    require("managed_download_supported" in domain, "MANAGED_DOWNLOAD_DOMAIN_CAPABILITY_MISSING")
    require("InstallerMediaMode::Direct => true" in domain, "DIRECT_MANAGED_DOWNLOAD_CAPABILITY_MISSING")
    require("InstallerMediaKind::Unknown | InstallerMediaKind::OfficialPage" in domain, "OFFICIAL_PAGE_FAIL_CLOSED_GUARD_MISSING")
    require("#[serde(default)]\n    pub managed_download: bool" in api, "MANAGED_DOWNLOAD_API_FIELD_NOT_BACKWARD_COMPATIBLE")
    require("managed_download: media.managed_download_supported()" in engine, "ENGINE_MANAGED_DOWNLOAD_MAPPING_MISSING")

    require("function installerMediaSourceManagedDownload(source)" in desktop, "DESKTOP_MANAGED_DOWNLOAD_HELPER_MISSING")
    require('const INSTALLER_MEDIA_MODE_OFFICIAL_PAGE = "official_page";' in desktop, "INSTALLER_MEDIA_MODE_CONSTANT_MISSING")
    require('source.mode === "direct"' not in desktop, "DESKTOP_DIRECT_MODE_GATE_RETURNED")
    require('source.mode !== "direct"' not in desktop, "DESKTOP_DIRECT_MODE_NEGATIVE_GATE_RETURNED")
    require("const managedSources = installerMediaSourcesForTemplate(template)" in desktop, "CREATE_WIZARD_MANAGED_SOURCE_FILTER_MISSING")
    require("installerMediaSourceManagedDownload(selected.source)" in desktop, "IMAGE_CENTER_MANAGED_DOWNLOAD_MISSING")

    require("if index > 0 && part_path.exists()" in download, "MIRROR_FAILOVER_PARTIAL_ISOLATION_MISSING")
    require("Mirror failover oncesi partial ISO temizlenemedi" in download, "MIRROR_FAILOVER_ERROR_CONTEXT_MISSING")

    print("V0401_INSTALLER_MANAGED_DOWNLOAD_CONTRACT=PASS")


if __name__ == "__main__":
    main()
