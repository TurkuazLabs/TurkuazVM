// # 📄 Dosya Yolu: /turkuazvm/crates/game-catalog/src/ports/game_catalog_repository_port.rs
// # 📌 Amac: Oyun katalogu persistence adapter contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: YAML veya gelecekte remote catalog implementasyonlarini domain servisinden ayirir
// # Bagimli Oldugu Katman: Repo | Tool

use crate::domain::game::{GameDefinition, GameId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameCatalogRepositoryError { NotFound(GameId), Storage(String) }

pub trait GameCatalogRepositoryPort {
    fn list(&self) -> Result<Vec<GameDefinition>, GameCatalogRepositoryError>;
    fn get(&self, id: &GameId) -> Result<GameDefinition, GameCatalogRepositoryError>;
}
