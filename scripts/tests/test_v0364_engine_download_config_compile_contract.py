# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0364_engine_download_config_compile_contract.py
# 📌 Amac: Engine config download path merkezilestirmesi sonrasi Rust dead_code regresyonlarini fail-closed dogrular
# 📌 Modul - Python
# Version: 0.36.4
# Aciklama: Eski cift path alanlarinin raw config structlarina geri donmesini ve Android CI base_url provenance kaybinı engeller
# Bagimli Oldugu Katman: Service | Repo | Tool | View | Config

from pathlib import Path
import yaml

ROOT = Path(__file__).resolve().parents[2]


def read(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


def main() -> int:
    engine = read("apps/engine/src/config/engine_config.rs")
    tool = read("crates/guest/src/tools/android_ci_distribution_tool.rs")
    main_config = read("config/turkuazvm.yml")
    verify = read("scripts/verify_structure.ps1")
    sources = yaml.safe_load(read("config/download-sources.yml"))

    forbidden_engine_fields = (
        "installer_media_download_root: String",
        "struct ArtifactCacheConfig {\n    enabled: bool,\n    root: String",
        "struct AndroidImageConfig {\n    source_root: String,\n    output_root: String",
    )
    leaked = [token for token in forbidden_engine_fields if token in engine]
    if leaked:
        raise AssertionError(f"deprecated raw download config fields returned: {leaked}")

    required_engine_wiring = (
        "download_sources.paths.installer_media",
        "download_sources.paths.android_images",
        "download_sources.paths.artifact_cache",
    )
    missing = [token for token in required_engine_wiring if token not in engine]
    if missing:
        raise AssertionError(f"central download path engine wiring missing: {missing}")

    legacy_main_tokens = (
        "installer_media_download_root:",
        "root: ./data/cache/artifacts",
        "output_root: ./data/android-image-builds",
    )
    leaked_main = [token for token in legacy_main_tokens if token in main_config]
    if leaked_main:
        raise AssertionError(f"duplicated download paths remain in turkuazvm.yml: {leaked_main}")

    paths = sources.get("paths") or {}
    for key in ("installer_media", "android_images", "artifact_cache"):
        if not str(paths.get(key, "")).strip():
            raise AssertionError(f"download-sources path missing: {key}")

    if 'base_url: \\"{base_url}\\"' not in tool:
        raise AssertionError("Android CI distribution provenance does not consume/persist base_url")

    if "installer_media_download_root: ./data/installer-media" in verify:
        raise AssertionError("launcher still requires legacy installer_media_download_root in main config")
    for token in ("installer_media:", "android_images:", "artifact_cache:"):
        if token not in verify:
            raise AssertionError(f"launcher centralized download path gate missing: {token}")

    for relative in (
        "config/turkuazvm.remote-engine.example.yml",
        "config/turkuazvm.desktop-remote.example.yml",
    ):
        text = read(relative)
        leaked_example = [token for token in legacy_main_tokens if token in text]
        if leaked_example:
            raise AssertionError(f"legacy duplicated paths remain in {relative}: {leaked_example}")

    print("V0364_ENGINE_DOWNLOAD_CONFIG_COMPILE_CONTRACT=PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
