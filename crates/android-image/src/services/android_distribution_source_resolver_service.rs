// # 📄 Dosya Yolu: /turkuazvm/crates/android-image/src/services/android_distribution_source_resolver_service.rs
// # 📌 Amac: Android release policy, resmi online provider ve last-known-good source cache secim kurallarini uygular
// # 📌 Modul - Rust
// # Version: 0.40.0
// # Aciklama: Online discovery birinci, dogrulanmis immutable cache ikinci yol olacak sekilde Android source resolution is kuralini merkezilestirir
// # Bagimli Oldugu Katman: Service | Repo | Tool

use crate::domain::build_profile::AndroidImageArchitecture;
use crate::domain::distribution_source::{
    AndroidDistributionResolverPolicy, AndroidDistributionSource, AndroidDistributionSourceError,
    AndroidDistributionSourceRequest,
};
use crate::domain::image::AndroidImage;
use crate::ports::android_distribution_source_cache_port::AndroidDistributionSourceCachePort;
use crate::ports::android_distribution_source_provider_port::AndroidDistributionSourceProviderPort;
use crate::ports::android_distribution_source_resolver_port::AndroidDistributionSourceResolverPort;

pub struct AndroidDistributionSourceResolverService<P, C>
where
    P: AndroidDistributionSourceProviderPort,
    C: AndroidDistributionSourceCachePort,
{
    policy: AndroidDistributionResolverPolicy,
    provider: P,
    cache: C,
}

impl<P, C> AndroidDistributionSourceResolverService<P, C>
where
    P: AndroidDistributionSourceProviderPort,
    C: AndroidDistributionSourceCachePort,
{
    pub const fn new(policy: AndroidDistributionResolverPolicy, provider: P, cache: C) -> Self {
        Self { policy, provider, cache }
    }

    fn request_for(&self, image: &AndroidImage) -> Result<AndroidDistributionSourceRequest, AndroidDistributionSourceError> {
        if image.architecture != AndroidImageArchitecture::X86_64 {
            return Err(AndroidDistributionSourceError::UnsupportedArchitecture);
        }

        let mut base_urls = vec![self.policy.primary_base_url.clone()];
        if self.policy.use_official_fallback && self.policy.official_base_url != self.policy.primary_base_url {
            base_urls.push(self.policy.official_base_url.clone());
        }

        let Some(release) = image.requested_release.as_deref() else {
            return Ok(AndroidDistributionSourceRequest {
                release: None,
                architecture: image.architecture,
                expected_sdk: None,
                allow_device_bootloader_fallback: false,
                base_urls,
                branch_candidates: vec![self.policy.default_branch.clone()],
                target_candidates: vec![self.policy.default_target.clone()],
            });
        };

        let channel = self.policy.channels.get(release).ok_or_else(|| {
            AndroidDistributionSourceError::Policy(format!("Android {release} icin CI resolver policy tanimli degil"))
        })?;

        let mut branches = Vec::new();
        for hint in &channel.branch_hints {
            if !branches.contains(hint) {
                branches.push(hint.clone());
            }
        }
        for template in &self.policy.branch_templates {
            let candidate = template.replace("{release}", release);
            if !branches.contains(&candidate) {
                branches.push(candidate);
            }
        }

        if branches.is_empty() || self.policy.target_candidates.is_empty() {
            return Err(AndroidDistributionSourceError::Policy(format!(
                "Android {release} resolver branch/target adaylari bos"
            )));
        }

        Ok(AndroidDistributionSourceRequest {
            release: Some(release.to_owned()),
            architecture: image.architecture,
            expected_sdk: Some(channel.expected_sdk),
            allow_device_bootloader_fallback: channel.allow_device_bootloader_fallback,
            base_urls,
            branch_candidates: branches,
            target_candidates: self.policy.target_candidates.clone(),
        })
    }
}

impl<P, C> AndroidDistributionSourceResolverPort for AndroidDistributionSourceResolverService<P, C>
where
    P: AndroidDistributionSourceProviderPort,
    C: AndroidDistributionSourceCachePort,
{
    fn resolve(&mut self, image: &AndroidImage) -> Result<AndroidDistributionSource, AndroidDistributionSourceError> {
        let request = self.request_for(image)?;
        match self.provider.resolve(&request) {
            Ok(source) => {
                let _ = self.cache.save(&source);
                Ok(source)
            }
            Err(provider_error) => {
                match self.cache.load(&request.cache_key()) {
                    Ok(Some(source)) if source.matches_request(&request) => Ok(source),
                    Ok(Some(_)) => Err(AndroidDistributionSourceError::Unavailable(format!(
                        "online Android source discovery basarisiz ve last-known-good cache request ile uyusmuyor: {provider_error:?}"
                    ))),
                    Ok(None) => Err(AndroidDistributionSourceError::Unavailable(format!(
                        "online Android source discovery basarisiz ve last-known-good cache yok: {provider_error:?}"
                    ))),
                    Err(cache_error) => Err(AndroidDistributionSourceError::Unavailable(format!(
                        "online Android source discovery basarisiz; cache de okunamadi: provider={provider_error:?} cache={cache_error:?}"
                    ))),
                }
            }
        }
    }
}
