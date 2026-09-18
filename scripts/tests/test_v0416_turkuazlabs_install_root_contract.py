# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0416_turkuazlabs_install_root_contract.py
# 📌 Amac: v0.41.6 Windows installer kokunun TurkuazLabs/TurkuazVM hiyerarsisinde kalmasini statik olarak kilitler
# 📌 Modul - Python Test
# Version: 0.41.6
# Aciklama: Pinli Tauri 2.11.4 NSIS template kaynagini, blob SHA dogrulamasini, install-root patchlerini ve gercek smoke test kontratini kontrol eder
# Bagimli Oldugu Katman: Tool | CI/CD | View

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def require(path: str, needle: str) -> None:
    content = read(path)
    if needle not in content:
        raise AssertionError(f"{path}: missing {needle!r}")


def forbid(path: str, needle: str) -> None:
    content = read(path)
    if needle in content:
        raise AssertionError(f"{path}: forbidden {needle!r}")


def main() -> None:
    package = "scripts/package_windows_distribution.ps1"
    smoke = "scripts/test_windows_installer_smoke.ps1"
    workflow = ".github/workflows/windows-distribution.yml"
    desktop_main = "apps/desktop/src-tauri/src/main.rs"

    require(package, '$TauriCliVersion = "2.11.4"')
    require(package, "tauri-cli-v$TauriCliVersion")
    require(package, '$TauriNsisTemplateBlobSha = "d372e3c391770cf231db974422a1e4f8adaac3a6"')
    require(package, "Get-GitBlobSha1")
    require(package, "TAURI_NSIS_TEMPLATE_BLOB_SHA_MISMATCH")
    require(package, 'MULTIUSER_INSTALLMODE_INSTDIR "TurkuazLabs\\${PRODUCTNAME}"')
    require(package, '$PROGRAMFILES64\\TurkuazLabs\\${PRODUCTNAME}')
    require(package, '$PROGRAMFILES\\TurkuazLabs\\${PRODUCTNAME}')
    require(package, '$LOCALAPPDATA\\TurkuazLabs\\${PRODUCTNAME}')
    require(package, '"template": "windows/installer.generated.nsi"')
    require(package, "Remove-Item -LiteralPath $GeneratedPath -Force")

    require(".gitignore", "/apps/desktop/src-tauri/windows/installer.generated.nsi")

    require(desktop_main, 'const WINDOWS_USER_PROFILE_ENV: &str = "USERPROFILE";')
    require(desktop_main, 'const USER_DATA_ROOT_ENV: &str = "TURKUAZVM_USER_DATA_ROOT";')
    require(desktop_main, 'const USER_DATA_VM_DIRECTORY: &str = "VMs";')
    require(desktop_main, 'const USER_DATA_ISO_DIRECTORY: &str = "ISOs";')
    require(desktop_main, 'const USER_DATA_IMAGE_DIRECTORY: &str = "Images";')
    require(desktop_main, 'const USER_DATA_ANDROID_IMAGE_DIRECTORY: &str = "Android";')
    require(desktop_main, "windows_user_data_root()?")
    require(desktop_main, '"  data_root: ./data"')
    require(desktop_main, '"  installer_media: ./data/installer-media"')
    require(desktop_main, '"  android_images: ./data/android-image-builds"')

    require(smoke, '$BrandInstallRelativePath = "TurkuazLabs\\TurkuazVM"')
    require(smoke, "CURRENT_USER_INSTALL_ROOT_MISMATCH")
    require(smoke, "ALL_USERS_INSTALL_ROOT_MISMATCH")
    require(smoke, "WINDOWS_TURKUAZLABS_INSTALL_ROOT_SMOKE=PASS")
    require(smoke, '$RuntimeRoot = Join-Path $SmokeLocalAppData "TurkuazVM"')
    require(smoke, '$env:TURKUAZVM_USER_DATA_ROOT = $UserDataRoot')
    require(smoke, '$ExpectedVmRoot = Join-Path $UserDataRoot "VMs"')
    require(smoke, '$ExpectedIsoRoot = Join-Path $UserDataRoot "ISOs"')
    require(smoke, '$ExpectedAndroidImageRoot = Join-Path $UserDataRoot "Images/Android"')
    require(smoke, "WINDOWS_USER_DATA_ROOT_SMOKE=PASS")

    require(workflow, "@tauri-apps/cli@2.11.4")
    require(workflow, "test_v0416_turkuazlabs_install_root_contract.py")
    require(workflow, "DISTRIBUTION_CONTRACT_FAILED")
    require(workflow, "if ($LASTEXITCODE -ne 0)")
    require(workflow, "TurkuazLabs install root")
    forbid(workflow, "@tauri-apps/cli@2\n")

    print("V0416_TURKUAZLABS_INSTALL_ROOT_CONTRACT=PASS")


if __name__ == "__main__":
    main()
