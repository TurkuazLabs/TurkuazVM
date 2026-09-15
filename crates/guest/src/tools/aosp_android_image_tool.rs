// # 📄 Dosya Yolu: /turkuazvm/crates/guest/src/tools/aosp_android_image_tool.rs
// # 📌 Amac: AOSP Android image build plani ve build artifact tarama adapterini uygular
// # 📌 Modul - Rust
// # Version: 0.40.13
// # Aciklama: Linux AOSP source rootunu dogrular, build script komutunu uretir ve bundle image dosyalarini checksum ile kaydeder
// # Bagimli Oldugu Katman: Tool | Repo

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use turkuazvm_android_image::domain::artifact::{AndroidImageArtifact, AndroidImageArtifactRole};
use turkuazvm_android_image::domain::image::{AndroidImage, AndroidImageRuntimeKind};
use turkuazvm_android_image::ports::android_image_builder_port::{AndroidImageBuildPlan, AndroidImageBuildRegistration, AndroidImageBuilderError, AndroidImageBuilderPort};

const MIN_AOSP_FREE_DISK_GIB: u64 = 400;

#[derive(Debug, Clone)]
pub struct AospAndroidImageSettings {
    pub source_root: PathBuf,
    pub output_root: PathBuf,
    pub build_script: PathBuf,
}

#[derive(Debug, Clone)]
pub struct AospAndroidImageTool {
    settings: AospAndroidImageSettings,
}

impl AospAndroidImageTool {
    pub fn new(settings: AospAndroidImageSettings) -> Self { Self { settings } }
    fn bundle_root(&self, image: &AndroidImage) -> PathBuf { self.settings.output_root.join(image.id.as_str()) }
}

impl AndroidImageBuilderPort for AospAndroidImageTool {
    fn build_plan(&self, image: &AndroidImage) -> Result<AndroidImageBuildPlan, AndroidImageBuilderError> {
        let linux_host = cfg!(target_os = "linux");
        let source_ready = self.settings.source_root.join("build/envsetup.sh").is_file();
        let script_ready = self.settings.build_script.is_file();
        let supported_host = linux_host && source_ready && script_ready;
        if supported_host {
            fs::create_dir_all(&self.settings.output_root).map_err(|_| AndroidImageBuilderError::OutputUnavailable)?;
        }
        let profile = image.build_profile;
        let lunch_target = format!("{}-{}-{}", profile.product.code(), profile.release_config.code(), profile.variant.code());
        let mut notes = vec![
            String::from("AOSP build host must be 64-bit Linux"),
            String::from("TurkuazInputAgent is copied into the custom product before build"),
            String::from("ARM translation is intentionally not included"),
        ];
        if !linux_host { notes.push(String::from("Current Engine host is not Linux")); }
        if !source_ready { notes.push(String::from("AOSP source_root build/envsetup.sh is unavailable")); }
        if !script_ready { notes.push(String::from("Turkuaz Android build script is unavailable")); }
        Ok(AndroidImageBuildPlan {
            supported_host,
            source_root: self.settings.source_root.display().to_string(),
            output_root: self.bundle_root(image).display().to_string(),
            branch: profile.source_track.branch().to_owned(),
            lunch_target,
            build_script: self.settings.build_script.display().to_string(),
            minimum_free_disk_gib: MIN_AOSP_FREE_DISK_GIB,
            notes,
        })
    }

    fn register_build(&self, image: &AndroidImage) -> Result<AndroidImageBuildRegistration, AndroidImageBuilderError> {
        let root = self.bundle_root(image);
        if !root.is_dir() { return Err(AndroidImageBuilderError::OutputUnavailable); }
        let mut artifacts = Vec::new();
        for entry in fs::read_dir(&root).map_err(|error| AndroidImageBuilderError::ArtifactScan(error.to_string()))? {
            let entry = entry.map_err(|error| AndroidImageBuilderError::ArtifactScan(error.to_string()))?;
            let path = entry.path();
            let file_type = entry.file_type().map_err(|error| AndroidImageBuilderError::ArtifactScan(error.to_string()))?;
            if file_type.is_symlink() {
                return Err(AndroidImageBuilderError::ArtifactScan(String::from("artifact symlinks are not allowed")));
            }
            if !file_type.is_file() { continue; }
            let file_name = path.file_name().and_then(|value| value.to_str()).unwrap_or_default();
            let Some(role) = artifact_role(file_name) else { continue; };
            let metadata = fs::metadata(&path).map_err(|error| AndroidImageBuilderError::ArtifactScan(error.to_string()))?;
            if metadata.len() == 0 { continue; }
            let relative = path.strip_prefix(&root).map_err(|error| AndroidImageBuilderError::ArtifactScan(error.to_string()))?.to_string_lossy().replace('\\', "/");
            let sha256 = sha256_file(&path)?;
            artifacts.push(AndroidImageArtifact::create(role, relative, metadata.len(), sha256).map_err(|error| AndroidImageBuilderError::ArtifactScan(format!("{error:?}")))?);
        }
        artifacts.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
        let build_info = read_build_info(&root.join("build-info.yml"))?;
        Ok(AndroidImageBuildRegistration {
            source_revision: build_info.source_revision,
            android_release: build_info.android_release,
            sdk_level: build_info.sdk_level,
            runtime_kind: AndroidImageRuntimeKind::QemuComposite,
            artifacts,
        })
    }
}

#[derive(Debug, Default, serde::Deserialize)]
struct BuildInfoDto {
    source_revision: Option<String>,
    android_release: Option<String>,
    sdk_level: Option<u32>,
}

fn read_build_info(path: &Path) -> Result<BuildInfoDto, AndroidImageBuilderError> {
    if !path.is_file() { return Ok(BuildInfoDto::default()); }
    let content = fs::read_to_string(path).map_err(|error| AndroidImageBuilderError::ArtifactScan(error.to_string()))?;
    let mut info: BuildInfoDto = serde_yaml_ng::from_str(&content)
        .map_err(|error| AndroidImageBuilderError::ArtifactScan(error.to_string()))?;
    info.sdk_level = info.sdk_level.filter(|value| *value > 0);
    Ok(info)
}

fn artifact_role(file_name: &str) -> Option<AndroidImageArtifactRole> {
    match file_name {
        "boot.img" => Some(AndroidImageArtifactRole::Boot),
        "init_boot.img" => Some(AndroidImageArtifactRole::InitBoot),
        "vendor_boot.img" => Some(AndroidImageArtifactRole::VendorBoot),
        "super.img" => Some(AndroidImageArtifactRole::Super),
        "userdata.img" => Some(AndroidImageArtifactRole::Userdata),
        "vbmeta.img" => Some(AndroidImageArtifactRole::Vbmeta),
        "vbmeta_system.img" => Some(AndroidImageArtifactRole::VbmetaSystem),
        "metadata.img" => Some(AndroidImageArtifactRole::Metadata),
        "misc.img" => Some(AndroidImageArtifactRole::Misc),
        "bootloader.qemu" => Some(AndroidImageArtifactRole::Bootloader),
        "composite.img" => Some(AndroidImageArtifactRole::CompositeDisk),
        "kernel" | "Image" => Some(AndroidImageArtifactRole::Kernel),
        value if value.ends_with(".img") => Some(AndroidImageArtifactRole::Other),
        _ => None,
    }
}

fn sha256_file(path: &Path) -> Result<String, AndroidImageBuilderError> {
    let mut file = fs::File::open(path).map_err(|error| AndroidImageBuilderError::ArtifactScan(error.to_string()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|error| AndroidImageBuilderError::ArtifactScan(error.to_string()))?;
        if read == 0 { break; }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
