// # 📄 Dosya Yolu: /turkuazvm/crates/game-catalog/src/domain/game.rs
// # 📌 Amac: Oyun katalog kaydi, package kimligi ve onerilen runtime/input preset modellerini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Oyun uyumluluk metadata ve normalized input presetlerini framework bagimsiz typed domain olarak tutar
// # Bagimli Oldugu Katman: Service | Repo | View

const MAX_ID_LEN: usize = 80;
const MAX_NAME_LEN: usize = 120;
const MAX_PACKAGE_LEN: usize = 255;
const NORMALIZED_MAX: u16 = 10_000;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameId(String);

impl GameId {
    pub fn parse(value: impl Into<String>) -> Result<Self, GameCatalogError> {
        let value = value.into();
        let valid = !value.is_empty() && value.len() <= MAX_ID_LEN && value.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'));
        valid.then_some(Self(value)).ok_or(GameCatalogError::InvalidGameId)
    }
    pub fn as_str(&self) -> &str { &self.0 }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageName(String);

impl PackageName {
    pub fn parse(value: impl Into<String>) -> Result<Self, GameCatalogError> {
        let value = value.into();
        let trimmed = value.trim();
        let valid = !trimmed.is_empty() && trimmed.len() <= MAX_PACKAGE_LEN && trimmed.contains('.')
            && trimmed.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_'));
        if !valid { return Err(GameCatalogError::InvalidPackageName); }
        Ok(Self(trimmed.to_owned()))
    }
    pub fn as_str(&self) -> &str { &self.0 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CatalogPoint { pub x: u16, pub y: u16 }

impl CatalogPoint {
    pub fn create(x: u16, y: u16) -> Result<Self, GameCatalogError> {
        if x > NORMALIZED_MAX || y > NORMALIZED_MAX { return Err(GameCatalogError::InvalidCoordinate); }
        Ok(Self { x, y })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogJoystickPreset {
    pub enabled: bool,
    pub center: CatalogPoint,
    pub radius: u16,
    pub pointer_id: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogMouseLookPreset {
    pub enabled: bool,
    pub anchor: CatalogPoint,
    pub sensitivity_x_milli: u16,
    pub sensitivity_y_milli: u16,
    pub pointer_id: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CatalogKey {
    W, A, S, D, Space, C, Z, R, F, Q, E, ShiftLeft, ControlLeft,
    Digit1, Digit2, Digit3, Digit4, Tab, Escape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CatalogMouseButton { Left, Right, Middle }

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CatalogInputSource {
    Key(CatalogKey),
    MouseButton(CatalogMouseButton),
    GamepadButton(u16),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogInputTarget {
    Tap(CatalogPoint),
    HoldTouch { pointer_id: u8, position: CatalogPoint },
    AndroidKey(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogInputBinding {
    pub source: CatalogInputSource,
    pub target: CatalogInputTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogInputPreset {
    pub joystick: CatalogJoystickPreset,
    pub mouse_look: CatalogMouseLookPreset,
    pub bindings: Vec<CatalogInputBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecommendedAndroidRuntime {
    pub width: u32,
    pub height: u32,
    pub density_dpi: u32,
    pub target_fps: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CatalogGpuBackend { Software, Virtio2d, VirglVenus, Gfxstream }

impl CatalogGpuBackend {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Software => "software",
            Self::Virtio2d => "virtio_2d",
            Self::VirglVenus => "virgl_venus",
            Self::Gfxstream => "gfxstream",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameRequirements {
    pub minimum_sdk: Option<u32>,
    pub accepted_abis: Vec<String>,
    pub persistent_multi_touch: bool,
    pub relative_mouse_look: bool,
    pub preferred_gpu_backends: Vec<CatalogGpuBackend>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogMaturity { Experimental, Playable, Recommended }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameDefinition {
    pub id: GameId,
    pub name: String,
    pub packages: Vec<PackageName>,
    pub maturity: CatalogMaturity,
    pub emulator_disclosure_required: bool,
    pub requirements: GameRequirements,
    pub android: RecommendedAndroidRuntime,
    pub input: CatalogInputPreset,
}

impl GameDefinition {
    pub fn create(
        id: GameId,
        name: String,
        packages: Vec<PackageName>,
        maturity: CatalogMaturity,
        emulator_disclosure_required: bool,
        requirements: GameRequirements,
        android: RecommendedAndroidRuntime,
        input: CatalogInputPreset,
    ) -> Result<Self, GameCatalogError> {
        if name.trim().is_empty() || name.len() > MAX_NAME_LEN { return Err(GameCatalogError::InvalidName); }
        if packages.is_empty() { return Err(GameCatalogError::MissingPackage); }
        if input.joystick.radius == 0 || input.joystick.radius > 5_000 { return Err(GameCatalogError::InvalidJoystick); }
        if input.mouse_look.sensitivity_x_milli == 0 || input.mouse_look.sensitivity_y_milli == 0 { return Err(GameCatalogError::InvalidMouseLook); }
        if android.width == 0 || android.height == 0 || android.density_dpi == 0 || android.target_fps == 0 { return Err(GameCatalogError::InvalidAndroidRuntime); }
        if input.joystick.pointer_id == input.mouse_look.pointer_id { return Err(GameCatalogError::PointerIdCollision); }
        let mut sources = std::collections::HashSet::new();
        let mut hold_pointers = std::collections::HashSet::new();
        for binding in &input.bindings {
            if !sources.insert(binding.source.clone()) { return Err(GameCatalogError::DuplicateBindingSource); }
            if let CatalogInputTarget::HoldTouch { pointer_id, .. } = &binding.target {
                if *pointer_id == input.joystick.pointer_id || *pointer_id == input.mouse_look.pointer_id || !hold_pointers.insert(*pointer_id) {
                    return Err(GameCatalogError::PointerIdCollision);
                }
            }
        }
        Ok(Self { id, name: name.trim().to_owned(), packages, maturity, emulator_disclosure_required, requirements, android, input })
    }

    pub fn matches_package(&self, package_name: &str) -> bool {
        self.packages.iter().any(|value| value.as_str() == package_name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameCatalogError {
    InvalidGameId,
    InvalidName,
    InvalidPackageName,
    MissingPackage,
    InvalidCoordinate,
    InvalidJoystick,
    InvalidMouseLook,
    InvalidAndroidRuntime,
    DuplicateBindingSource,
    PointerIdCollision,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn runtime() -> RecommendedAndroidRuntime {
        RecommendedAndroidRuntime { width: 1920, height: 1080, density_dpi: 320, target_fps: 60 }
    }

    fn preset(joystick_pointer: u8, mouse_pointer: u8) -> CatalogInputPreset {
        CatalogInputPreset {
            joystick: CatalogJoystickPreset {
                enabled: true,
                center: CatalogPoint::create(1_800, 8_000).expect("center"),
                radius: 1_000,
                pointer_id: joystick_pointer,
            },
            mouse_look: CatalogMouseLookPreset {
                enabled: true,
                anchor: CatalogPoint::create(7_000, 5_000).expect("anchor"),
                sensitivity_x_milli: 1_000,
                sensitivity_y_milli: 1_000,
                pointer_id: mouse_pointer,
            },
            bindings: Vec::new(),
        }
    }

    #[test]
    fn joystick_and_mouse_pointer_collision_is_rejected() {
        let game = GameDefinition::create(
            GameId::parse("test-game").expect("id"),
            String::from("Test Game"),
            vec![PackageName::parse("com.example.game").expect("package")],
            CatalogMaturity::Experimental,
            true,
            GameRequirements {
                minimum_sdk: Some(26),
                accepted_abis: Vec::new(),
                persistent_multi_touch: true,
                relative_mouse_look: true,
                preferred_gpu_backends: Vec::new(),
            },
            runtime(),
            preset(1, 1),
        );
        assert_eq!(game, Err(GameCatalogError::PointerIdCollision));
    }
}
