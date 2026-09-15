// # 📄 Dosya Yolu: /turkuazvm/crates/android-image/src/domain/distribution_source.rs
// # 📌 Amac: Android hazir dagitim kaynak kesfi icin typed request, policy ve resolved source modellerini tanimlar
// # 📌 Modul - Rust
// # Version: 0.40.0
// # Aciklama: Android CI branch/target/build secimini download Tool kodundan ayirir ve last-known-good cache icin immutable kaynak modelini saglar
// # Bagimli Oldugu Katman: Service | Repo | Tool

use std::collections::BTreeMap;

use crate::domain::build_profile::AndroidImageArchitecture;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidDistributionChannelPolicy {
    pub expected_sdk: u32,
    pub allow_device_bootloader_fallback: bool,
    pub branch_hints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidDistributionResolverPolicy {
    pub primary_base_url: String,
    pub official_base_url: String,
    pub use_official_fallback: bool,
    pub branch_templates: Vec<String>,
    pub target_candidates: Vec<String>,
    pub default_branch: String,
    pub default_target: String,
    pub channels: BTreeMap<String, AndroidDistributionChannelPolicy>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidDistributionSourceRequest {
    pub release: Option<String>,
    pub architecture: AndroidImageArchitecture,
    pub expected_sdk: Option<u32>,
    pub allow_device_bootloader_fallback: bool,
    pub base_urls: Vec<String>,
    pub branch_candidates: Vec<String>,
    pub target_candidates: Vec<String>,
}

impl AndroidDistributionSourceRequest {
    pub fn cache_key(&self) -> String {
        let release = self.release.as_deref().unwrap_or("latest");
        format!("{}::{}", release, self.architecture.code())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidDistributionSource {
    pub cache_key: String,
    pub provider: String,
    pub base_url: String,
    pub branch: String,
    pub target: String,
    pub build_id: String,
    pub artifact_base_url: String,
    pub device_artifact_name: String,
    pub host_artifact_name: String,
    pub host_package_available: bool,
    pub allow_device_bootloader_fallback: bool,
    pub android_release: Option<String>,
    pub sdk_level: Option<u32>,
    pub resolved_at_unix: u64,
}

impl AndroidDistributionSource {
    pub fn matches_request(&self, request: &AndroidDistributionSourceRequest) -> bool {
        if self.cache_key != request.cache_key() {
            return false;
        }
        if let (Some(expected), Some(actual)) = (request.expected_sdk, self.sdk_level) {
            if expected != actual {
                return false;
            }
        }
        if !self.host_package_available && !request.allow_device_bootloader_fallback {
            return false;
        }
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndroidDistributionSourceError {
    UnsupportedArchitecture,
    Policy(String),
    Provider(String),
    Cache(String),
    Unavailable(String),
}
