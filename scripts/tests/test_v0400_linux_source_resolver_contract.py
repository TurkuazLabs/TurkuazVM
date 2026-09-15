# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0400_linux_source_resolver_contract.py
# 📌 Amac: Linux installer media runtime discovery, mirror failover, checksum failover ve cache mimarisini dogrular
# 📌 Modul - Python
# Version: 0.40.0
# Aciklama: Catalog exact URL yerine Service -> Provider Tool -> Cache Repo -> HttpDownloadTool zincirinin aktif oldugunu fail-closed kilitler
# Bagimli Oldugu Katman: Service | Repo | Tool | Config

from pathlib import Path
import yaml

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> None:
    config = yaml.safe_load(read("config/download-sources.yml"))
    require(config["schema_version"] == 5, "DOWNLOAD_SOURCE_SCHEMA_NOT_V5")
    require(config["paths"]["installer_media_source_cache"].endswith("installer-media-sources.yml"), "LINUX_SOURCE_CACHE_PATH_MISSING")

    linux = config["sources"]["linux_media"]
    require(linux["use_catalog_fallback"] is True, "LINUX_CATALOG_LAST_RESORT_DISABLED")
    providers = linux["providers"]
    for provider in ("ubuntu", "debian", "fedora", "rocky-linux", "linux-mint"):
        require(provider in providers, f"LINUX_PROVIDER_MISSING:{provider}")
    require(providers["linux-mint"]["discovery_mode"] == "official_page_mirrors", "MINT_PAGE_DISCOVERY_MISSING")
    require(providers["fedora"]["discovery_mode"] == "directory_index", "FEDORA_DIRECTORY_DISCOVERY_MISSING")
    require("debian-{release}." in providers["debian"]["rules"]["network_install"]["filename_tokens"], "DEBIAN_RELEASE_GUARD_MISSING")
    require("Fedora-Server-netinst-x86_64-{release}-" in providers["fedora"]["rules"]["network_install"]["filename_tokens"], "FEDORA_NETINSTALL_RULE_MISSING")
    require("Rocky-{release}-latest-x86_64-boot.iso" in providers["rocky-linux"]["rules"]["boot"]["filename_tokens"], "ROCKY_BOOT_RULE_MISSING")
    require("linuxmint-{release}-cinnamon-64bit.iso" in providers["linux-mint"]["rules"]["desktop_live"]["filename_tokens"], "MINT_CINNAMON_RULE_MISSING")

    domain = read("crates/guest-catalog/src/domain/installer_media_source.rs")
    resolver = read("crates/guest-catalog/src/services/installer_media_source_resolver_service.rs")
    provider = read("crates/guest/src/tools/linux_installer_media_provider_tool.rs")
    http = read("crates/guest/src/tools/http_download_tool.rs")
    cache = read("crates/repositories/src/repositories/yaml_installer_media_source_cache_repository.rs")
    service = read("apps/engine/src/services/installer_media_download_application_service.rs")
    engine = read("apps/engine/src/services/engine_application_service.rs")
    desktop_settings = read("apps/desktop/src-tauri/src/tools/download_settings_tool.rs")

    for token in ("InstallerMediaDiscoveryMode", "InstallerMediaSourceRequest", "ResolvedInstallerMediaSource", "download_urls", "checksum_urls"):
        require(token in domain, f"LINUX_SOURCE_DOMAIN_MISSING:{token}")
    for token in ("self.provider.resolve(request, provider_policy)", "self.cache.load", "LastKnownGoodCache", "catalog_fallback", "append_catalog_fallback"):
        require(token in resolver, f"LINUX_RESOLVER_SERVICE_MISSING:{token}")
    require("catalog_filename != source.filename" in resolver, "LINUX_CATALOG_FALLBACK_FILENAME_GUARD_MISSING")
    for token in ("OfficialPageMirrors", "resolve_official_page_mirrors", "stable_fedora_filename", "extract_href_values", "checksum_filename", "mirror_urls_for_filename"):
        require(token in provider, f"LINUX_PROVIDER_DISCOVERY_MISSING:{token}")
    for token in ("fetch_text", "spawn_file_download", "--continue-at", "--retry-all-errors"):
        require(token in http, f"SHARED_HTTP_TOOL_MISSING:{token}")
    for token in ("YamlInstallerMediaSourceCacheRepository", "CACHE_SCHEMA_VERSION", "atomic_write", "resolved_at_unix"):
        require(token in cache, f"LINUX_SOURCE_CACHE_REPO_MISSING:{token}")

    for token in ("InstallerMediaSourceResolverService", "LinuxInstallerMediaProviderTool", "YamlInstallerMediaSourceCacheRepository", "HttpDownloadTool", "fetch_expected_sha256", "Tum resmi ISO mirrorlari basarisiz", "Tum resmi SHA-256 kaynaklari basarisiz"):
        require(token in service, f"INSTALLER_DOWNLOAD_RESOLVER_WIRING_MISSING:{token}")
    require("Command::new(curl_binary)" not in service, "LEGACY_INSTALLER_DIRECT_CURL_RETURNED")
    require("provider: template.product_id" in service, "INSTALLER_PROVIDER_ID_NOT_PRODUCT_ID")
    require("config.guest.installer_media_source_cache_path.clone()" in engine, "ENGINE_INSTALLER_CACHE_WIRING_MISSING")
    require("config.guest.installer_media_resolver_policy.clone()" in engine, "ENGINE_INSTALLER_POLICY_WIRING_MISSING")

    require("SETTINGS_SCHEMA_VERSION: u16 = 5" in desktop_settings, "DESKTOP_DOWNLOAD_SETTINGS_SCHEMA_NOT_V5")
    for token in ("installer_media_source_cache", "linux_media", "discovery_mode", "filename_tokens", "checksum_strategy"):
        require(token in desktop_settings, f"DESKTOP_DOWNLOAD_SETTINGS_LOSSY_FIELD:{token}")

    print("V0400_LINUX_SOURCE_RESOLVER_CONTRACT=PASS")


if __name__ == "__main__":
    main()
