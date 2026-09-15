// # 📄 Dosya Yolu: /turkuazvm/crates/repositories/src/repositories/yaml_android_image_repository.rs
// # 📌 Amac: Android image registry ve VM assignment verisini YAML dosyalarinda saklar
// # 📌 Modul - Rust
// # Version: 0.40.13
// # Aciklama: android-images/<id>/image.yml ve machines/<vm>/android-image.yml persistence adapterini uygular
// # Bagimli Oldugu Katman: Repo

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use serde::{Deserialize, Serialize};
use turkuazvm_android_image::domain::artifact::{AndroidImageArtifact, AndroidImageArtifactRole};
use turkuazvm_android_image::domain::build_profile::{AndroidBuildProduct, AndroidBuildVariant, AndroidImageArchitecture, AndroidImageBuildProfile, AndroidReleaseConfig, AndroidSourceTrack};
use turkuazvm_android_image::domain::image::{AndroidImage, AndroidImageAssignment, AndroidImageCapabilities, AndroidImageId, AndroidImageProvisioningState, AndroidImageRuntimeKind, AndroidImageState};
use turkuazvm_android_image::ports::android_image_repository_port::{AndroidImageRepositoryError, AndroidImageRepositoryPort};

const IMAGE_SCHEMA_VERSION: u16 = 4;
const LEGACY_IMAGE_SCHEMA_V3: u16 = 3;
const LEGACY_IMAGE_SCHEMA_V2: u16 = 2;
const LEGACY_IMAGE_SCHEMA_VERSION: u16 = 1;
const ASSIGNMENT_SCHEMA_VERSION: u16 = 2;
const LEGACY_ASSIGNMENT_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone)]
pub struct YamlAndroidImageRepository {
    data_root: PathBuf,
    access_guard: Arc<Mutex<()>>,
}

impl YamlAndroidImageRepository {
    pub fn new(data_root: PathBuf) -> Self {
        Self { data_root, access_guard: Arc::new(Mutex::new(())) }
    }

    fn lock_access(&self) -> Result<MutexGuard<'_, ()>, AndroidImageRepositoryError> {
        self.access_guard
            .lock()
            .map_err(|_| AndroidImageRepositoryError::Storage(String::from("android image repository lock poisoned")))
    }

    fn registry_root(&self) -> PathBuf { self.data_root.join("android-images") }
    fn image_path(&self, image_id: &AndroidImageId) -> PathBuf { self.registry_root().join(image_id.as_str()).join("image.yml") }
    fn assignment_path(&self, vm_id: &str) -> PathBuf { self.data_root.join("machines").join(vm_id).join("android-image.yml") }
}

impl AndroidImageRepositoryPort for YamlAndroidImageRepository {
    fn list(&self) -> Result<Vec<AndroidImage>, AndroidImageRepositoryError> {
        let _guard = self.lock_access()?;
        let root = self.registry_root();
        if !root.exists() { return Ok(Vec::new()); }
        let mut images = Vec::new();
        for entry in fs::read_dir(&root).map_err(storage_error)? {
            let entry = entry.map_err(storage_error)?;
            if !entry.file_type().map_err(storage_error)?.is_dir() { continue; }
            let path = entry.path().join("image.yml");
            if !path.is_file() { continue; }
            images.push(read_image(&path)?);
        }
        images.sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
        Ok(images)
    }

    fn get(&self, image_id: &AndroidImageId) -> Result<AndroidImage, AndroidImageRepositoryError> {
        let _guard = self.lock_access()?;
        let path = self.image_path(image_id);
        if !path.is_file() { return Err(AndroidImageRepositoryError::NotFound(image_id.clone())); }
        read_image(&path)
    }

    fn save(&self, image: AndroidImage) -> Result<(), AndroidImageRepositoryError> {
        let _guard = self.lock_access()?;
        let path = self.image_path(&image.id);
        write_yaml(&path, &ImageManifestDto::from_domain(&image))
    }

    fn get_assignment(&self, vm_id: &str) -> Result<Option<AndroidImageAssignment>, AndroidImageRepositoryError> {
        let _guard = self.lock_access()?;
        let path = self.assignment_path(vm_id);
        if !path.is_file() { return Ok(None); }
        let dto: AssignmentDto = read_yaml(&path)?;
        if dto.schema_version != ASSIGNMENT_SCHEMA_VERSION && dto.schema_version != LEGACY_ASSIGNMENT_SCHEMA_VERSION {
            return Err(AndroidImageRepositoryError::Storage(String::from("unsupported android image assignment schema")));
        }
        let image_id = AndroidImageId::parse(dto.image_id).map_err(|error| AndroidImageRepositoryError::Storage(format!("invalid assignment image id: {error:?}")))?;
        let mut assignment = AndroidImageAssignment::create(dto.vm_id, image_id)
            .map_err(|error| AndroidImageRepositoryError::Storage(format!("invalid assignment: {error:?}")))?;
        assignment.provisioning_state = parse_provisioning_state(&dto.provisioning_state)?;
        assignment.boot_attempts = dto.boot_attempts;
        assignment.last_error = dto.last_error;
        Ok(Some(assignment))
    }

    fn save_assignment(&self, assignment: AndroidImageAssignment) -> Result<(), AndroidImageRepositoryError> {
        let _guard = self.lock_access()?;
        let path = self.assignment_path(&assignment.vm_id);
        write_yaml(&path, &AssignmentDto {
            schema_version: ASSIGNMENT_SCHEMA_VERSION,
            vm_id: assignment.vm_id,
            image_id: assignment.image_id.as_str().to_owned(),
            provisioning_state: provisioning_state_code(assignment.provisioning_state).to_owned(),
            boot_attempts: assignment.boot_attempts,
            last_error: assignment.last_error,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ImageManifestDto {
    schema_version: u16,
    id: String,
    name: String,
    architecture: String,
    source_track: String,
    product: String,
    release_config: String,
    variant: String,
    state: String,
    capabilities: ImageCapabilitiesDto,
    #[serde(default)]
    requested_release: Option<String>,
    source_revision: Option<String>,
    android_release: Option<String>,
    sdk_level: Option<u32>,
    #[serde(default = "default_runtime_kind")]
    runtime_kind: String,
    artifacts: Vec<ImageArtifactDto>,
    last_error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ImageCapabilitiesDto {
    adb_tcp: bool,
    guest_agent_included: bool,
    persistent_multi_touch: bool,
    native_x86_64: bool,
    arm_translation: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ImageArtifactDto {
    role: String,
    relative_path: String,
    size_bytes: u64,
    sha256: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AssignmentDto {
    schema_version: u16,
    vm_id: String,
    image_id: String,
    #[serde(default = "default_provisioning_state")]
    provisioning_state: String,
    #[serde(default)]
    boot_attempts: u32,
    #[serde(default)]
    last_error: Option<String>,
}

impl ImageManifestDto {
    fn from_domain(image: &AndroidImage) -> Self {
        Self {
            schema_version: IMAGE_SCHEMA_VERSION,
            id: image.id.as_str().to_owned(),
            name: image.name.clone(),
            architecture: image.architecture.code().to_owned(),
            source_track: source_track_code(image.build_profile.source_track).to_owned(),
            product: image.build_profile.product.code().to_owned(),
            release_config: image.build_profile.release_config.code().to_owned(),
            variant: image.build_profile.variant.code().to_owned(),
            state: state_code(image.state).to_owned(),
            capabilities: ImageCapabilitiesDto {
                adb_tcp: image.capabilities.adb_tcp,
                guest_agent_included: image.capabilities.guest_agent_included,
                persistent_multi_touch: image.capabilities.persistent_multi_touch,
                native_x86_64: image.capabilities.native_x86_64,
                arm_translation: image.capabilities.arm_translation,
            },
            requested_release: image.requested_release.clone(),
            source_revision: image.source_revision.clone(),
            android_release: image.android_release.clone(),
            sdk_level: image.sdk_level,
            runtime_kind: runtime_kind_code(image.runtime_kind).to_owned(),
            artifacts: image.artifacts.iter().map(|artifact| ImageArtifactDto {
                role: artifact.role.code().to_owned(),
                relative_path: artifact.relative_path.clone(),
                size_bytes: artifact.size_bytes,
                sha256: artifact.sha256.clone(),
            }).collect(),
            last_error: image.last_error.clone(),
        }
    }

    fn into_domain(self) -> Result<AndroidImage, AndroidImageRepositoryError> {
        if self.schema_version != IMAGE_SCHEMA_VERSION && self.schema_version != LEGACY_IMAGE_SCHEMA_V3 && self.schema_version != LEGACY_IMAGE_SCHEMA_V2 && self.schema_version != LEGACY_IMAGE_SCHEMA_VERSION { return Err(AndroidImageRepositoryError::Storage(String::from("unsupported android image schema"))); }
        let product = parse_product(&self.product)?;
        let build_profile = AndroidImageBuildProfile {
            source_track: parse_source_track(&self.source_track)?,
            product,
            release_config: parse_release_config(&self.release_config)?,
            variant: parse_variant(&self.variant)?,
        };
        let architecture = parse_architecture(&self.architecture)?;
        if architecture != product.architecture() { return Err(AndroidImageRepositoryError::Storage(String::from("android image architecture/product mismatch"))); }
        let artifacts = self.artifacts.into_iter().map(|artifact| {
            AndroidImageArtifact::create(parse_artifact_role(&artifact.role)?, artifact.relative_path, artifact.size_bytes, artifact.sha256)
                .map_err(|error| AndroidImageRepositoryError::Storage(format!("invalid image artifact: {error:?}")))
        }).collect::<Result<Vec<_>, _>>()?;
        Ok(AndroidImage {
            id: AndroidImageId::parse(self.id).map_err(domain_storage)?,
            name: self.name,
            architecture,
            build_profile,
            state: parse_state(&self.state)?,
            capabilities: AndroidImageCapabilities {
                adb_tcp: self.capabilities.adb_tcp,
                guest_agent_included: self.capabilities.guest_agent_included,
                persistent_multi_touch: self.capabilities.persistent_multi_touch,
                native_x86_64: self.capabilities.native_x86_64,
                arm_translation: self.capabilities.arm_translation,
            },
            requested_release: self.requested_release,
            source_revision: self.source_revision,
            android_release: self.android_release,
            sdk_level: self.sdk_level,
            runtime_kind: parse_runtime_kind(&self.runtime_kind)?,
            artifacts,
            last_error: self.last_error,
        })
    }
}

fn read_image(path: &Path) -> Result<AndroidImage, AndroidImageRepositoryError> { read_yaml::<ImageManifestDto>(path)?.into_domain() }
fn read_yaml<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, AndroidImageRepositoryError> {
    let content = fs::read_to_string(path).map_err(storage_error)?;
    serde_yaml_ng::from_str(&content).map_err(|error| AndroidImageRepositoryError::Storage(error.to_string()))
}
fn write_yaml<T: Serialize>(path: &Path, value: &T) -> Result<(), AndroidImageRepositoryError> {
    let parent = path.parent().ok_or_else(|| AndroidImageRepositoryError::Storage(String::from("manifest path has no parent")))?;
    fs::create_dir_all(parent).map_err(storage_error)?;
    let content = serde_yaml_ng::to_string(value).map_err(|error| AndroidImageRepositoryError::Storage(error.to_string()))?;
    let file_name = path.file_name().and_then(|value| value.to_str()).ok_or_else(|| AndroidImageRepositoryError::Storage(String::from("manifest file name is invalid")))?;
    let temp_path = parent.join(format!(".{file_name}.tmp"));
    let backup_path = parent.join(format!(".{file_name}.backup"));
    if temp_path.exists() { fs::remove_file(&temp_path).map_err(storage_error)?; }
    if backup_path.exists() { fs::remove_file(&backup_path).map_err(storage_error)?; }
    fs::write(&temp_path, content).map_err(storage_error)?;
    let had_current = path.exists();
    if had_current { fs::rename(path, &backup_path).map_err(storage_error)?; }
    match fs::rename(&temp_path, path) {
        Ok(()) => {
            if backup_path.exists() { let _ = fs::remove_file(&backup_path); }
            Ok(())
        }
        Err(error) => {
            if had_current && backup_path.exists() && !path.exists() { let _ = fs::rename(&backup_path, path); }
            let _ = fs::remove_file(&temp_path);
            Err(storage_error(error))
        }
    }
}
fn storage_error(error: std::io::Error) -> AndroidImageRepositoryError { AndroidImageRepositoryError::Storage(error.to_string()) }
fn domain_storage(error: turkuazvm_android_image::domain::image::AndroidImageError) -> AndroidImageRepositoryError { AndroidImageRepositoryError::Storage(format!("invalid image domain: {error:?}")) }
fn source_track_code(value: AndroidSourceTrack) -> &'static str { match value { AndroidSourceTrack::LatestRelease => "latest_release" } }
fn state_code(value: AndroidImageState) -> &'static str { match value { AndroidImageState::Defined => "defined", AndroidImageState::BuildPlanned => "build_planned", AndroidImageState::Installing => "installing", AndroidImageState::Ready => "ready", AndroidImageState::Failed => "failed" } }
fn parse_architecture(value: &str) -> Result<AndroidImageArchitecture, AndroidImageRepositoryError> { match value { "x86_64" => Ok(AndroidImageArchitecture::X86_64), "arm64" => Ok(AndroidImageArchitecture::Arm64), _ => Err(AndroidImageRepositoryError::Storage(String::from("unsupported image architecture"))) } }
fn parse_source_track(value: &str) -> Result<AndroidSourceTrack, AndroidImageRepositoryError> { match value { "latest_release" => Ok(AndroidSourceTrack::LatestRelease), _ => Err(AndroidImageRepositoryError::Storage(String::from("unsupported source track"))) } }
fn parse_product(value: &str) -> Result<AndroidBuildProduct, AndroidImageRepositoryError> { match value { "turkuazvm_cf_x86_64_phone" => Ok(AndroidBuildProduct::TurkuazCuttlefishX86_64Phone), _ => Err(AndroidImageRepositoryError::Storage(String::from("unsupported build product"))) } }
fn parse_release_config(value: &str) -> Result<AndroidReleaseConfig, AndroidImageRepositoryError> { match value { "aosp_current" => Ok(AndroidReleaseConfig::AospCurrent), _ => Err(AndroidImageRepositoryError::Storage(String::from("unsupported release config"))) } }
fn parse_variant(value: &str) -> Result<AndroidBuildVariant, AndroidImageRepositoryError> { match value { "userdebug" => Ok(AndroidBuildVariant::Userdebug), _ => Err(AndroidImageRepositoryError::Storage(String::from("unsupported build variant"))) } }
fn parse_state(value: &str) -> Result<AndroidImageState, AndroidImageRepositoryError> { match value { "defined" => Ok(AndroidImageState::Defined), "build_planned" => Ok(AndroidImageState::BuildPlanned), "installing" => Ok(AndroidImageState::Installing), "ready" => Ok(AndroidImageState::Ready), "failed" => Ok(AndroidImageState::Failed), _ => Err(AndroidImageRepositoryError::Storage(String::from("unsupported image state"))) } }
fn default_runtime_kind() -> String { String::from("qemu_composite") }
fn runtime_kind_code(value: AndroidImageRuntimeKind) -> &'static str { match value { AndroidImageRuntimeKind::QemuComposite => "qemu_composite", AndroidImageRuntimeKind::SdkEmulator => "sdk_emulator" } }
fn parse_runtime_kind(value: &str) -> Result<AndroidImageRuntimeKind, AndroidImageRepositoryError> { match value { "qemu_composite" => Ok(AndroidImageRuntimeKind::QemuComposite), "sdk_emulator" => Ok(AndroidImageRuntimeKind::SdkEmulator), _ => Err(AndroidImageRepositoryError::Storage(String::from("unsupported Android image runtime kind"))) } }
fn default_provisioning_state() -> String { String::from("pending_first_boot") }
fn provisioning_state_code(value: AndroidImageProvisioningState) -> &'static str {
    match value {
        AndroidImageProvisioningState::PendingFirstBoot => "pending_first_boot",
        AndroidImageProvisioningState::Ready => "ready",
        AndroidImageProvisioningState::Failed => "failed",
    }
}
fn parse_provisioning_state(value: &str) -> Result<AndroidImageProvisioningState, AndroidImageRepositoryError> {
    match value {
        "pending_first_boot" => Ok(AndroidImageProvisioningState::PendingFirstBoot),
        "ready" => Ok(AndroidImageProvisioningState::Ready),
        "failed" => Ok(AndroidImageProvisioningState::Failed),
        _ => Err(AndroidImageRepositoryError::Storage(String::from("unsupported provisioning state"))),
    }
}
fn parse_artifact_role(value: &str) -> Result<AndroidImageArtifactRole, AndroidImageRepositoryError> { match value { "boot" => Ok(AndroidImageArtifactRole::Boot), "init_boot" => Ok(AndroidImageArtifactRole::InitBoot), "vendor_boot" => Ok(AndroidImageArtifactRole::VendorBoot), "super" => Ok(AndroidImageArtifactRole::Super), "userdata" => Ok(AndroidImageArtifactRole::Userdata), "vbmeta" => Ok(AndroidImageArtifactRole::Vbmeta), "vbmeta_system" => Ok(AndroidImageArtifactRole::VbmetaSystem), "metadata" => Ok(AndroidImageArtifactRole::Metadata), "misc" => Ok(AndroidImageArtifactRole::Misc), "bootloader" => Ok(AndroidImageArtifactRole::Bootloader), "composite_disk" => Ok(AndroidImageArtifactRole::CompositeDisk), "kernel" => Ok(AndroidImageArtifactRole::Kernel), "ramdisk" => Ok(AndroidImageArtifactRole::Ramdisk), "system" => Ok(AndroidImageArtifactRole::System), "other" => Ok(AndroidImageArtifactRole::Other), _ => Err(AndroidImageRepositoryError::Storage(String::from("unsupported artifact role"))) } }
