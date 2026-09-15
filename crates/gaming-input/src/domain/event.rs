// # 📄 Dosya Yolu: /turkuazvm/crates/gaming-input/src/domain/event.rs
// # 📌 Amac: Host tarafindan uretilen gaming input eventlerini typed domain modeliyle tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Keyboard, mouse capture/motion/button ve concrete gamepad eventlerini platform keycode stringlerinden ayirir
// # Bagimli Oldugu Katman: Service | Tool

use crate::domain::profile::{GamingKey, GamingMouseButton};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GamingInputEvent {
    Key { key: GamingKey, pressed: bool },
    MouseButton { button: GamingMouseButton, pressed: bool },
    MouseMotion { delta_x: i32, delta_y: i32 },
    MouseCapture { captured: bool },
    GamepadButton { button: u16, pressed: bool },
    GamepadAxis { axis: u16, value_milli: i16 },
}
