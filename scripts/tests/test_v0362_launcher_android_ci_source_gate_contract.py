# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0362_launcher_android_ci_source_gate_contract.py
# 📌 Amac: Launcher ve Engine Android CI dogrulamasinin static exact URL yerine v0.40 Source Resolver mimarisini kabul ettigini dogrular
# 📌 Modul - Python
# Version: 0.40.0
# Aciklama: Eski Distribution Tool-owned discovery beklentisinin launcher'i bloke etmesini onler ve provider/cache resolver wiringini kilitler
# Bagimli Oldugu Katman: Service | Repo | Tool | Config

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
VERIFY = (ROOT / "scripts/verify_structure.ps1").read_text(encoding="utf-8")
PROVIDER = (ROOT / "crates/guest/src/tools/android_ci_source_provider_tool.rs").read_text(encoding="utf-8")
RESOLVER = (ROOT / "crates/android-image/src/services/android_distribution_source_resolver_service.rs").read_text(encoding="utf-8")
ENGINE_CONFIG = (ROOT / "apps/engine/src/config/engine_config.rs").read_text(encoding="utf-8")
SOURCES = (ROOT / "config/download-sources.yml").read_text(encoding="utf-8")


def main() -> int:
    if '"ci.android.com/builds/branches"' in VERIFY:
        raise AssertionError("launcher still requires obsolete hardcoded Android CI branch URL")
    for token in ("ANDROID_DOWNLOAD_SOURCE_CONFIG_MISSING", "ANDROID_DOWNLOAD_SOURCE_ENGINE_WIRING_MISSING"):
        if token not in VERIFY:
            raise AssertionError(f"launcher Android source verifier missing: {token}")
    for token in ("status.json", 'format!("{base_url}/builds/branches/{branch}/{BRANCH_STATUS_FILE}")', "AndroidCiSourceProviderTool"):
        if token not in PROVIDER:
            raise AssertionError(f"Android CI provider discovery contract missing: {token}")
    for token in ("AndroidDistributionSourceResolverService", "self.provider.resolve(&request)", "self.cache.load"):
        if token not in RESOLVER:
            raise AssertionError(f"Android resolver contract missing: {token}")
    for token in ("download_sources.sources.android_ci.base_url", "parse_android_distribution_channels", "distribution_official_fallback", "source_cache_path"):
        if token not in ENGINE_CONFIG:
            raise AssertionError(f"Engine download-source wiring missing: {token}")
    for token in ("schema_version: 5", "android_source_cache:", "android_release_policy:", "android_ci:", "base_url: https://ci.android.com", '"17":', '"10":'):
        if token not in SOURCES:
            raise AssertionError(f"Android download source config missing: {token}")
    print("V0362_LAUNCHER_ANDROID_CI_SOURCE_GATE_CONTRACT=PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
