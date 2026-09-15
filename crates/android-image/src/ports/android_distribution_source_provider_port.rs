// # 📄 Dosya Yolu: /turkuazvm/crates/android-image/src/ports/android_distribution_source_provider_port.rs
// # 📌 Amac: Android resmi dagitim kaynagindan concrete build ve artifact URL bilgisi kesfeden Tool contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.40.0
// # Aciklama: Service katmanini Android CI HTTP ayrintilarindan ayirir
// # Bagimli Oldugu Katman: Service | Tool

use crate::domain::distribution_source::{
    AndroidDistributionSource, AndroidDistributionSourceError, AndroidDistributionSourceRequest,
};

pub trait AndroidDistributionSourceProviderPort: Send {
    fn resolve(
        &self,
        request: &AndroidDistributionSourceRequest,
    ) -> Result<AndroidDistributionSource, AndroidDistributionSourceError>;
}
