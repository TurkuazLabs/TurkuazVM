// # 📄 Dosya Yolu: /turkuazvm/crates/gaming-input/src/ports/host_gamepad_port.rs
// # 📌 Amac: Cross-platform host gamepad observation adapter contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Gamepad discovery ve event polling'i GilRs veya platform API seciminden ayirir
// # Bagimli Oldugu Katman: Service | Tool

use crate::domain::event::GamingInputEvent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostGamepadPortError {
    Unavailable,
    Observation(String),
}

pub trait HostGamepadPort {
    fn available(&self) -> bool;
    fn poll_events(&mut self) -> Result<Vec<GamingInputEvent>, HostGamepadPortError>;
}
