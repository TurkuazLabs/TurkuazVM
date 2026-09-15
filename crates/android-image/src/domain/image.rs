// # 📄 Dosya Yolu: /turkuazvm/crates/android-image/src/domain/image.rs
// # 📌 Amac: Android image aggregate, capability ve VM assignment modellerini tanimlar
// # 📌 Modul - Rust
// # Version: 0.40.13
// # Aciklama: Image registry state ve build sonucunu AOSP adapter detaylarindan bagimsiz tutar
// # Bagimli Oldugu Katman: Service | Repo | Tool | View

use crate::domain::artifact::{validate_boot_candidate, validate_sdk_emulator_candidate, AndroidImageArtifact, AndroidImageArtifactError};
use crate::domain::build_profile::{AndroidImageArchitecture, AndroidImageBuildProfile};

const MAX_ID_LEN: usize = 80;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AndroidImageId(String);

impl AndroidImageId {
    pub fn parse(value: impl Into<String>) -> Result<Self, AndroidImageError> {
        let value = value.into();
        let valid = !value.is_empty()
            && value.len() <= MAX_ID_LEN
            && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'));
        valid.then_some(Self(value)).ok_or(AndroidImageError::InvalidImageId)
    }

    pub fn as_str(&self) -> &str { &self.0 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndroidImageState {
    Defined,
    BuildPlanned,
    Installing,
    Ready,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AndroidImageCapabilities {
    pub adb_tcp: bool,
    pub guest_agent_included: bool,
    pub persistent_multi_touch: bool,
    pub native_x86_64: bool,
    pub arm_translation: bool,
}

impl AndroidImageCapabilities {
    pub const fn turkuaz_x86_64_foundation() -> Self {
        Self {
            adb_tcp: true,
            guest_agent_included: true,
            persistent_multi_touch: true,
            native_x86_64: true,
            arm_translation: false,
        }
    }

    pub const fn stock_cuttlefish_x86_64() -> Self {
        Self {
            adb_tcp: true,
            guest_agent_included: false,
            persistent_multi_touch: false,
            native_x86_64: true,
            arm_translation: false,
        }
    }

    pub const fn stock_sdk_emulator_x86_64() -> Self {
        Self {
            adb_tcp: true,
            guest_agent_included: false,
            persistent_multi_touch: false,
            native_x86_64: true,
            arm_translation: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndroidImageRuntimeKind {
    QemuComposite,
    SdkEmulator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidImage {
    pub id: AndroidImageId,
    pub name: String,
    pub architecture: AndroidImageArchitecture,
    pub build_profile: AndroidImageBuildProfile,
    pub state: AndroidImageState,
    pub capabilities: AndroidImageCapabilities,
    pub requested_release: Option<String>,
    pub source_revision: Option<String>,
    pub android_release: Option<String>,
    pub sdk_level: Option<u32>,
    pub runtime_kind: AndroidImageRuntimeKind,
    pub artifacts: Vec<AndroidImageArtifact>,
    pub last_error: Option<String>,
}

impl AndroidImage {
    pub fn define(id: AndroidImageId, name: String, build_profile: AndroidImageBuildProfile, requested_release: Option<String>) -> Result<Self, AndroidImageError> {
        if name.trim().is_empty() || name.len() > 120 {
            return Err(AndroidImageError::InvalidName);
        }
        let requested_release = requested_release.map(|value| value.trim().to_owned()).filter(|value| !value.is_empty());
        Ok(Self {
            id,
            name: name.trim().to_owned(),
            architecture: build_profile.product.architecture(),
            build_profile,
            state: AndroidImageState::Defined,
            capabilities: AndroidImageCapabilities::turkuaz_x86_64_foundation(),
            requested_release,
            source_revision: None,
            android_release: None,
            sdk_level: None,
            runtime_kind: AndroidImageRuntimeKind::QemuComposite,
            artifacts: Vec::new(),
            last_error: None,
        })
    }

    pub fn mark_build_planned(&mut self) {
        self.state = AndroidImageState::BuildPlanned;
        self.last_error = None;
    }

    pub fn mark_installing(&mut self) {
        self.state = AndroidImageState::Installing;
        self.last_error = None;
    }

    pub fn register_ready(
        &mut self,
        source_revision: Option<String>,
        android_release: Option<String>,
        sdk_level: Option<u32>,
        runtime_kind: AndroidImageRuntimeKind,
        artifacts: Vec<AndroidImageArtifact>,
    ) -> Result<(), AndroidImageError> {
        match runtime_kind {
            AndroidImageRuntimeKind::QemuComposite => validate_boot_candidate(&artifacts),
            AndroidImageRuntimeKind::SdkEmulator => validate_sdk_emulator_candidate(&artifacts),
        }
        .map_err(AndroidImageError::Artifact)?;
        self.source_revision = source_revision;
        self.android_release = android_release;
        self.sdk_level = sdk_level;
        self.runtime_kind = runtime_kind;
        self.artifacts = artifacts;
        self.state = AndroidImageState::Ready;
        self.last_error = None;
        Ok(())
    }

    pub fn mark_failed(&mut self, error: String) {
        self.state = AndroidImageState::Failed;
        self.last_error = Some(error);
    }

    pub fn reset_after_install_cleanup(&mut self) {
        self.state = AndroidImageState::Defined;
        self.last_error = None;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndroidImageProvisioningState {
    PendingFirstBoot,
    Ready,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidImageAssignment {
    pub vm_id: String,
    pub image_id: AndroidImageId,
    pub provisioning_state: AndroidImageProvisioningState,
    pub boot_attempts: u32,
    pub last_error: Option<String>,
}

impl AndroidImageAssignment {
    pub fn create(vm_id: String, image_id: AndroidImageId) -> Result<Self, AndroidImageError> {
        let valid_vm = !vm_id.is_empty() && vm_id.len() <= 80 && vm_id.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'));
        if !valid_vm { return Err(AndroidImageError::InvalidVmId); }
        Ok(Self {
            vm_id,
            image_id,
            provisioning_state: AndroidImageProvisioningState::PendingFirstBoot,
            boot_attempts: 0,
            last_error: None,
        })
    }

    pub fn begin_boot_attempt(&mut self) {
        self.boot_attempts = self.boot_attempts.saturating_add(1);
        self.provisioning_state = AndroidImageProvisioningState::PendingFirstBoot;
        self.last_error = None;
    }

    pub fn mark_ready(&mut self) {
        self.provisioning_state = AndroidImageProvisioningState::Ready;
        self.last_error = None;
    }

    pub fn mark_failed(&mut self, error: String) {
        self.provisioning_state = AndroidImageProvisioningState::Failed;
        self.last_error = Some(error);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndroidImageError {
    InvalidImageId,
    InvalidVmId,
    InvalidName,
    Artifact(AndroidImageArtifactError),
    ImageNotReady,
}
