// # 📄 Dosya Yolu: /turkuazvm/crates/android-image/src/ports/android_distribution_source_cache_port.rs
// # 📌 Amac: Android resolved source last-known-good cache persistence contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.40.0
// # Aciklama: Resolver Service ile YAML cache Repository arasindaki persistence sinirini kurar
// # Bagimli Oldugu Katman: Service | Repo

use crate::domain::distribution_source::{AndroidDistributionSource, AndroidDistributionSourceError};

pub trait AndroidDistributionSourceCachePort: Send {
    fn load(&self, cache_key: &str) -> Result<Option<AndroidDistributionSource>, AndroidDistributionSourceError>;
    fn save(&self, source: &AndroidDistributionSource) -> Result<(), AndroidDistributionSourceError>;
}
