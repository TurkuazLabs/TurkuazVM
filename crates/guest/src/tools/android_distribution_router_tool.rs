// # 📄 Dosya Yolu: /turkuazvm/crates/guest/src/tools/android_distribution_router_tool.rs
// # 📌 Amac: Android surumunu host ve release politikasina gore resmi SDK, legacy CI veya source-build providerina yonlendirir
// # 📌 Modul - Rust
// # Version: 0.40.13
// # Aciklama: Windows'ta Android SDK Emulator providerini varsayilan yapar; Cuttlefish CI yolunu legacy/fallback olarak korur
// # Bagimli Oldugu Katman: Service | Tool

use std::collections::BTreeMap;

use turkuazvm_android_image::domain::image::AndroidImage;
use turkuazvm_android_image::ports::android_image_distribution_port::{
    AndroidImageDistributionError, AndroidImageDistributionPort, AndroidImageDistributionProgress,
    AndroidImageDistributionRegistration,
};

use crate::tools::android_ci_distribution_tool::AndroidCiDistributionTool;
use crate::tools::android_sdk_distribution_tool::AndroidSdkDistributionTool;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndroidDistributionProvider {
    AndroidSdk,
    AndroidCi,
    SourceBuild,
}

#[derive(Debug, Clone)]
pub struct AndroidDistributionRouterTool {
    android_ci: AndroidCiDistributionTool,
    android_sdk: AndroidSdkDistributionTool,
    release_providers: BTreeMap<String, AndroidDistributionProvider>,
}

impl AndroidDistributionRouterTool {
    pub fn new(
        android_ci: AndroidCiDistributionTool,
        android_sdk: AndroidSdkDistributionTool,
        release_providers: BTreeMap<String, AndroidDistributionProvider>,
    ) -> Self {
        Self { android_ci, android_sdk, release_providers }
    }

    fn provider_for(&self, image: &AndroidImage) -> Result<AndroidDistributionProvider, AndroidImageDistributionError> {
        let Some(release) = image.requested_release.as_deref() else {
            return Ok(if cfg!(windows) { AndroidDistributionProvider::AndroidSdk } else { AndroidDistributionProvider::AndroidCi });
        };
        self.release_providers.get(release).copied().ok_or_else(|| AndroidImageDistributionError::ToolUnavailable(format!("Android {release} icin dagitim provider politikasi tanimli degil")))
    }

    fn source_build_required(image: &AndroidImage) -> AndroidImageDistributionError {
        let release = image.requested_release.as_deref().unwrap_or("custom");
        AndroidImageDistributionError::ToolUnavailable(format!("Android {release} icin hazir resmi binary provider tanimli degil. Goruntu Merkezi > Build Plan ile AOSP source build kullanilmasi gerekiyor"))
    }
}

impl AndroidImageDistributionPort for AndroidDistributionRouterTool {
    fn prepare_install(&self, image: &AndroidImage) -> Result<(), AndroidImageDistributionError> {
        match self.provider_for(image)? {
            AndroidDistributionProvider::AndroidSdk => self.android_sdk.prepare_install(image),
            AndroidDistributionProvider::AndroidCi => self.android_ci.prepare_install(image),
            AndroidDistributionProvider::SourceBuild => Err(Self::source_build_required(image)),
        }
    }

    fn install(&self, image: &AndroidImage) -> Result<AndroidImageDistributionRegistration, AndroidImageDistributionError> {
        match self.provider_for(image)? {
            AndroidDistributionProvider::AndroidSdk => self.android_sdk.install(image),
            AndroidDistributionProvider::AndroidCi => self.android_ci.install(image),
            AndroidDistributionProvider::SourceBuild => Err(Self::source_build_required(image)),
        }
    }

    fn progress(&self, image: &AndroidImage) -> Result<Option<AndroidImageDistributionProgress>, AndroidImageDistributionError> {
        match self.provider_for(image)? {
            AndroidDistributionProvider::AndroidSdk => self.android_sdk.progress(image),
            AndroidDistributionProvider::AndroidCi => self.android_ci.progress(image),
            AndroidDistributionProvider::SourceBuild => Ok(None),
        }
    }

    fn cancel(&self, image: &AndroidImage) -> Result<(), AndroidImageDistributionError> {
        match self.provider_for(image)? {
            AndroidDistributionProvider::AndroidSdk => self.android_sdk.cancel(image),
            AndroidDistributionProvider::AndroidCi => self.android_ci.cancel(image),
            AndroidDistributionProvider::SourceBuild => Err(Self::source_build_required(image)),
        }
    }

    fn cleanup(&self, image: &AndroidImage) -> Result<(), AndroidImageDistributionError> {
        match self.provider_for(image)? {
            AndroidDistributionProvider::AndroidSdk => self.android_sdk.cleanup(image),
            AndroidDistributionProvider::AndroidCi => self.android_ci.cleanup(image),
            AndroidDistributionProvider::SourceBuild => {
                self.android_sdk.cleanup(image)?;
                self.android_ci.cleanup(image)
            }
        }
    }
}
