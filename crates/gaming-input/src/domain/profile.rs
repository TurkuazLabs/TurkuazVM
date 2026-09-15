// # 📄 Dosya Yolu: /turkuazvm/crates/gaming-input/src/domain/profile.rs
// # 📌 Amac: Game input profile, normalized koordinat ve keymapping invariantlarini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: WASD joystick, relative mouse look ve keyboard/mouse bindinglerini cozunurlukten bagimsiz typed modellerle saklar
// # Bagimli Oldugu Katman: Service | Repo | View

const NORMALIZED_MAX: u16 = 10_000;
const MIN_SENSITIVITY_MILLI: u16 = 50;
const MAX_SENSITIVITY_MILLI: u16 = 5_000;
const MAX_BINDINGS: usize = 128;
const MAX_PACKAGE_NAME_LENGTH: usize = 255;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GamingVmId(String);

impl GamingVmId {
    pub fn parse(value: impl Into<String>) -> Result<Self, GamingInputProfileError> {
        let value = value.into();
        let valid = !value.is_empty()
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'));
        if !valid {
            return Err(GamingInputProfileError::InvalidVmId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NormalizedPoint {
    x: u16,
    y: u16,
}

impl NormalizedPoint {
    pub fn create(x: u16, y: u16) -> Result<Self, GamingInputProfileError> {
        if x > NORMALIZED_MAX || y > NORMALIZED_MAX {
            return Err(GamingInputProfileError::InvalidNormalizedCoordinate);
        }
        Ok(Self { x, y })
    }

    pub const fn x(self) -> u16 {
        self.x
    }

    pub const fn y(self) -> u16 {
        self.y
    }

    pub fn to_pixels(self, width: u32, height: u32) -> (u32, u32) {
        let x = u32::from(self.x) * width.saturating_sub(1) / u32::from(NORMALIZED_MAX);
        let y = u32::from(self.y) * height.saturating_sub(1) / u32::from(NORMALIZED_MAX);
        (x, y)
    }

    pub fn offset_clamped(self, delta_x: i32, delta_y: i32) -> Self {
        let x = (i32::from(self.x) + delta_x).clamp(0, i32::from(NORMALIZED_MAX)) as u16;
        let y = (i32::from(self.y) + delta_y).clamp(0, i32::from(NORMALIZED_MAX)) as u16;
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamingKey {
    W,
    A,
    S,
    D,
    Space,
    C,
    Z,
    R,
    F,
    Q,
    E,
    ShiftLeft,
    ControlLeft,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Tab,
    Escape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamingMouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputSource {
    Key(GamingKey),
    MouseButton(GamingMouseButton),
    GamepadButton(u16),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputTarget {
    Tap { position: NormalizedPoint },
    HoldTouch { pointer_id: u8, position: NormalizedPoint },
    AndroidKey { key_code: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputBinding {
    source: InputSource,
    target: InputTarget,
}

impl InputBinding {
    pub const fn new(source: InputSource, target: InputTarget) -> Self {
        Self { source, target }
    }

    pub const fn source(&self) -> InputSource {
        self.source
    }

    pub const fn target(&self) -> &InputTarget {
        &self.target
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualJoystickProfile {
    enabled: bool,
    center: NormalizedPoint,
    radius: u16,
    pointer_id: u8,
}

impl VirtualJoystickProfile {
    pub fn create(
        enabled: bool,
        center: NormalizedPoint,
        radius: u16,
        pointer_id: u8,
    ) -> Result<Self, GamingInputProfileError> {
        if radius == 0 || radius > NORMALIZED_MAX / 2 {
            return Err(GamingInputProfileError::InvalidJoystickRadius);
        }
        Ok(Self {
            enabled,
            center,
            radius,
            pointer_id,
        })
    }

    pub const fn enabled(&self) -> bool { self.enabled }
    pub const fn center(&self) -> NormalizedPoint { self.center }
    pub const fn radius(&self) -> u16 { self.radius }
    pub const fn pointer_id(&self) -> u8 { self.pointer_id }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MouseLookProfile {
    enabled: bool,
    anchor: NormalizedPoint,
    sensitivity_x_milli: u16,
    sensitivity_y_milli: u16,
    pointer_id: u8,
}

impl MouseLookProfile {
    pub fn create(
        enabled: bool,
        anchor: NormalizedPoint,
        sensitivity_x_milli: u16,
        sensitivity_y_milli: u16,
        pointer_id: u8,
    ) -> Result<Self, GamingInputProfileError> {
        if !(MIN_SENSITIVITY_MILLI..=MAX_SENSITIVITY_MILLI).contains(&sensitivity_x_milli)
            || !(MIN_SENSITIVITY_MILLI..=MAX_SENSITIVITY_MILLI).contains(&sensitivity_y_milli)
        {
            return Err(GamingInputProfileError::InvalidMouseSensitivity);
        }
        Ok(Self {
            enabled,
            anchor,
            sensitivity_x_milli,
            sensitivity_y_milli,
            pointer_id,
        })
    }

    pub const fn enabled(&self) -> bool { self.enabled }
    pub const fn anchor(&self) -> NormalizedPoint { self.anchor }
    pub const fn sensitivity_x_milli(&self) -> u16 { self.sensitivity_x_milli }
    pub const fn sensitivity_y_milli(&self) -> u16 { self.sensitivity_y_milli }
    pub const fn pointer_id(&self) -> u8 { self.pointer_id }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameInputProfile {
    vm_id: GamingVmId,
    package_name: Option<String>,
    joystick: VirtualJoystickProfile,
    mouse_look: MouseLookProfile,
    bindings: Vec<InputBinding>,
}

impl GameInputProfile {
    pub fn create(
        vm_id: GamingVmId,
        package_name: Option<String>,
        joystick: VirtualJoystickProfile,
        mouse_look: MouseLookProfile,
        bindings: Vec<InputBinding>,
    ) -> Result<Self, GamingInputProfileError> {
        if bindings.len() > MAX_BINDINGS {
            return Err(GamingInputProfileError::TooManyBindings);
        }
        let package_name = package_name
            .map(|value| validate_package_name(value))
            .transpose()?;
        if joystick.pointer_id() == mouse_look.pointer_id() {
            return Err(GamingInputProfileError::PointerIdCollision);
        }
        let mut hold_pointer_ids = std::collections::HashSet::new();
        for (index, binding) in bindings.iter().enumerate() {
            if bindings
                .iter()
                .skip(index + 1)
                .any(|other| other.source() == binding.source())
            {
                return Err(GamingInputProfileError::DuplicateSourceBinding);
            }
            if let InputTarget::HoldTouch { pointer_id, .. } = binding.target() {
                if *pointer_id == joystick.pointer_id()
                    || *pointer_id == mouse_look.pointer_id()
                    || !hold_pointer_ids.insert(*pointer_id)
                {
                    return Err(GamingInputProfileError::PointerIdCollision);
                }
            }
        }
        Ok(Self {
            vm_id,
            package_name,
            joystick,
            mouse_look,
            bindings,
        })
    }

    pub fn vm_id(&self) -> &GamingVmId { &self.vm_id }
    pub fn package_name(&self) -> Option<&str> { self.package_name.as_deref() }
    pub const fn joystick(&self) -> &VirtualJoystickProfile { &self.joystick }
    pub const fn mouse_look(&self) -> &MouseLookProfile { &self.mouse_look }
    pub fn bindings(&self) -> &[InputBinding] { &self.bindings }

    pub fn binding_for(&self, source: InputSource) -> Option<&InputBinding> {
        self.bindings.iter().find(|binding| binding.source() == source)
    }
}

fn validate_package_name(value: String) -> Result<String, GamingInputProfileError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_PACKAGE_NAME_LENGTH {
        return Err(GamingInputProfileError::InvalidPackageName);
    }
    let valid = trimmed
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_'))
        && trimmed.contains('.');
    if !valid {
        return Err(GamingInputProfileError::InvalidPackageName);
    }
    Ok(trimmed.to_owned())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GamingInputProfileError {
    InvalidVmId,
    InvalidNormalizedCoordinate,
    InvalidJoystickRadius,
    InvalidMouseSensitivity,
    InvalidPackageName,
    TooManyBindings,
    DuplicateSourceBinding,
    PointerIdCollision,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized_point_maps_to_display_pixels() {
        let point = NormalizedPoint::create(5_000, 5_000).expect("point");
        assert_eq!(point.to_pixels(1920, 1080), (959, 539));
    }

    #[test]
    fn duplicate_source_binding_is_rejected() {
        let center = NormalizedPoint::create(2_000, 8_000).expect("center");
        let look = NormalizedPoint::create(7_000, 5_000).expect("look");
        let bindings = vec![
            InputBinding::new(InputSource::Key(GamingKey::F), InputTarget::AndroidKey { key_code: 23 }),
            InputBinding::new(InputSource::Key(GamingKey::F), InputTarget::AndroidKey { key_code: 24 }),
        ];
        let profile = GameInputProfile::create(
            GamingVmId::parse("android-game").expect("vm"),
            None,
            VirtualJoystickProfile::create(true, center, 1_000, 0).expect("joystick"),
            MouseLookProfile::create(true, look, 1_000, 1_000, 1).expect("look"),
            bindings,
        );
        assert_eq!(profile, Err(GamingInputProfileError::DuplicateSourceBinding));
    }
}
