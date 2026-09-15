# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0365_launcher_download_sources_path_gate_contract.py
# 📌 Amac: Launcher'in merkezi download-sources path mimarisini eski turkuazvm.yml output_root alanina baglamamasini fail-closed dogrular
# 📌 Modul - Python
# Version: 0.36.5
# Aciklama: Android image output path dahil tum download hedeflerinin config/download-sources.yml uzerinden dogrulanmasini ve legacy main-config beklentisinin geri donmemesini korur
# Bagimli Oldugu Katman: Tool | Config | Service

from pathlib import Path
import yaml

ROOT = Path(__file__).resolve().parents[2]


def read(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


def main() -> int:
    verify = read("scripts/verify_structure.ps1")
    main_config = read("config/turkuazvm.yml")
    sources_text = read("config/download-sources.yml")
    sources = yaml.safe_load(sources_text)

    legacy_main_path = "output_root: ./data/android-image-builds"
    if legacy_main_path in main_config:
        raise AssertionError("legacy Android output_root returned to turkuazvm.yml")

    android_gate_match = 'ANDROID_IMAGE_CONFIG_MISSING: $Token'
    if android_gate_match not in verify:
        raise AssertionError("Android main config launcher gate missing")
    if '"output_root: ./data/android-image-builds"' in verify:
        raise AssertionError("launcher still requires legacy Android output_root in main config")

    required_source_tokens = (
        "installer_media: ./data/installer-media",
        "android_images: ./data/android-image-builds",
        "artifact_cache: ./data/cache/artifacts",
    )
    missing_verify = [token for token in required_source_tokens if token not in verify]
    if missing_verify:
        raise AssertionError(f"launcher exact centralized path gate missing: {missing_verify}")

    paths = (sources or {}).get("paths") or {}
    expected = {
        "installer_media": "./data/installer-media",
        "android_images": "./data/android-image-builds",
        "artifact_cache": "./data/cache/artifacts",
    }
    wrong = {key: paths.get(key) for key, value in expected.items() if paths.get(key) != value}
    if wrong:
        raise AssertionError(f"download-sources path defaults changed unexpectedly: {wrong}")

    print("V0365_LAUNCHER_DOWNLOAD_SOURCES_PATH_GATE_CONTRACT=PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
