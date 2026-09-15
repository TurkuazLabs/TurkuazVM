// # 📄 Dosya Yolu: /turkuazvm/crates/repositories/src/repositories/yaml_game_catalog_repository.rs
// # 📌 Amac: Oyun katalogunu versioned YAML dosyasindan yukler
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Game Catalog persistence portunu serde/YAML adapteri ile uygular ve magic string oyun bilgisini uygulama kodundan uzak tutar
// # Bagimli Oldugu Katman: Repo

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use serde::Deserialize;
use turkuazvm_game_catalog::domain::game::{
    CatalogGpuBackend, CatalogInputBinding, CatalogInputPreset, CatalogInputSource, CatalogInputTarget, CatalogJoystickPreset,
    CatalogKey, CatalogMaturity, CatalogMouseButton, CatalogMouseLookPreset, CatalogPoint, GameDefinition, GameId, GameRequirements,
    PackageName, RecommendedAndroidRuntime,
};
use turkuazvm_game_catalog::ports::game_catalog_repository_port::{GameCatalogRepositoryError, GameCatalogRepositoryPort};

const CATALOG_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone)]
pub struct YamlGameCatalogRepository { path: PathBuf }
impl YamlGameCatalogRepository { pub fn new(path: PathBuf) -> Self { Self { path } } }

#[derive(Debug, Deserialize)]
struct CatalogManifest { schema_version: u16, games: Vec<GameManifest> }
#[derive(Debug, Deserialize)]
struct GameManifest {
    id: String, name: String, packages: Vec<String>, maturity: String,
    #[serde(default = "default_true")] emulator_disclosure_required: bool,
    requirements: RequirementsManifest, android: AndroidManifest, input: InputManifest,
}
#[derive(Debug, Deserialize)]
struct RequirementsManifest {
    minimum_sdk: Option<u32>, #[serde(default)] accepted_abis: Vec<String>,
    persistent_multi_touch: bool, relative_mouse_look: bool,
    #[serde(default)] preferred_gpu_backends: Vec<CatalogGpuBackendManifest>,
}
#[derive(Debug, Deserialize)] #[serde(rename_all = "snake_case")]
enum CatalogGpuBackendManifest { Software, Virtio2d, VirglVenus, Gfxstream }
#[derive(Debug, Deserialize)] struct AndroidManifest { width: u32, height: u32, density_dpi: u32, target_fps: u16 }
#[derive(Debug, Deserialize)] struct InputManifest { joystick: JoystickManifest, mouse_look: MouseLookManifest, #[serde(default)] bindings: Vec<BindingManifest> }
#[derive(Debug, Deserialize)] struct JoystickManifest { enabled: bool, center: PointManifest, radius: u16, pointer_id: u8 }
#[derive(Debug, Deserialize)] struct MouseLookManifest { enabled: bool, anchor: PointManifest, sensitivity_x_milli: u16, sensitivity_y_milli: u16, pointer_id: u8 }
#[derive(Debug, Deserialize)] struct PointManifest { x: u16, y: u16 }
#[derive(Debug, Deserialize)] struct BindingManifest { source: SourceManifest, target: TargetManifest }
#[derive(Debug, Deserialize)] #[serde(tag = "type", rename_all = "snake_case")]
enum SourceManifest { Key { key: CatalogKeyManifest }, MouseButton { button: CatalogMouseButtonManifest }, GamepadButton { button: u16 } }
#[derive(Debug, Deserialize)] #[serde(rename_all = "snake_case")]
enum CatalogKeyManifest { W, A, S, D, Space, C, Z, R, F, Q, E, ShiftLeft, ControlLeft, Digit1, Digit2, Digit3, Digit4, Tab, Escape }
#[derive(Debug, Deserialize)] #[serde(rename_all = "snake_case")]
enum CatalogMouseButtonManifest { Left, Right, Middle }
#[derive(Debug, Deserialize)] #[serde(tag = "type", rename_all = "snake_case")]
enum TargetManifest { Tap { position: PointManifest }, HoldTouch { pointer_id: u8, position: PointManifest }, AndroidKey { key_code: u32 } }

impl GameCatalogRepositoryPort for YamlGameCatalogRepository {
    fn list(&self) -> Result<Vec<GameDefinition>, GameCatalogRepositoryError> {
        let content = fs::read_to_string(&self.path).map_err(storage)?;
        let manifest: CatalogManifest = serde_yaml_ng::from_str(&content).map_err(storage)?;
        if manifest.schema_version != CATALOG_SCHEMA_VERSION { return Err(GameCatalogRepositoryError::Storage(format!("Unsupported game catalog schema: {}", manifest.schema_version))); }
        let games = manifest.games.into_iter().map(to_domain).collect::<Result<Vec<_>, _>>()?;
        validate_catalog(&games)?;
        Ok(games)
    }
    fn get(&self, id: &GameId) -> Result<GameDefinition, GameCatalogRepositoryError> {
        self.list()?.into_iter().find(|game| game.id == *id).ok_or_else(|| GameCatalogRepositoryError::NotFound(id.clone()))
    }
}

fn to_domain(value: GameManifest) -> Result<GameDefinition, GameCatalogRepositoryError> {
    let maturity = match value.maturity.as_str() {
        "experimental" => CatalogMaturity::Experimental,
        "playable" => CatalogMaturity::Playable,
        "recommended" => CatalogMaturity::Recommended,
        other => return Err(GameCatalogRepositoryError::Storage(format!("Unsupported catalog maturity: {other}"))),
    };
    let packages = value.packages.into_iter().map(PackageName::parse).collect::<Result<Vec<_>, _>>().map_err(domain)?;
    GameDefinition::create(
        GameId::parse(value.id).map_err(domain)?, value.name, packages, maturity, value.emulator_disclosure_required,
        GameRequirements { minimum_sdk: value.requirements.minimum_sdk, accepted_abis: value.requirements.accepted_abis,
            persistent_multi_touch: value.requirements.persistent_multi_touch, relative_mouse_look: value.requirements.relative_mouse_look,
            preferred_gpu_backends: value.requirements.preferred_gpu_backends.into_iter().map(catalog_gpu_backend).collect() },
        RecommendedAndroidRuntime { width: value.android.width, height: value.android.height, density_dpi: value.android.density_dpi, target_fps: value.android.target_fps },
        CatalogInputPreset {
            joystick: CatalogJoystickPreset { enabled: value.input.joystick.enabled, center: point(value.input.joystick.center)?, radius: value.input.joystick.radius, pointer_id: value.input.joystick.pointer_id },
            mouse_look: CatalogMouseLookPreset { enabled: value.input.mouse_look.enabled, anchor: point(value.input.mouse_look.anchor)?, sensitivity_x_milli: value.input.mouse_look.sensitivity_x_milli, sensitivity_y_milli: value.input.mouse_look.sensitivity_y_milli, pointer_id: value.input.mouse_look.pointer_id },
            bindings: value.input.bindings.into_iter().map(binding).collect::<Result<Vec<_>, _>>()?,
        },
    ).map_err(domain)
}
fn point(value: PointManifest) -> Result<CatalogPoint, GameCatalogRepositoryError> { CatalogPoint::create(value.x, value.y).map_err(domain) }
fn binding(value: BindingManifest) -> Result<CatalogInputBinding, GameCatalogRepositoryError> {
    let source = match value.source {
        SourceManifest::Key { key } => CatalogInputSource::Key(catalog_key(key)),
        SourceManifest::MouseButton { button } => CatalogInputSource::MouseButton(catalog_mouse_button(button)),
        SourceManifest::GamepadButton { button } => CatalogInputSource::GamepadButton(button),
    };
    let target = match value.target { TargetManifest::Tap { position } => CatalogInputTarget::Tap(point(position)?), TargetManifest::HoldTouch { pointer_id, position } => CatalogInputTarget::HoldTouch { pointer_id, position: point(position)? }, TargetManifest::AndroidKey { key_code } => CatalogInputTarget::AndroidKey(key_code) };
    Ok(CatalogInputBinding { source, target })
}
fn default_true() -> bool { true }
fn storage(error: impl std::fmt::Display) -> GameCatalogRepositoryError { GameCatalogRepositoryError::Storage(error.to_string()) }
fn domain(error: impl std::fmt::Debug) -> GameCatalogRepositoryError { GameCatalogRepositoryError::Storage(format!("{error:?}")) }

fn catalog_gpu_backend(value: CatalogGpuBackendManifest) -> CatalogGpuBackend {
    match value {
        CatalogGpuBackendManifest::Software => CatalogGpuBackend::Software,
        CatalogGpuBackendManifest::Virtio2d => CatalogGpuBackend::Virtio2d,
        CatalogGpuBackendManifest::VirglVenus => CatalogGpuBackend::VirglVenus,
        CatalogGpuBackendManifest::Gfxstream => CatalogGpuBackend::Gfxstream,
    }
}
fn catalog_key(value: CatalogKeyManifest) -> CatalogKey {
    match value {
        CatalogKeyManifest::W => CatalogKey::W, CatalogKeyManifest::A => CatalogKey::A, CatalogKeyManifest::S => CatalogKey::S, CatalogKeyManifest::D => CatalogKey::D,
        CatalogKeyManifest::Space => CatalogKey::Space, CatalogKeyManifest::C => CatalogKey::C, CatalogKeyManifest::Z => CatalogKey::Z, CatalogKeyManifest::R => CatalogKey::R,
        CatalogKeyManifest::F => CatalogKey::F, CatalogKeyManifest::Q => CatalogKey::Q, CatalogKeyManifest::E => CatalogKey::E, CatalogKeyManifest::ShiftLeft => CatalogKey::ShiftLeft,
        CatalogKeyManifest::ControlLeft => CatalogKey::ControlLeft, CatalogKeyManifest::Digit1 => CatalogKey::Digit1, CatalogKeyManifest::Digit2 => CatalogKey::Digit2,
        CatalogKeyManifest::Digit3 => CatalogKey::Digit3, CatalogKeyManifest::Digit4 => CatalogKey::Digit4, CatalogKeyManifest::Tab => CatalogKey::Tab, CatalogKeyManifest::Escape => CatalogKey::Escape,
    }
}
fn catalog_mouse_button(value: CatalogMouseButtonManifest) -> CatalogMouseButton {
    match value { CatalogMouseButtonManifest::Left => CatalogMouseButton::Left, CatalogMouseButtonManifest::Right => CatalogMouseButton::Right, CatalogMouseButtonManifest::Middle => CatalogMouseButton::Middle }
}

fn validate_catalog(games: &[GameDefinition]) -> Result<(), GameCatalogRepositoryError> {
    let mut ids = HashSet::new();
    let mut packages = HashSet::new();
    for game in games {
        if !ids.insert(game.id.as_str()) {
            return Err(GameCatalogRepositoryError::Storage(format!("Duplicate game id: {}", game.id.as_str())));
        }
        for package in &game.packages {
            if !packages.insert(package.as_str()) {
                return Err(GameCatalogRepositoryError::Storage(format!("Duplicate game package: {}", package.as_str())));
            }
        }
    }
    Ok(())
}
