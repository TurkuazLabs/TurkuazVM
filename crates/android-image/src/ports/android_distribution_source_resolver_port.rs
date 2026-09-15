// # 📄 Dosya Yolu: /turkuazvm/crates/android-image/src/ports/android_distribution_source_resolver_port.rs
// # 📌 Amac: Android Image Distribution Tool icin provider/cache orkestrasyonunu typed resolver contracti arkasina alir
// # 📌 Modul - Rust
// # Version: 0.40.0
// # Aciklama: Distribution Tool'un release policy, online discovery ve last-known-good cache ayrintilarini bilmesini engeller
// # Bagimli Oldugu Katman: Service | Tool

use crate::domain::distribution_source::{AndroidDistributionSource, AndroidDistributionSourceError};
use crate::domain::image::AndroidImage;

pub trait AndroidDistributionSourceResolverPort: Send {
    fn resolve(&mut self, image: &AndroidImage) -> Result<AndroidDistributionSource, AndroidDistributionSourceError>;
}
