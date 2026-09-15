// # 📄 Dosya Yolu: /turkuazvm/crates/game-catalog/src/domain/compatibility.rs
// # 📌 Amac: Oyun compatibility degerlendirme girdisi ve sonucunu tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Android SDK/ABI, input ve GPU capability bilgilerini katalog maturity seviyesinden ayri degerlendirir
// # Bagimli Oldugu Katman: Service | View

use crate::domain::game::{CatalogGpuBackend, CatalogMaturity};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameRuntimeContext {
    pub sdk_level: Option<u32>,
    pub abi: Option<String>,
    pub persistent_multi_touch: bool,
    pub relative_mouse_look: bool,
    pub gpu_backend: CatalogGpuBackend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityStatus { Blocked, Experimental, Playable, Recommended }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilityReport {
    pub status: CompatibilityStatus,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
}

impl CompatibilityReport {
    pub fn from_maturity(maturity: CatalogMaturity, blockers: Vec<String>, warnings: Vec<String>) -> Self {
        let status = if !blockers.is_empty() {
            CompatibilityStatus::Blocked
        } else {
            match maturity {
                CatalogMaturity::Experimental => CompatibilityStatus::Experimental,
                CatalogMaturity::Playable => CompatibilityStatus::Playable,
                CatalogMaturity::Recommended => CompatibilityStatus::Recommended,
            }
        };
        Self { status, blockers, warnings }
    }
}
