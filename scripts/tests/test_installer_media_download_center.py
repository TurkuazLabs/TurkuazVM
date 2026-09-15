# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_installer_media_download_center.py
# 📌 Amac: Resmi ISO indirme merkezi, SHA-256 dogrulama ve local ISO fallback kontratini denetler
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Guest Catalog medya metadata, Engine API v24, background downloader ve Desktop UI baglantilarini regression olarak kontrol eder
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

    config = yaml.safe_load((root / "config/turkuazvm.yml").read_text(encoding="utf-8"))
    require(errors, config.get("schema_version") == 22, "config schema 22 required")
    sources_path = config.get("downloads", {}).get("sources_path")
    require(errors, sources_path == "config/download-sources.yml", "download sources config path missing")
    download_sources = yaml.safe_load((root / sources_path).read_text(encoding="utf-8")) if sources_path else {}
    require(errors, download_sources.get("paths", {}).get("installer_media"), "installer media download root missing")

    catalog = yaml.safe_load((root / "config/guest-catalog.yml").read_text(encoding="utf-8"))
    require(errors, catalog.get("schema_version") == 4, "guest catalog schema 4 required")
    templates = catalog.get("templates", [])
    media_templates = [item for item in templates if item.get("installer_media")]
    require(errors, len(media_templates) >= 8, "installer media catalog coverage is too small")
    direct = [item for item in media_templates if item["installer_media"].get("mode") == "direct"]
    official = [item for item in media_templates if item["installer_media"].get("mode") == "official_page"]
    media_options = [media for item in media_templates for media in item.get("installer_media_options", [])]
    require(errors, any(item.get("id") == "ubuntu-26.04-lts-desktop" for item in direct), "Ubuntu 26.04 direct ISO source missing")
    require(errors, any(item.get("id") == "debian-13-server" for item in direct), "Debian 13 direct ISO source missing")
    require(errors, any(media.get("id") == "debian-12.15-netinst-amd64" for media in media_options), "Debian oldstable media option missing")
    require(errors, any(item.get("id", "").startswith("windows-11") for item in official), "Windows official download page source missing")
    for item in direct:
        media = item["installer_media"]
        require(errors, str(media.get("url", "")).startswith("https://"), f"direct media URL must be HTTPS: {item.get('id')}")
        require(errors, str(media.get("filename", "")).endswith(".iso"), f"direct media filename must be ISO: {item.get('id')}")
        require(errors, str(media.get("checksum_url", "")).startswith("https://"), f"direct media checksum URL missing: {item.get('id')}")
        require(errors, media.get("id"), f"direct media id missing: {item.get('id')}")
        require(errors, media.get("architecture"), f"direct media architecture missing: {item.get('id')}")

    api = (root / "crates/engine-api/src/lib.rs").read_text(encoding="utf-8")
    for token in (
        "ENGINE_API_VERSION: u16 = 24",
        "StartInstallerMediaDownload",
        "GetInstallerMediaDownload",
        "CancelInstallerMediaDownload",
        "AttachDownloadedInstallerMedia",
        "InstallerMediaDownloadDto",
        "InstallerMediaSourceDto",
    ):
        require(errors, token in api, f"Engine API installer media contract missing: {token}")

    service = (root / "apps/engine/src/services/installer_media_download_application_service.rs").read_text(encoding="utf-8")
    for token in ("thread::spawn", "Sha256", "checksum_url", ".part", "InstallerMediaDownloadState::Ready", "InstallerMediaDownloadState::Cancelled", "child.kill()"):
        require(errors, token in service, f"installer media background downloader missing: {token}")

    html = (root / "apps/desktop/ui/index.html").read_text(encoding="utf-8")
    js = (root / "apps/desktop/ui/app.js").read_text(encoding="utf-8")
    for token in (
        'id="installer-media-download-button"',
        'id="installer-media-cancel-download-button"',
        'id="installer-media-attach-downloaded-button"',
        'id="installer-media-official-page-button"',
        'id="installer-media-download-progress"',
        'id="installer-media-pick-button"',
        'id="installer-media-options-toggle"',
        'id="installer-media-source-select"',
        'id="installer-media-host-architecture"',
    ):
        require(errors, token in html, f"installer media UI missing: {token}")
    for token in (
        'invoke("start_installer_media_download"',
        'invoke("get_installer_media_download"',
        'invoke("cancel_installer_media_download"',
        'invoke("attach_downloaded_installer_media"',
        'invoke("open_external_url"',
        "startInstallerMediaPolling",
        "pickInstallerMedia",
        "installerMediaSourceCompatible",
        "handleInstallerMediaSourceChange",
    ):
        require(errors, token in js, f"installer media UI behavior missing: {token}")

    if errors:
        for error in errors:
            print(f"FAIL: {error}")
        return 1
    print(f"INSTALLER_MEDIA_DOWNLOAD_CENTER_OK templates={len(media_templates)} direct={len(direct)} official={len(official)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
