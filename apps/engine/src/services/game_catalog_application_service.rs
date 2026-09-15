// # 📄 Dosya Yolu: /turkuazvm/apps/engine/src/services/game_catalog_application_service.rs
// # 📌 Amac: Game Catalog bounded contextini Engine seviyesinde orkestre eder
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: YAML catalog sorgu, installed package eslestirme ve compatibility degerlendirme use-case'lerini sunar
// # Bagimli Oldugu Katman: Service | Repo

use std::path::PathBuf;
use turkuazvm_game_catalog::domain::compatibility::{CompatibilityReport, GameRuntimeContext};
use turkuazvm_game_catalog::domain::game::GameDefinition;
use turkuazvm_game_catalog::services::game_catalog_service::GameCatalogService;
use turkuazvm_repositories::repositories::yaml_game_catalog_repository::YamlGameCatalogRepository;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameCatalogApplicationError { Catalog(String) }

type CatalogService = GameCatalogService<YamlGameCatalogRepository>;

pub struct GameCatalogApplicationService { service: CatalogService }

impl GameCatalogApplicationService {
    pub fn new(path: PathBuf) -> Self { Self { service: GameCatalogService::new(YamlGameCatalogRepository::new(path)) } }
    pub fn list(&self) -> Result<Vec<GameDefinition>, GameCatalogApplicationError> { self.service.list().map_err(map_error) }
    pub fn get(&self, game_id: &str) -> Result<GameDefinition, GameCatalogApplicationError> { self.service.get(game_id).map_err(map_error) }
    pub fn detect(&self, packages: &[String]) -> Result<Vec<GameDefinition>, GameCatalogApplicationError> { self.service.detect(packages).map_err(map_error) }
    pub fn evaluate(&self, game: &GameDefinition, context: &GameRuntimeContext) -> CompatibilityReport { self.service.evaluate(game, context) }
}

fn map_error(error: impl std::fmt::Debug) -> GameCatalogApplicationError { GameCatalogApplicationError::Catalog(format!("{error:?}")) }
