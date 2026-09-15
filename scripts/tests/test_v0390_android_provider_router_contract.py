# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0390_android_provider_router_contract.py
# 📌 Amac: Android release-provider router ile v0.40 Source Resolver artifact preflight davranisini regression olarak kilitler
# 📌 Modul - Python
# Version: 0.40.0
# Aciklama: Android 10 source-build politikasini ve Android 11-17 icin same-build device/host probe isleminin Provider Tool'da kalmasini dogrular
# Bagimli Oldugu Katman: Service | Tool | Config

from pathlib import Path
import yaml

ROOT = Path(__file__).resolve().parents[2]
CONFIG = yaml.safe_load((ROOT / "config/download-sources.yml").read_text(encoding="utf-8"))
ROUTER = (ROOT / "crates/guest/src/tools/android_distribution_router_tool.rs").read_text(encoding="utf-8")
PROVIDER = (ROOT / "crates/guest/src/tools/android_ci_source_provider_tool.rs").read_text(encoding="utf-8")
ENGINE = (ROOT / "apps/engine/src/services/android_image_application_service.rs").read_text(encoding="utf-8")


def main() -> None:
    assert CONFIG["schema_version"] == 5, "ANDROID_PROVIDER_CONFIG_SCHEMA_NOT_V5"
    policy = CONFIG["sources"]["android_release_policy"]
    for release in ("17", "16", "15", "14", "13", "12L", "12", "11"):
        assert policy[release] == "android_ci", f"ANDROID_{release}_LIVE_PROVIDER_CHANGED"
    assert policy["10"] == "source_build", "ANDROID10_BROKEN_LIVE_CI_PROVIDER_RETURNED"
    for token in ("pub enum AndroidDistributionProvider", "pub struct AndroidDistributionRouterTool", "AndroidDistributionProvider::AndroidCi", "AndroidDistributionProvider::SourceBuild", "Goruntu Merkezi > Build Plan"):
        assert token in ROUTER, f"ANDROID_PROVIDER_ROUTER_TOKEN_MISSING:{token}"
    for token in ("AndroidDistributionRouterTool", "AndroidDistributionSourceResolverService::new", "AndroidCiSourceProviderTool::new"):
        assert token in ENGINE, f"ANDROID_PROVIDER_ENGINE_WIRING_MISSING:{token}"
    for token in ("same-build Cuttlefish host package artifact bulunamadi", "same-build Cuttlefish x86_64 device image artifact bulunamadi", "artifact_exists", "host_package_available"):
        assert token in PROVIDER, f"ANDROID_ARTIFACT_PREFLIGHT_MISSING:{token}"
    print("V0390_ANDROID_PROVIDER_ROUTER_CONTRACT=PASS")


if __name__ == "__main__":
    main()
