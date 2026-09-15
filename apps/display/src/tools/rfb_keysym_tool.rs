// # 📄 Dosya Yolu: /turkuazvm/apps/display/src/tools/rfb_keysym_tool.rs
// # 📌 Amac: Winit fiziksel klavye kodlarini RFB/X11 keysym degerlerine donusturur
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Native display klavye inputunun QEMU RFB serverina magic number dagitmadan aktarilmasini saglar
// # Bagimli Oldugu Katman: Service | Tool

use winit::keyboard::KeyCode;

const XK_BACKSPACE: u32 = 0xFF08;
const XK_TAB: u32 = 0xFF09;
const XK_RETURN: u32 = 0xFF0D;
const XK_ESCAPE: u32 = 0xFF1B;
const XK_INSERT: u32 = 0xFF63;
const XK_DELETE: u32 = 0xFFFF;
const XK_HOME: u32 = 0xFF50;
const XK_END: u32 = 0xFF57;
const XK_PAGE_UP: u32 = 0xFF55;
const XK_PAGE_DOWN: u32 = 0xFF56;
const XK_LEFT: u32 = 0xFF51;
const XK_UP: u32 = 0xFF52;
const XK_RIGHT: u32 = 0xFF53;
const XK_DOWN: u32 = 0xFF54;
const XK_SHIFT_L: u32 = 0xFFE1;
const XK_SHIFT_R: u32 = 0xFFE2;
const XK_CONTROL_L: u32 = 0xFFE3;
const XK_CONTROL_R: u32 = 0xFFE4;
const XK_ALT_L: u32 = 0xFFE9;
const XK_ALT_R: u32 = 0xFFEA;
const XK_SUPER_L: u32 = 0xFFEB;
const XK_SUPER_R: u32 = 0xFFEC;
const XK_F1: u32 = 0xFFBE;

pub fn keysym(code: KeyCode) -> Option<u32> {
    match code {
        KeyCode::KeyA => Some(u32::from(b'a')),
        KeyCode::KeyB => Some(u32::from(b'b')),
        KeyCode::KeyC => Some(u32::from(b'c')),
        KeyCode::KeyD => Some(u32::from(b'd')),
        KeyCode::KeyE => Some(u32::from(b'e')),
        KeyCode::KeyF => Some(u32::from(b'f')),
        KeyCode::KeyG => Some(u32::from(b'g')),
        KeyCode::KeyH => Some(u32::from(b'h')),
        KeyCode::KeyI => Some(u32::from(b'i')),
        KeyCode::KeyJ => Some(u32::from(b'j')),
        KeyCode::KeyK => Some(u32::from(b'k')),
        KeyCode::KeyL => Some(u32::from(b'l')),
        KeyCode::KeyM => Some(u32::from(b'm')),
        KeyCode::KeyN => Some(u32::from(b'n')),
        KeyCode::KeyO => Some(u32::from(b'o')),
        KeyCode::KeyP => Some(u32::from(b'p')),
        KeyCode::KeyQ => Some(u32::from(b'q')),
        KeyCode::KeyR => Some(u32::from(b'r')),
        KeyCode::KeyS => Some(u32::from(b's')),
        KeyCode::KeyT => Some(u32::from(b't')),
        KeyCode::KeyU => Some(u32::from(b'u')),
        KeyCode::KeyV => Some(u32::from(b'v')),
        KeyCode::KeyW => Some(u32::from(b'w')),
        KeyCode::KeyX => Some(u32::from(b'x')),
        KeyCode::KeyY => Some(u32::from(b'y')),
        KeyCode::KeyZ => Some(u32::from(b'z')),
        KeyCode::Digit0 => Some(u32::from(b'0')),
        KeyCode::Digit1 => Some(u32::from(b'1')),
        KeyCode::Digit2 => Some(u32::from(b'2')),
        KeyCode::Digit3 => Some(u32::from(b'3')),
        KeyCode::Digit4 => Some(u32::from(b'4')),
        KeyCode::Digit5 => Some(u32::from(b'5')),
        KeyCode::Digit6 => Some(u32::from(b'6')),
        KeyCode::Digit7 => Some(u32::from(b'7')),
        KeyCode::Digit8 => Some(u32::from(b'8')),
        KeyCode::Digit9 => Some(u32::from(b'9')),
        KeyCode::Space => Some(u32::from(b' ')),
        KeyCode::Minus => Some(u32::from(b'-')),
        KeyCode::Equal => Some(u32::from(b'=')),
        KeyCode::BracketLeft => Some(u32::from(b'[')),
        KeyCode::BracketRight => Some(u32::from(b']')),
        KeyCode::Backslash => Some(u32::from(b'\\')),
        KeyCode::Semicolon => Some(u32::from(b';')),
        KeyCode::Quote => Some(u32::from(b'\'')),
        KeyCode::Comma => Some(u32::from(b',')),
        KeyCode::Period => Some(u32::from(b'.')),
        KeyCode::Slash => Some(u32::from(b'/')),
        KeyCode::Backquote => Some(u32::from(b'`')),
        KeyCode::Backspace => Some(XK_BACKSPACE),
        KeyCode::Tab => Some(XK_TAB),
        KeyCode::Enter | KeyCode::NumpadEnter => Some(XK_RETURN),
        KeyCode::Escape => Some(XK_ESCAPE),
        KeyCode::Insert => Some(XK_INSERT),
        KeyCode::Delete => Some(XK_DELETE),
        KeyCode::Home => Some(XK_HOME),
        KeyCode::End => Some(XK_END),
        KeyCode::PageUp => Some(XK_PAGE_UP),
        KeyCode::PageDown => Some(XK_PAGE_DOWN),
        KeyCode::ArrowLeft => Some(XK_LEFT),
        KeyCode::ArrowUp => Some(XK_UP),
        KeyCode::ArrowRight => Some(XK_RIGHT),
        KeyCode::ArrowDown => Some(XK_DOWN),
        KeyCode::ShiftLeft => Some(XK_SHIFT_L),
        KeyCode::ShiftRight => Some(XK_SHIFT_R),
        KeyCode::ControlLeft => Some(XK_CONTROL_L),
        KeyCode::ControlRight => Some(XK_CONTROL_R),
        KeyCode::AltLeft => Some(XK_ALT_L),
        KeyCode::AltRight => Some(XK_ALT_R),
        KeyCode::SuperLeft => Some(XK_SUPER_L),
        KeyCode::SuperRight => Some(XK_SUPER_R),
        KeyCode::F1 => Some(XK_F1),
        KeyCode::F2 => Some(XK_F1 + 1),
        KeyCode::F3 => Some(XK_F1 + 2),
        KeyCode::F4 => Some(XK_F1 + 3),
        KeyCode::F5 => Some(XK_F1 + 4),
        KeyCode::F6 => Some(XK_F1 + 5),
        KeyCode::F7 => Some(XK_F1 + 6),
        KeyCode::F8 => Some(XK_F1 + 7),
        KeyCode::F9 => Some(XK_F1 + 8),
        KeyCode::F10 => Some(XK_F1 + 9),
        KeyCode::F11 => Some(XK_F1 + 10),
        KeyCode::F12 => Some(XK_F1 + 11),
        _ => None,
    }
}
