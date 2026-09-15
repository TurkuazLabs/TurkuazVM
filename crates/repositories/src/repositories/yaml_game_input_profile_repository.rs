// # 📄 Dosya Yolu: /turkuazvm/crates/repositories/src/repositories/yaml_game_input_profile_repository.rs
// # 📌 Amac: Gaming Input profile bilgisini VM klasorundeki gaming-input.yml dosyasinda saklar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Gaming Input bounded context persistence portunu serde/YAML adapteri ile uygular
// # Bagimli Oldugu Katman: Repo

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use turkuazvm_gaming_input::domain::profile::{
    GameInputProfile, GamingKey, GamingMouseButton, GamingVmId, InputBinding, InputSource,
    InputTarget, MouseLookProfile, NormalizedPoint, VirtualJoystickProfile,
};
use turkuazvm_gaming_input::ports::game_input_profile_repository_port::{
    GameInputProfileRepositoryError, GameInputProfileRepositoryPort,
};

const PROFILE_SCHEMA_VERSION: u16 = 1;
const DIR_MACHINES: &str = "machines";
const FILE_GAME_INPUT_PROFILE: &str = "gaming-input.yml";

#[derive(Debug, Clone)]
pub struct YamlGameInputProfileRepository {
    data_root: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct GameInputManifest {
    schema_version: u16,
    vm_id: String,
    package_name: Option<String>,
    joystick: JoystickManifest,
    mouse_look: MouseLookManifest,
    bindings: Vec<BindingManifest>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PointManifest {
    x: u16,
    y: u16,
}

#[derive(Debug, Serialize, Deserialize)]
struct JoystickManifest {
    enabled: bool,
    center: PointManifest,
    radius: u16,
    pointer_id: u8,
}

#[derive(Debug, Serialize, Deserialize)]
struct MouseLookManifest {
    enabled: bool,
    anchor: PointManifest,
    sensitivity_x_milli: u16,
    sensitivity_y_milli: u16,
    pointer_id: u8,
}

#[derive(Debug, Serialize, Deserialize)]
struct BindingManifest {
    source: SourceManifest,
    target: TargetManifest,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum SourceManifest {
    Key { key: String },
    MouseButton { button: String },
    GamepadButton { button: u16 },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum TargetManifest {
    Tap { position: PointManifest },
    HoldTouch { pointer_id: u8, position: PointManifest },
    AndroidKey { key_code: u32 },
}

impl YamlGameInputProfileRepository {
    pub fn new(data_root: PathBuf) -> Self {
        Self { data_root }
    }

    fn machine_dir(&self, vm_id: &GamingVmId) -> PathBuf {
        self.data_root.join(DIR_MACHINES).join(vm_id.as_str())
    }

    fn profile_path(&self, vm_id: &GamingVmId) -> PathBuf {
        self.machine_dir(vm_id).join(FILE_GAME_INPUT_PROFILE)
    }

    fn manifest_header(path: &Path) -> String {
        format!(
            "# \u{1F4C4} Dosya Yolu: {}\n# \u{1F4CC} Amac: Bu Android gaming VM icin keymapping ve input profilini saklar\n# \u{1F4CC} Modul - YAML\n# Version: {}\n# Aciklama: Normalized joystick, mouse look ve binding konfigurasyonunu kalici tutar\n# Bagimli Oldugu Katman: Repo\n\n",
            path.display(),
            env!("CARGO_PKG_VERSION")
        )
    }

    fn read_profile(&self, path: &Path) -> Result<GameInputProfile, GameInputProfileRepositoryError> {
        let content = fs::read_to_string(path)
            .map_err(|error| GameInputProfileRepositoryError::Storage(error.to_string()))?;
        let manifest: GameInputManifest = serde_yaml_ng::from_str(&content)
            .map_err(|error| GameInputProfileRepositoryError::Storage(error.to_string()))?;
        if manifest.schema_version != PROFILE_SCHEMA_VERSION {
            return Err(GameInputProfileRepositoryError::Storage(format!(
                "Unsupported gaming input profile schema: {}",
                manifest.schema_version
            )));
        }
        manifest_to_domain(manifest)
    }
}

impl GameInputProfileRepositoryPort for YamlGameInputProfileRepository {
    fn get(&self, vm_id: &GamingVmId) -> Result<GameInputProfile, GameInputProfileRepositoryError> {
        let path = self.profile_path(vm_id);
        if !path.is_file() {
            return Err(GameInputProfileRepositoryError::NotFound(vm_id.clone()));
        }
        self.read_profile(&path)
    }

    fn save(&mut self, profile: GameInputProfile) -> Result<(), GameInputProfileRepositoryError> {
        let directory = self.machine_dir(profile.vm_id());
        fs::create_dir_all(&directory)
            .map_err(|error| GameInputProfileRepositoryError::Storage(error.to_string()))?;
        let path = self.profile_path(profile.vm_id());
        let manifest = domain_to_manifest(&profile);
        let yaml = serde_yaml_ng::to_string(&manifest)
            .map_err(|error| GameInputProfileRepositoryError::Storage(error.to_string()))?;
        fs::write(&path, format!("{}{}", Self::manifest_header(&path), yaml))
            .map_err(|error| GameInputProfileRepositoryError::Storage(error.to_string()))
    }

    fn delete(&mut self, vm_id: &GamingVmId) -> Result<(), GameInputProfileRepositoryError> {
        let path = self.profile_path(vm_id);
        if path.exists() {
            fs::remove_file(path)
                .map_err(|error| GameInputProfileRepositoryError::Storage(error.to_string()))?;
        }
        Ok(())
    }
}

fn manifest_to_domain(manifest: GameInputManifest) -> Result<GameInputProfile, GameInputProfileRepositoryError> {
    let vm_id = GamingVmId::parse(manifest.vm_id).map_err(domain_error)?;
    let joystick = VirtualJoystickProfile::create(
        manifest.joystick.enabled,
        point_from_manifest(manifest.joystick.center)?,
        manifest.joystick.radius,
        manifest.joystick.pointer_id,
    )
    .map_err(domain_error)?;
    let mouse_look = MouseLookProfile::create(
        manifest.mouse_look.enabled,
        point_from_manifest(manifest.mouse_look.anchor)?,
        manifest.mouse_look.sensitivity_x_milli,
        manifest.mouse_look.sensitivity_y_milli,
        manifest.mouse_look.pointer_id,
    )
    .map_err(domain_error)?;
    let bindings = manifest
        .bindings
        .into_iter()
        .map(binding_from_manifest)
        .collect::<Result<Vec<_>, _>>()?;
    GameInputProfile::create(vm_id, manifest.package_name, joystick, mouse_look, bindings)
        .map_err(domain_error)
}

fn domain_to_manifest(profile: &GameInputProfile) -> GameInputManifest {
    GameInputManifest {
        schema_version: PROFILE_SCHEMA_VERSION,
        vm_id: profile.vm_id().as_str().to_owned(),
        package_name: profile.package_name().map(str::to_owned),
        joystick: JoystickManifest {
            enabled: profile.joystick().enabled(),
            center: point_to_manifest(profile.joystick().center()),
            radius: profile.joystick().radius(),
            pointer_id: profile.joystick().pointer_id(),
        },
        mouse_look: MouseLookManifest {
            enabled: profile.mouse_look().enabled(),
            anchor: point_to_manifest(profile.mouse_look().anchor()),
            sensitivity_x_milli: profile.mouse_look().sensitivity_x_milli(),
            sensitivity_y_milli: profile.mouse_look().sensitivity_y_milli(),
            pointer_id: profile.mouse_look().pointer_id(),
        },
        bindings: profile.bindings().iter().map(binding_to_manifest).collect(),
    }
}

fn point_from_manifest(point: PointManifest) -> Result<NormalizedPoint, GameInputProfileRepositoryError> {
    NormalizedPoint::create(point.x, point.y).map_err(domain_error)
}

fn point_to_manifest(point: NormalizedPoint) -> PointManifest {
    PointManifest { x: point.x(), y: point.y() }
}

fn binding_from_manifest(binding: BindingManifest) -> Result<InputBinding, GameInputProfileRepositoryError> {
    let source = match binding.source {
        SourceManifest::Key { key } => InputSource::Key(parse_key(&key)?),
        SourceManifest::MouseButton { button } => InputSource::MouseButton(parse_mouse_button(&button)?),
        SourceManifest::GamepadButton { button } => InputSource::GamepadButton(button),
    };
    let target = match binding.target {
        TargetManifest::Tap { position } => InputTarget::Tap { position: point_from_manifest(position)? },
        TargetManifest::HoldTouch { pointer_id, position } => InputTarget::HoldTouch {
            pointer_id,
            position: point_from_manifest(position)?,
        },
        TargetManifest::AndroidKey { key_code } => InputTarget::AndroidKey { key_code },
    };
    Ok(InputBinding::new(source, target))
}

fn binding_to_manifest(binding: &InputBinding) -> BindingManifest {
    let source = match binding.source() {
        InputSource::Key(key) => SourceManifest::Key { key: key_name(key).to_owned() },
        InputSource::MouseButton(button) => SourceManifest::MouseButton { button: mouse_button_name(button).to_owned() },
        InputSource::GamepadButton(button) => SourceManifest::GamepadButton { button },
    };
    let target = match binding.target() {
        InputTarget::Tap { position } => TargetManifest::Tap { position: point_to_manifest(*position) },
        InputTarget::HoldTouch { pointer_id, position } => TargetManifest::HoldTouch {
            pointer_id: *pointer_id,
            position: point_to_manifest(*position),
        },
        InputTarget::AndroidKey { key_code } => TargetManifest::AndroidKey { key_code: *key_code },
    };
    BindingManifest { source, target }
}

fn parse_key(value: &str) -> Result<GamingKey, GameInputProfileRepositoryError> {
    match value {
        "w" => Ok(GamingKey::W), "a" => Ok(GamingKey::A), "s" => Ok(GamingKey::S), "d" => Ok(GamingKey::D),
        "space" => Ok(GamingKey::Space), "c" => Ok(GamingKey::C), "z" => Ok(GamingKey::Z), "r" => Ok(GamingKey::R),
        "f" => Ok(GamingKey::F), "q" => Ok(GamingKey::Q), "e" => Ok(GamingKey::E), "shift_left" => Ok(GamingKey::ShiftLeft),
        "control_left" => Ok(GamingKey::ControlLeft), "digit1" => Ok(GamingKey::Digit1), "digit2" => Ok(GamingKey::Digit2),
        "digit3" => Ok(GamingKey::Digit3), "digit4" => Ok(GamingKey::Digit4), "tab" => Ok(GamingKey::Tab),
        "escape" => Ok(GamingKey::Escape),
        other => Err(GameInputProfileRepositoryError::Storage(format!("Unsupported gaming key: {other}"))),
    }
}

fn key_name(value: GamingKey) -> &'static str {
    match value {
        GamingKey::W => "w", GamingKey::A => "a", GamingKey::S => "s", GamingKey::D => "d",
        GamingKey::Space => "space", GamingKey::C => "c", GamingKey::Z => "z", GamingKey::R => "r",
        GamingKey::F => "f", GamingKey::Q => "q", GamingKey::E => "e", GamingKey::ShiftLeft => "shift_left",
        GamingKey::ControlLeft => "control_left", GamingKey::Digit1 => "digit1", GamingKey::Digit2 => "digit2",
        GamingKey::Digit3 => "digit3", GamingKey::Digit4 => "digit4", GamingKey::Tab => "tab",
        GamingKey::Escape => "escape",
    }
}

fn parse_mouse_button(value: &str) -> Result<GamingMouseButton, GameInputProfileRepositoryError> {
    match value {
        "left" => Ok(GamingMouseButton::Left),
        "right" => Ok(GamingMouseButton::Right),
        "middle" => Ok(GamingMouseButton::Middle),
        other => Err(GameInputProfileRepositoryError::Storage(format!("Unsupported mouse button: {other}"))),
    }
}

fn mouse_button_name(value: GamingMouseButton) -> &'static str {
    match value {
        GamingMouseButton::Left => "left",
        GamingMouseButton::Right => "right",
        GamingMouseButton::Middle => "middle",
    }
}

fn domain_error(error: impl std::fmt::Debug) -> GameInputProfileRepositoryError {
    GameInputProfileRepositoryError::Storage(format!("{error:?}"))
}
