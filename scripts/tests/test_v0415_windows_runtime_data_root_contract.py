# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0415_windows_runtime_data_root_contract.py
# 📌 Amac: v0.41.5 Windows installed/portable runtime data-root ve dual install-mode ayrimini statik olarak dogrular
# 📌 Modul - Python Test
# Version: 0.41.5
# Aciklama: LOCALAPPDATA runtime materialization'ini, portable marker bypass'ini ve current-user/per-machine gercek installer smoke baglantisini fail-closed korur
# Bagimli Oldugu Katman: Tool | CI/CD | View

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def require(path: str, needle: str) -> None:
    content = (ROOT / path).read_text(encoding="utf-8")
    if needle not in content:
        raise AssertionError(f"{path}: missing {needle!r}")


def main() -> None:
    desktop_main = "apps/desktop/src-tauri/src/main.rs"
    require(desktop_main, 'const PORTABLE_MARKER_FILE: &str = "README-PORTABLE.txt";')
    require(desktop_main, 'const WINDOWS_LOCAL_APP_DATA_ENV: &str = "LOCALAPPDATA";')
    require(desktop_main, "materialize_windows_runtime_config(executable_directory)?;")
    require(desktop_main, 'Path::new(&local_app_data).join(RUNTIME_PRODUCT_DIRECTORY)')
    require(desktop_main, 'fs::create_dir_all(runtime_root.join("data"))')
    require(desktop_main, 'fs::create_dir_all(runtime_root.join("packages"))')
    require(desktop_main, "RUNTIME_DOWNLOAD_SOURCES_FILE")
    require(desktop_main, "false,")
    require(desktop_main, 'display_executable_path: bin/turkuazvm-display.exe')
    require(desktop_main, 'executable_path: bin/turkuazvm-engine.exe')
    require(desktop_main, 'managed_helper_path: scripts/network_windows_managed.ps1')
    require(desktop_main, 'build_script: guest/android-image/scripts/build_turkuaz_android_image.sh')
    require(desktop_main, "env::set_current_dir(&runtime_root)")

    tauri_windows = "apps/desktop/src-tauri/tauri.windows.conf.json5"
    require(tauri_windows, '"installMode": "both"')

    package_script = "scripts/package_windows_distribution.ps1"
    require(package_script, 'Join-Path $PortableStage "README-PORTABLE.txt"')
    require(package_script, '"display_executable_path: bin/turkuazvm-display.exe"')
    require(package_script, '"executable_path: bin/turkuazvm-engine.exe"')

    smoke = "scripts/test_windows_installer_smoke.ps1"
    require(smoke, '$SmokeLocalAppData = Join-Path ([System.IO.Path]::GetTempPath())')
    require(smoke, '$env:LOCALAPPDATA = $SmokeLocalAppData')
    require(smoke, 'ValidateSet("/CurrentUser", "/AllUsers")')
    require(smoke, 'ModeArgument "/CurrentUser"')
    require(smoke, 'ModeArgument "/AllUsers"')
    require(smoke, 'ExpectedScope "CurrentUser"')
    require(smoke, 'ExpectedScope "AllUsers"')
    require(smoke, 'Start-Process -FilePath $InstalledExe -PassThru')
    require(smoke, 'Wait-FileCreated -Path $RuntimeConfigPath -Process $DesktopProcess')
    require(smoke, 'RUNTIME_CONFIG_ABSOLUTE_ASSET_PATH_MISSING')
    require(smoke, 'UNINSTALL_REMOVED_USER_RUNTIME_CONFIG')
    require(smoke, 'WINDOWS_RUNTIME_DATA_ROOT_SMOKE=PASS')
    require(smoke, 'WINDOWS_DUAL_INSTALL_MODE_SMOKE=PASS')

    workflow = ".github/workflows/windows-distribution.yml"
    require(workflow, "test_v0415_windows_runtime_data_root_contract.py")
    require(workflow, "test_windows_installer_smoke.ps1")
    require(workflow, "current-user, per-machine, AppData runtime")

    print("V0415_WINDOWS_RUNTIME_DATA_ROOT_CONTRACT=PASS")


if __name__ == "__main__":
    main()
