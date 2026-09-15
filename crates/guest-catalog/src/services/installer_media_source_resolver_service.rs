// # 📄 Dosya Yolu: /turkuazvm/crates/guest-catalog/src/services/installer_media_source_resolver_service.rs
// # 📌 Amac: Linux installer medya resmi online discovery, last-known-good cache ve katalog fallback is kurallarini uygular
// # 📌 Modul - Rust
// # Version: 0.40.0
// # Aciklama: Resmi provider birinci, cache ikinci, config izin veriyorsa sabit katalog URL son care olacak sekilde source resolution merkezilestirir
// # Bagimli Oldugu Katman: Service | Repo | Tool

use std::time::{SystemTime, UNIX_EPOCH};

use crate::domain::guest_template::InstallerMediaMode;
use crate::domain::installer_media_source::{
    InstallerMediaResolverPolicy, InstallerMediaSourceError, InstallerMediaSourceOrigin,
    InstallerMediaSourceRequest, ResolvedInstallerMediaSource,
};
use crate::ports::installer_media_source_cache_port::InstallerMediaSourceCachePort;
use crate::ports::installer_media_source_provider_port::InstallerMediaSourceProviderPort;
use crate::ports::installer_media_source_resolver_port::InstallerMediaSourceResolverPort;

pub struct InstallerMediaSourceResolverService<P, C>
where
    P: InstallerMediaSourceProviderPort,
    C: InstallerMediaSourceCachePort,
{
    policy: InstallerMediaResolverPolicy,
    provider: P,
    cache: C,
}

impl<P, C> InstallerMediaSourceResolverService<P, C>
where
    P: InstallerMediaSourceProviderPort,
    C: InstallerMediaSourceCachePort,
{
    pub const fn new(policy: InstallerMediaResolverPolicy, provider: P, cache: C) -> Self {
        Self { policy, provider, cache }
    }

    fn append_catalog_fallback(
        &self,
        request: &InstallerMediaSourceRequest,
        source: &mut ResolvedInstallerMediaSource,
    ) {
        if !self.policy.use_catalog_fallback || request.catalog_source.mode != InstallerMediaMode::Direct {
            return;
        }
        let Some(catalog_filename) = request.catalog_source.filename.as_deref() else {
            return;
        };
        if catalog_filename != source.filename {
            return;
        }
        if !source.download_urls.iter().any(|url| url == &request.catalog_source.url) {
            source.download_urls.push(request.catalog_source.url.clone());
        }
        if let Some(checksum_url) = request.catalog_source.checksum_url.as_ref() {
            if !source.checksum_urls.iter().any(|url| url == checksum_url) {
                source.checksum_urls.push(checksum_url.clone());
            }
        }
    }

    fn catalog_fallback(
        &self,
        request: &InstallerMediaSourceRequest,
    ) -> Result<ResolvedInstallerMediaSource, InstallerMediaSourceError> {
        let media = &request.catalog_source;
        if media.mode != InstallerMediaMode::Direct {
            return Err(InstallerMediaSourceError::Unavailable(String::from(
                "installer media direct download degil",
            )));
        }
        let filename = media.filename.clone().ok_or_else(|| {
            InstallerMediaSourceError::Unavailable(String::from("installer media filename eksik"))
        })?;
        Ok(ResolvedInstallerMediaSource {
            cache_key: request.cache_key(),
            provider: request.provider.clone(),
            media_kind: request.media_kind,
            architecture: request.architecture.clone(),
            filename,
            download_urls: vec![media.url.clone()],
            checksum_urls: media.checksum_url.clone().into_iter().collect(),
            size_bytes: media.size_bytes,
            origin: InstallerMediaSourceOrigin::CatalogFallback,
            resolved_at_unix: now_unix(),
        })
    }
}

impl<P, C> InstallerMediaSourceResolverPort for InstallerMediaSourceResolverService<P, C>
where
    P: InstallerMediaSourceProviderPort,
    C: InstallerMediaSourceCachePort,
{
    fn resolve(
        &mut self,
        request: &InstallerMediaSourceRequest,
    ) -> Result<ResolvedInstallerMediaSource, InstallerMediaSourceError> {
        let Some(provider_policy) = self.policy.providers.get(&request.provider) else {
            return if self.policy.use_catalog_fallback {
                self.catalog_fallback(request)
            } else {
                Err(InstallerMediaSourceError::Policy(format!(
                    "{} icin installer media provider policy tanimli degil",
                    request.provider
                )))
            };
        };

        match self.provider.resolve(request, provider_policy) {
            Ok(mut source) => {
                source.origin = InstallerMediaSourceOrigin::OnlineDiscovery;
                self.append_catalog_fallback(request, &mut source);
                let _ = self.cache.save(&source);
                Ok(source)
            }
            Err(provider_error) => match self.cache.load(&request.cache_key()) {
                Ok(Some(mut source)) if source.matches_request(request) => {
                    source.origin = InstallerMediaSourceOrigin::LastKnownGoodCache;
                    self.append_catalog_fallback(request, &mut source);
                    Ok(source)
                }
                Ok(_) | Err(_) if self.policy.use_catalog_fallback => self.catalog_fallback(request),
                Ok(_) => Err(InstallerMediaSourceError::Unavailable(format!(
                    "online installer source discovery basarisiz ve uygun cache yok: {provider_error:?}"
                ))),
                Err(cache_error) => Err(InstallerMediaSourceError::Unavailable(format!(
                    "online installer source discovery basarisiz; cache de okunamadi: provider={provider_error:?} cache={cache_error:?}"
                ))),
            },
        }
    }
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}
