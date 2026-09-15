// # 📄 Dosya Yolu: /turkuazvm/crates/game-catalog/src/services/game_catalog_service.rs
// # 📌 Amac: Oyun katalogu eslestirme ve compatibility is kurallarini uygular
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Installed package eslestirmesi ile capability blocker/warning degerlendirmesini repository adapterinden ayirir
// # Bagimli Oldugu Katman: Repo

use std::collections::HashSet;
use crate::domain::compatibility::{CompatibilityReport, GameRuntimeContext};
use crate::domain::game::{GameDefinition, GameId};
use crate::ports::game_catalog_repository_port::{GameCatalogRepositoryError, GameCatalogRepositoryPort};

pub struct GameCatalogService<R: GameCatalogRepositoryPort> { repository: R }

impl<R: GameCatalogRepositoryPort> GameCatalogService<R> {
    pub const fn new(repository: R) -> Self { Self { repository } }
    pub fn list(&self) -> Result<Vec<GameDefinition>, GameCatalogRepositoryError> { self.repository.list() }
    pub fn get(&self, game_id: &str) -> Result<GameDefinition, GameCatalogRepositoryError> {
        let id = GameId::parse(game_id.to_owned()).map_err(|error| GameCatalogRepositoryError::Storage(format!("{error:?}")))?;
        self.repository.get(&id)
    }
    pub fn detect(&self, installed_packages: &[String]) -> Result<Vec<GameDefinition>, GameCatalogRepositoryError> {
        let installed = installed_packages.iter().map(String::as_str).collect::<HashSet<_>>();
        Ok(self.repository.list()?.into_iter().filter(|game| game.packages.iter().any(|package| installed.contains(package.as_str()))).collect())
    }
    pub fn evaluate(&self, game: &GameDefinition, context: &GameRuntimeContext) -> CompatibilityReport {
        let mut blockers = Vec::new();
        let mut warnings = Vec::new();
        if let (Some(minimum), Some(actual)) = (game.requirements.minimum_sdk, context.sdk_level) {
            if actual < minimum { blockers.push(format!("android_sdk_below_minimum:{actual}<{minimum}")); }
        } else if game.requirements.minimum_sdk.is_some() && context.sdk_level.is_none() {
            warnings.push(String::from("android_sdk_unknown"));
        }
        if !game.requirements.accepted_abis.is_empty() {
            match context.abi.as_deref() {
                Some(abi) if !game.requirements.accepted_abis.iter().any(|accepted| accepted == abi) => blockers.push(format!("android_abi_unsupported:{abi}")),
                None => warnings.push(String::from("android_abi_unknown")),
                _ => {}
            }
        }
        if game.requirements.persistent_multi_touch && !context.persistent_multi_touch { blockers.push(String::from("persistent_multi_touch_required")); }
        if game.requirements.relative_mouse_look && !context.relative_mouse_look { blockers.push(String::from("relative_mouse_look_required")); }
        if !game.requirements.preferred_gpu_backends.is_empty() && !game.requirements.preferred_gpu_backends.contains(&context.gpu_backend) {
            warnings.push(format!("gpu_backend_not_preferred:{}", context.gpu_backend.code()));
        }
        if game.emulator_disclosure_required { warnings.push(String::from("emulator_identity_must_remain_disclosed")); }
        CompatibilityReport::from_maturity(game.maturity, blockers, warnings)
    }
}
