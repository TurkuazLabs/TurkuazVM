// # 📄 Dosya Yolu: /turkuazvm/crates/android-image/src/ports/android_image_builder_port.rs
// # 📌 Amac: AOSP build environment plan ve artifact tarama adapter contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.40.13
// # Aciklama: Linux/AOSP shell araclarini Android Image Service'ten ayirir
// # Bagimli Oldugu Katman: Tool

use crate::domain::artifact::AndroidImageArtifact;
use crate::domain::image::{AndroidImage, AndroidImageRuntimeKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidImageBuildPlan {
    pub supported_host: bool,
    pub source_root: String,
    pub output_root: String,
    pub branch: String,
    pub lunch_target: String,
    pub build_script: String,
    pub minimum_free_disk_gib: u64,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidImageBuildRegistration {
    pub source_revision: Option<String>,
    pub android_release: Option<String>,
    pub sdk_level: Option<u32>,
    pub runtime_kind: AndroidImageRuntimeKind,
    pub artifacts: Vec<AndroidImageArtifact>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndroidImageBuilderError {
    UnsupportedHost,
    SourceRootUnavailable,
    OutputUnavailable,
    ArtifactScan(String),
}

pub trait AndroidImageBuilderPort {
    fn build_plan(&self, image: &AndroidImage) -> Result<AndroidImageBuildPlan, AndroidImageBuilderError>;
    fn register_build(&self, image: &AndroidImage) -> Result<AndroidImageBuildRegistration, AndroidImageBuilderError>;
}
