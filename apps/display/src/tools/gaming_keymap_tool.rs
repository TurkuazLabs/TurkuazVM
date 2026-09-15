// # 📄 Dosya Yolu: /turkuazvm/apps/display/src/tools/gaming_keymap_tool.rs
// # 📌 Amac: Winit physical keyboard/mouse girdilerini Engine API Gaming Input enumlarina cevirir
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Platform/native keycode ayrintilarini Engine API DTO mappinginden ayirir
// # Bagimli Oldugu Katman: Tool

use turkuazvm_engine_api::{GamingKeyDto, GamingMouseButtonDto};
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

pub const fn key(code: KeyCode) -> Option<GamingKeyDto> {
    match code {
        KeyCode::KeyW => Some(GamingKeyDto::W),
        KeyCode::KeyA => Some(GamingKeyDto::A),
        KeyCode::KeyS => Some(GamingKeyDto::S),
        KeyCode::KeyD => Some(GamingKeyDto::D),
        KeyCode::Space => Some(GamingKeyDto::Space),
        KeyCode::KeyC => Some(GamingKeyDto::C),
        KeyCode::KeyZ => Some(GamingKeyDto::Z),
        KeyCode::KeyR => Some(GamingKeyDto::R),
        KeyCode::KeyF => Some(GamingKeyDto::F),
        KeyCode::KeyQ => Some(GamingKeyDto::Q),
        KeyCode::KeyE => Some(GamingKeyDto::E),
        KeyCode::ShiftLeft => Some(GamingKeyDto::ShiftLeft),
        KeyCode::ControlLeft => Some(GamingKeyDto::ControlLeft),
        KeyCode::Digit1 => Some(GamingKeyDto::Digit1),
        KeyCode::Digit2 => Some(GamingKeyDto::Digit2),
        KeyCode::Digit3 => Some(GamingKeyDto::Digit3),
        KeyCode::Digit4 => Some(GamingKeyDto::Digit4),
        KeyCode::Tab => Some(GamingKeyDto::Tab),
        KeyCode::Escape => Some(GamingKeyDto::Escape),
        _ => None,
    }
}

pub const fn mouse_button(button: MouseButton) -> Option<GamingMouseButtonDto> {
    match button {
        MouseButton::Left => Some(GamingMouseButtonDto::Left),
        MouseButton::Right => Some(GamingMouseButtonDto::Right),
        MouseButton::Middle => Some(GamingMouseButtonDto::Middle),
        _ => None,
    }
}
