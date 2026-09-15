# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0400_shared_download_transport_contract.py
# 📌 Amac: Linux ve Android guest download yollarinin ortak HTTP transport config ve Tool kullandigini dogrular
# 📌 Modul - Python
# Version: 0.40.0
# Aciklama: Android'a ozel curl config geri donusunu, daginik retry/timeout wiring'ini ve shared transport bypass'ini fail-closed engeller
# Bagimli Oldugu Katman: Service | Tool | Config

from pathlib import Path
import yaml

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> None:
    config = yaml.safe_load(read("config/turkuazvm.yml"))
    require(config["schema_version"] == 22, "MAIN_CONFIG_SCHEMA_NOT_V22")
    http = config["downloads"]["http"]
    for key in ("curl_binary", "connect_timeout_seconds", "retry_count", "retry_delay_seconds"):
        require(key in http, f"DOWNLOAD_HTTP_SETTING_MISSING:{key}")
    distribution = config["android"]["image"]["distribution"]
    require("curl_binary" not in distribution, "ANDROID_LOCAL_CURL_CONFIG_RETURNED")
    require("branch" not in distribution and "target" not in distribution, "ANDROID_LEGACY_SOURCE_POLICY_RETURNED")

    engine_config = read("apps/engine/src/config/engine_config.rs")
    engine_service = read("apps/engine/src/services/engine_application_service.rs")
    android_service = read("apps/engine/src/services/android_image_application_service.rs")
    android_provider = read("crates/guest/src/tools/android_ci_source_provider_tool.rs")
    android_distribution = read("crates/guest/src/tools/android_ci_distribution_tool.rs")
    linux_download = read("apps/engine/src/services/installer_media_download_application_service.rs")
    http_tool = read("crates/guest/src/tools/http_download_tool.rs")

    for token in ("DownloadHttpEngineConfig", "downloads.http.curl_binary", "download_http"):
        require(token in engine_config, f"ENGINE_SHARED_DOWNLOAD_CONFIG_MISSING:{token}")
    require("distribution_curl_binary" not in engine_config, "ANDROID_ENGINE_CURL_FIELD_RETURNED")
    for token in (
        "config.download_http.curl_binary.clone()",
        "config.download_http.connect_timeout_seconds",
        "config.download_http.retry_count",
        "config.download_http.retry_delay_seconds",
    ):
        require(token in engine_service, f"ENGINE_SHARED_DOWNLOAD_WIRING_MISSING:{token}")
    require("DownloadHttpEngineConfig" in android_service, "ANDROID_SERVICE_SHARED_DOWNLOAD_CONFIG_MISSING")
    require("artifact_cache.downloader.connect_timeout_seconds" not in android_service, "ANDROID_SERVICE_ARTIFACT_POLICY_LEAK_RETURNED")
    require("artifact_cache.downloader.retry_count" not in android_service, "ANDROID_SERVICE_ARTIFACT_RETRY_LEAK_RETURNED")

    for source in (android_provider, android_distribution, linux_download):
        require("HttpDownloadTool" in source, "GUEST_DOWNLOAD_SHARED_HTTP_TOOL_MISSING")
    require("Command::new(&self.settings.curl_binary)" not in android_provider, "ANDROID_PROVIDER_DIRECT_CURL_RETURNED")
    require("Command::new(&self.settings.curl_binary)" not in android_distribution, "ANDROID_DISTRIBUTION_DIRECT_CURL_RETURNED")
    require("Command::new(curl_binary)" not in linux_download, "LINUX_DOWNLOAD_DIRECT_CURL_RETURNED")
    for token in ("--retry-all-errors", "--continue-at", "--range", "--max-filesize"):
        require(token in http_tool, f"SHARED_HTTP_CAPABILITY_MISSING:{token}")

    print("V0400_SHARED_DOWNLOAD_TRANSPORT_CONTRACT=PASS")


if __name__ == "__main__":
    main()
