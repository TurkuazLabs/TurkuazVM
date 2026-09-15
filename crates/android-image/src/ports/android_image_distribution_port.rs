// # 📄 Dosya Yolu: /turkuazvm/crates/android-image/src/ports/android_image_distribution_port.rs
// # 📌 Amac: Hazir Android dagitim image'larini indirip TurkuazVM bundle'ina donusturen Tool contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.40.13
// # Aciklama: Android SDK ve legacy CI dis dagitim kaynaklarini Android Image Service'ten ayiran typed install, progress ve registration modellerini tutar
// # Bagimli Oldugu Katman: Service | Tool

use crate::domain::artifact::AndroidImageArtifact;
use crate::domain::image::{AndroidImage, AndroidImageCapabilities, AndroidImageRuntimeKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidImageDistributionRegistration {
    pub source_revision: Option<String>,
    pub android_release: Option<String>,
    pub sdk_level: Option<u32>,
    pub runtime_kind: AndroidImageRuntimeKind,
    pub artifacts: Vec<AndroidImageArtifact>,
    pub capabilities: AndroidImageCapabilities,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndroidImageDistributionStage {
    Discovering,
    CheckingDisk,
    DownloadingDevice,
    DownloadingHost,
    Validating,
    Extracting,
    Assembling,
    Finalizing,
    Cancelling,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidImageDistributionProgress {
    pub stage: AndroidImageDistributionStage,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub detail: String,
    pub log_path: String,
    pub elapsed_seconds: u64,
    pub bytes_per_second: Option<u64>,
    pub eta_seconds: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndroidImageDistributionError {
    UnsupportedArchitecture,
    ToolUnavailable(String),
    Discovery(String),
    Download(String),
    Archive(String),
    Artifact(String),
    Install(String),
    Cancelled,
}

pub trait AndroidImageDistributionPort {
    fn prepare_install(&self, image: &AndroidImage) -> Result<(), AndroidImageDistributionError>;

    fn install(
        &self,
        image: &AndroidImage,
    ) -> Result<AndroidImageDistributionRegistration, AndroidImageDistributionError>;

    fn progress(
        &self,
        image: &AndroidImage,
    ) -> Result<Option<AndroidImageDistributionProgress>, AndroidImageDistributionError>;

    fn cancel(&self, image: &AndroidImage) -> Result<(), AndroidImageDistributionError>;

    fn cleanup(&self, image: &AndroidImage) -> Result<(), AndroidImageDistributionError>;
}
