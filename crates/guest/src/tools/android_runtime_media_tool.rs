// # 📄 Dosya Yolu: /turkuazvm/crates/guest/src/tools/android_runtime_media_tool.rs
// # 📌 Amac: Kayitli Android image bundle'ini runtime tipine gore QEMU veya Android SDK Emulator medyasina hazirlar
// # 📌 Modul - Rust
// # Version: 0.41.6
// # Aciklama: VM-ozel Android private disk ve AVD userdata'yi vm_root altinda tutar; mevcut data_root runtime klasoru varsa legacy fallback ile kullanir
// # Bagimli Oldugu Katman: Service | Repo | Tool

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use sha2::{Digest, Sha256};
use turkuazvm_android::domain::runtime_profile::AndroidRuntimeProfile;
use turkuazvm_android_image::domain::artifact::{AndroidImageArtifact, AndroidImageArtifactRole};
use turkuazvm_android_image::domain::image::{AndroidImage, AndroidImageRuntimeKind};
use turkuazvm_core::domain::runtime_media::{AndroidRuntimeMediaPlan, AndroidSdkEmulatorRuntimeMediaPlan, VmRuntimeMediaPlan};

const DIR_MACHINES: &str = "machines";
const DIR_RUNTIME: &str = "runtime";
const DIR_ANDROID: &str = "android";
const DIR_ANDROID_SDK: &str = "android-sdk";
const DIR_AVD: &str = "avd";
const FILE_PFLASH: &str = "pflash.img";
const FILE_OS_PRIVATE: &str = "os-private.qcow2";
const FILE_USERDATA_PRIVATE: &str = "userdata-qemu.img";
const MIB_BYTES: u64 = 1024 * 1024;
const PFLASH_TOTAL_MIB: u64 = 4;
const QEMU_IMG_CONVERT: &str = "convert";
const QEMU_IMG_INPUT_FORMAT_FLAG: &str = "-f";
const QEMU_IMG_OUTPUT_FORMAT_FLAG: &str = "-O";
const QEMU_IMG_QCOW2: &str = "qcow2";
const QEMU_IMG_RAW: &str = "raw";

#[derive(Debug, Clone)]
pub struct AndroidRuntimeMediaSettings {
    pub data_root: PathBuf,
    pub vm_root: PathBuf,
    pub image_output_root: PathBuf,
    pub qemu_img_binary: Option<PathBuf>,
    pub android_sdk_tool_root: PathBuf,
    pub emulator_console_port_min: u16,
    pub emulator_console_port_max: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndroidRuntimeMediaError {
    MissingArtifact(AndroidImageArtifactRole),
    ArtifactUnavailable(PathBuf),
    ArtifactSizeMismatch(PathBuf),
    ArtifactChecksumMismatch(PathBuf),
    BootloaderTooLarge(u64),
    QemuImgUnavailable,
    RuntimeDirectory(String),
    PrivateDiskCreate(String),
    PflashCreate(String),
    InvalidEmulatorAdbPort(u16),
    EmulatorUnavailable(PathBuf),
    AvdCreate(String),
}

#[derive(Debug, Clone)]
pub struct AndroidRuntimeMediaTool { settings: AndroidRuntimeMediaSettings }

impl AndroidRuntimeMediaTool {
    pub fn new(settings: AndroidRuntimeMediaSettings) -> Self { Self { settings } }

    fn vm_runtime_root(&self, vm_id: &str) -> PathBuf {
        let primary = self.settings.vm_root.join(vm_id).join(DIR_RUNTIME);
        if primary.exists() {
            return primary;
        }
        let legacy = self
            .settings
            .data_root
            .join(DIR_MACHINES)
            .join(vm_id)
            .join(DIR_RUNTIME);
        if legacy.exists() {
            return legacy;
        }
        primary
    }

    pub fn prepare(&self, image: &AndroidImage, vm_id: &str, profile: &AndroidRuntimeProfile) -> Result<VmRuntimeMediaPlan, AndroidRuntimeMediaError> {
        match image.runtime_kind {
            AndroidImageRuntimeKind::QemuComposite => self.prepare_qemu(image, vm_id),
            AndroidImageRuntimeKind::SdkEmulator => self.prepare_sdk_emulator(image, vm_id, profile),
        }
    }

    fn prepare_qemu(&self, image: &AndroidImage, vm_id: &str) -> Result<VmRuntimeMediaPlan, AndroidRuntimeMediaError> {
        let composite = self.required_artifact(image, AndroidImageArtifactRole::CompositeDisk)?;
        let bootloader = self.required_artifact(image, AndroidImageArtifactRole::Bootloader)?;
        let composite_path = self.artifact_path(image, composite);
        let bootloader_path = self.artifact_path(image, bootloader);
        self.verify_artifact(&composite_path, composite)?;
        self.verify_artifact(&bootloader_path, bootloader)?;
        let runtime_root = self
            .vm_runtime_root(vm_id)
            .join(DIR_ANDROID)
            .join(image.id.as_str());
        fs::create_dir_all(&runtime_root).map_err(|error| AndroidRuntimeMediaError::RuntimeDirectory(error.to_string()))?;
        let pflash_path = runtime_root.join(FILE_PFLASH);
        self.ensure_pflash(&bootloader_path, &pflash_path)?;
        let os_disk_path = runtime_root.join(FILE_OS_PRIVATE);
        self.ensure_private_disk(&composite_path, &os_disk_path)?;
        Ok(VmRuntimeMediaPlan::Android(AndroidRuntimeMediaPlan { bootloader_path, pflash_path, os_disk_path }))
    }

    fn prepare_sdk_emulator(&self, image: &AndroidImage, vm_id: &str, profile: &AndroidRuntimeProfile) -> Result<VmRuntimeMediaPlan, AndroidRuntimeMediaError> {
        let adb_port = profile.adb_host_port();
        let Some(console_port) = adb_port.checked_sub(1) else {
            return Err(AndroidRuntimeMediaError::InvalidEmulatorAdbPort(adb_port));
        };
        if adb_port % 2 == 0
            || !(self.settings.emulator_console_port_min..=self.settings.emulator_console_port_max).contains(&console_port)
        {
            return Err(AndroidRuntimeMediaError::InvalidEmulatorAdbPort(adb_port));
        }
        let kernel = self.required_artifact(image, AndroidImageArtifactRole::Kernel)?;
        let ramdisk = self.required_artifact(image, AndroidImageArtifactRole::Ramdisk)?;
        let system = self.required_artifact(image, AndroidImageArtifactRole::System)?;
        let userdata = self.required_artifact(image, AndroidImageArtifactRole::Userdata)?;
        for artifact in [kernel, ramdisk, system, userdata] { self.verify_artifact(&self.artifact_path(image, artifact), artifact)?; }
        let system_path = self.artifact_path(image, system);
        let system_image_dir = system_path.parent().ok_or_else(|| AndroidRuntimeMediaError::AvdCreate(String::from("system.img parent directory unavailable")))?.to_path_buf();
        let emulator_binary = self.settings.android_sdk_tool_root.join("emulator").join(if cfg!(windows) { "emulator.exe" } else { "emulator" });
        if !emulator_binary.is_file() { return Err(AndroidRuntimeMediaError::EmulatorUnavailable(emulator_binary)); }

        let avd_home = self
            .vm_runtime_root(vm_id)
            .join(DIR_ANDROID_SDK)
            .join(DIR_AVD);
        fs::create_dir_all(&avd_home).map_err(|error| AndroidRuntimeMediaError::AvdCreate(error.to_string()))?;
        let avd_name = format!("turkuazvm-{vm_id}");
        let avd_content = avd_home.join(format!("{avd_name}.avd"));
        fs::create_dir_all(&avd_content).map_err(|error| AndroidRuntimeMediaError::AvdCreate(error.to_string()))?;
        let private_userdata = avd_content.join(FILE_USERDATA_PRIVATE);
        if !private_userdata.is_file() {
            fs::copy(self.artifact_path(image, userdata), &private_userdata).map_err(|error| AndroidRuntimeMediaError::AvdCreate(error.to_string()))?;
        }
        let display = profile.display();
        let sdk_level = image.sdk_level.ok_or_else(|| AndroidRuntimeMediaError::AvdCreate(String::from("SDK emulator image sdk_level unavailable")))?;
        let config = format!(
            "AvdId={avd_name}\nabi.type=x86_64\nhw.cpu.arch=x86_64\nhw.keyboard=yes\nhw.lcd.width={}\nhw.lcd.height={}\nhw.lcd.density={}\nimage.sysdir.1={}\nshowDeviceFrame=no\ndisk.dataPartition.path={}\n",
            display.width(), display.height(), display.density_dpi(), normalize_ini_path(&system_image_dir), normalize_ini_path(&private_userdata)
        );
        fs::write(avd_content.join("config.ini"), config).map_err(|error| AndroidRuntimeMediaError::AvdCreate(error.to_string()))?;
        let ini = format!("path={}\npath.rel={}\ntarget=android-{sdk_level}\n", normalize_ini_path(&avd_content), format!("{}.avd", avd_name));
        fs::write(avd_home.join(format!("{avd_name}.ini")), ini).map_err(|error| AndroidRuntimeMediaError::AvdCreate(error.to_string()))?;
        Ok(VmRuntimeMediaPlan::AndroidSdkEmulator(AndroidSdkEmulatorRuntimeMediaPlan { emulator_binary, avd_home, avd_name, system_image_dir, console_port }))
    }

    fn required_artifact<'a>(&self, image: &'a AndroidImage, role: AndroidImageArtifactRole) -> Result<&'a AndroidImageArtifact, AndroidRuntimeMediaError> {
        image.artifacts.iter().find(|artifact| artifact.role == role).ok_or(AndroidRuntimeMediaError::MissingArtifact(role))
    }

    fn artifact_path(&self, image: &AndroidImage, artifact: &AndroidImageArtifact) -> PathBuf { self.settings.image_output_root.join(image.id.as_str()).join(&artifact.relative_path) }

    fn verify_artifact(&self, path: &Path, artifact: &AndroidImageArtifact) -> Result<(), AndroidRuntimeMediaError> {
        let metadata = fs::metadata(path).map_err(|_| AndroidRuntimeMediaError::ArtifactUnavailable(path.to_path_buf()))?;
        if !metadata.is_file() { return Err(AndroidRuntimeMediaError::ArtifactUnavailable(path.to_path_buf())); }
        if metadata.len() != artifact.size_bytes { return Err(AndroidRuntimeMediaError::ArtifactSizeMismatch(path.to_path_buf())); }
        if sha256_file(path)? != artifact.sha256 { return Err(AndroidRuntimeMediaError::ArtifactChecksumMismatch(path.to_path_buf())); }
        Ok(())
    }

    fn ensure_pflash(&self, bootloader_path: &Path, pflash_path: &Path) -> Result<(), AndroidRuntimeMediaError> {
        let bootloader_size = fs::metadata(bootloader_path).map_err(|error| AndroidRuntimeMediaError::PflashCreate(error.to_string()))?.len();
        let bootloader_size_mib = bootloader_size / MIB_BYTES;
        if bootloader_size_mib >= PFLASH_TOTAL_MIB { return Err(AndroidRuntimeMediaError::BootloaderTooLarge(bootloader_size)); }
        let expected_size = (PFLASH_TOTAL_MIB - bootloader_size_mib) * MIB_BYTES;
        if pflash_path.is_file() && fs::metadata(pflash_path).map(|metadata| metadata.len() == expected_size).unwrap_or(false) { return Ok(()); }
        if pflash_path.exists() { fs::remove_file(pflash_path).map_err(|error| AndroidRuntimeMediaError::PflashCreate(error.to_string()))?; }
        let file = fs::File::create(pflash_path).map_err(|error| AndroidRuntimeMediaError::PflashCreate(error.to_string()))?;
        file.set_len(expected_size).map_err(|error| AndroidRuntimeMediaError::PflashCreate(error.to_string()))
    }

    fn ensure_private_disk(&self, composite_path: &Path, private_disk_path: &Path) -> Result<(), AndroidRuntimeMediaError> {
        if private_disk_path.is_file() && fs::metadata(private_disk_path).map(|metadata| metadata.len() > 0).unwrap_or(false) { return Ok(()); }
        if private_disk_path.exists() { fs::remove_file(private_disk_path).map_err(|error| AndroidRuntimeMediaError::PrivateDiskCreate(error.to_string()))?; }
        let binary = self.settings.qemu_img_binary.as_deref().filter(|path| path.is_file()).ok_or(AndroidRuntimeMediaError::QemuImgUnavailable)?;
        let output = Command::new(binary).arg(QEMU_IMG_CONVERT).arg(QEMU_IMG_INPUT_FORMAT_FLAG).arg(QEMU_IMG_RAW).arg(QEMU_IMG_OUTPUT_FORMAT_FLAG).arg(QEMU_IMG_QCOW2).arg(composite_path).arg(private_disk_path).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).output().map_err(|error| AndroidRuntimeMediaError::PrivateDiskCreate(error.to_string()))?;
        if output.status.success() { return Ok(()); }
        Err(AndroidRuntimeMediaError::PrivateDiskCreate(String::from_utf8_lossy(&output.stderr).trim().to_owned()))
    }
}

fn normalize_ini_path(path: &Path) -> String { path.to_string_lossy().replace('\\', "/") }

fn sha256_file(path: &Path) -> Result<String, AndroidRuntimeMediaError> {
    let mut file = fs::File::open(path).map_err(|_| AndroidRuntimeMediaError::ArtifactUnavailable(path.to_path_buf()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|_| AndroidRuntimeMediaError::ArtifactUnavailable(path.to_path_buf()))?;
        if read == 0 { break; }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
