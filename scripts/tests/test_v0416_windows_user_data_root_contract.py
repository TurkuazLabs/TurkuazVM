# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0416_windows_user_data_root_contract.py
# 📌 Amac: Windows buyuk kullanici verisinin USERPROFILE/TurkuazVM altinda tutulmasini ve legacy disk fallbackini statik olarak kilitler
# 📌 Modul - Python Test
# Version: 0.41.6
# Aciklama: AppData metadata data_root ile VMs image_root, ISOs ve Android image koklerinin ayrimini Config -> Engine -> Storage -> QEMU -> Smoke zincirinde dogrular
# Bagimli Oldugu Katman: Config | Service | Tool | CI/CD

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def require(path: str, needle: str) -> None:
    content = read(path)
    if needle not in content:
        raise AssertionError(f"{path}: missing {needle!r}")


def main() -> None:
    config = "config/turkuazvm.yml"
    desktop = "apps/desktop/src-tauri/src/main.rs"
    engine_config = "apps/engine/src/config/engine_config.rs"
    engine_service = "apps/engine/src/services/engine_application_service.rs"
    engine_storage = "apps/engine/src/tools/engine_storage_tool.rs"
    qemu_runtime = "crates/qemu/src/domain/qemu_runtime.rs"
    qemu_builder = "crates/qemu/src/tools/qemu_command_builder.rs"
    qemu_tool = "crates/storage/src/tools/qemu_img_tool.rs"
    runtime_tool = "crates/qemu/src/tools/qemu_runtime_tool.rs"
    guest_media = "crates/guest/src/tools/local_guest_media_tool.rs"
    android_runtime = "crates/guest/src/tools/android_runtime_media_tool.rs"
    smoke = "scripts/test_windows_installer_smoke.ps1"

    require(config, "  data_root: ./data")
    require(config, "  image_root: ./data/machines")

    require(desktop, 'const WINDOWS_USER_PROFILE_ENV: &str = "USERPROFILE";')
    require(desktop, 'const USER_DATA_ROOT_ENV: &str = "TURKUAZVM_USER_DATA_ROOT";')
    require(desktop, 'const USER_DATA_VM_DIRECTORY: &str = "VMs";')
    require(desktop, 'const USER_DATA_ISO_DIRECTORY: &str = "ISOs";')
    require(desktop, 'const USER_DATA_IMAGE_DIRECTORY: &str = "Images";')
    require(desktop, 'const USER_DATA_ANDROID_IMAGE_DIRECTORY: &str = "Android";')
    require(desktop, '"  image_root: ./data/machines"')
    require(desktop, "preserve_existing_download_paths")
    require(desktop, '"  installer_media: ./data/installer-media"')
    require(desktop, '"  android_images: ./data/android-image-builds"')

    require(engine_config, "pub image_root: PathBuf,")
    require(engine_config, "#[serde(default)]\n    image_root: Option<PathBuf>,")
    require(engine_config, 'const LEGACY_MACHINE_DIRECTORY: &str = "machines";')
    require(engine_config, "unwrap_or_else(|| data_root.join(LEGACY_MACHINE_DIRECTORY))")
    require(engine_config, "image_root: image_root.clone(),")

    require(engine_service, "config.image_root.clone(),")
    require(engine_service, "HostStorageTool::new(config.image_root.clone())")
    require(engine_storage, "pub fn new(binary: Option<PathBuf>, data_root: PathBuf, image_root: PathBuf)")
    require(engine_storage, "QemuImgTool::new(binary, data_root, image_root)")

    require(qemu_runtime, "pub image_root: PathBuf,")
    require(qemu_tool, "image_root: PathBuf,")
    require(qemu_tool, "fn image_machine_root")
    require(qemu_tool, "self.image_root.join(vm_id.as_str())")
    require(qemu_tool, "fn resolve_existing_image_path")
    require(qemu_tool, "self.data_machine_root(vm_id)")
    require(qemu_tool, "self.resolve_image_target_path(target_vm_id, target)")

    require(qemu_builder, "build_arguments_with_gpu_and_image_root")
    require(qemu_builder, "image_root: &Path")
    require(qemu_builder, ".join(machine.id().as_str())")
    require(qemu_builder, ".join(DIR_MACHINES)")
    require(runtime_tool, "&self.settings.image_root")

    require(guest_media, "pub image_root: PathBuf,")
    require(guest_media, "fn primary_target_path")
    require(guest_media, "fn legacy_target_path")
    require(guest_media, ".image_root")
    require(guest_media, ".data_root")
    require(engine_service, "image_root: config.image_root.clone(),")

    require(android_runtime, "pub vm_root: PathBuf,")
    require(android_runtime, "fn vm_runtime_root")
    require(android_runtime, "self.settings.vm_root.join(vm_id)")
    require(android_runtime, ".data_root")
    require(engine_service, "config.image_root.clone(),")

    require(smoke, '$env:TURKUAZVM_USER_DATA_ROOT = $UserDataRoot')
    require(smoke, '$ExpectedVmRoot = Join-Path $UserDataRoot "VMs"')
    require(smoke, '$ExpectedIsoRoot = Join-Path $UserDataRoot "ISOs"')
    require(smoke, '$ExpectedAndroidImageRoot = Join-Path $UserDataRoot "Images/Android"')
    require(smoke, 'RUNTIME_METADATA_DATA_ROOT_NOT_LOCALAPPDATA_RELATIVE')
    require(smoke, 'USER_DATA_VM_IMAGE_ROOT_NOT_MATERIALIZED')
    require(smoke, "WINDOWS_USER_DATA_ROOT_SMOKE=PASS")

    print("V0416_WINDOWS_USER_DATA_ROOT_CONTRACT=PASS")


if __name__ == "__main__":
    main()
