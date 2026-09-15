// # 📄 Dosya Yolu: /turkuazvm/crates/gaming-input/src/ports/game_input_profile_repository_port.rs
// # 📌 Amac: Gaming Input profile persistence contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: GameInputProfile storage islemlerini concrete YAML adapterinden ayirir
// # Bagimli Oldugu Katman: Service | Repo

use crate::domain::profile::{GameInputProfile, GamingVmId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameInputProfileRepositoryError {
    NotFound(GamingVmId),
    Storage(String),
}

pub trait GameInputProfileRepositoryPort {
    fn get(&self, vm_id: &GamingVmId) -> Result<GameInputProfile, GameInputProfileRepositoryError>;
    fn save(&mut self, profile: GameInputProfile) -> Result<(), GameInputProfileRepositoryError>;
    fn delete(&mut self, vm_id: &GamingVmId) -> Result<(), GameInputProfileRepositoryError>;
}
