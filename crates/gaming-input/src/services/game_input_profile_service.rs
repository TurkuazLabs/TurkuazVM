// # 📄 Dosya Yolu: /turkuazvm/crates/gaming-input/src/services/game_input_profile_service.rs
// # 📌 Amac: Gaming Input profile persistence use-case is kurallarini yonetir
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Typed profile domain validation sonrasi repository get/save/delete akislarini orkestre eder
// # Bagimli Oldugu Katman: Repo

use crate::domain::profile::{GameInputProfile, GamingInputProfileError, GamingVmId};
use crate::ports::game_input_profile_repository_port::{
    GameInputProfileRepositoryError, GameInputProfileRepositoryPort,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameInputProfileServiceError {
    Domain(GamingInputProfileError),
    Repository(GameInputProfileRepositoryError),
}

pub struct GameInputProfileService<R>
where
    R: GameInputProfileRepositoryPort,
{
    repository: R,
}

impl<R> GameInputProfileService<R>
where
    R: GameInputProfileRepositoryPort,
{
    pub const fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn save(&mut self, profile: GameInputProfile) -> Result<GameInputProfile, GameInputProfileServiceError> {
        self.repository
            .save(profile.clone())
            .map_err(GameInputProfileServiceError::Repository)?;
        Ok(profile)
    }

    pub fn get(&self, vm_id: &str) -> Result<GameInputProfile, GameInputProfileServiceError> {
        let vm_id = GamingVmId::parse(vm_id.to_owned()).map_err(GameInputProfileServiceError::Domain)?;
        self.repository.get(&vm_id).map_err(GameInputProfileServiceError::Repository)
    }

    pub fn delete(&mut self, vm_id: &str) -> Result<(), GameInputProfileServiceError> {
        let vm_id = GamingVmId::parse(vm_id.to_owned()).map_err(GameInputProfileServiceError::Domain)?;
        self.repository.delete(&vm_id).map_err(GameInputProfileServiceError::Repository)
    }
}
