// # 📄 Dosya Yolu: /turkuazvm/crates/android/src/domain/guest_agent.rs
// # 📌 Amac: Android Guest Agent capability ve persistent touch domain modellerini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Guest Agent wire protokolunden bagimsiz capability, status ve normalized multi-touch contact modellerini tutar
// # Bagimli Oldugu Katman: Service | Tool | View

const NORMALIZED_MAX: u16 = 10_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidGuestAgentReport {
    pub available: bool,
    pub persistent_multi_touch: bool,
    pub max_contacts: u8,
    pub continuation_api: bool,
    pub input_backend: Option<String>,
    pub endpoint: Option<String>,
    pub secure_transport_v2: bool,
    pub clipboard: bool,
}

impl AndroidGuestAgentReport {
    pub fn unavailable() -> Self {
        Self { available: false, persistent_multi_touch: false, max_contacts: 0, continuation_api: false, input_backend: None, endpoint: None, secure_transport_v2: false, clipboard: false }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndroidTouchPhase { Down, Move, Up }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AndroidTouchContact {
    pub pointer_id: u8,
    pub phase: AndroidTouchPhase,
    pub x: u16,
    pub y: u16,
}

impl AndroidTouchContact {
    pub fn create(pointer_id: u8, phase: AndroidTouchPhase, x: u16, y: u16) -> Result<Self, AndroidGuestAgentDomainError> {
        if x > NORMALIZED_MAX || y > NORMALIZED_MAX { return Err(AndroidGuestAgentDomainError::InvalidCoordinate); }
        Ok(Self { pointer_id, phase, x, y })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndroidGuestAgentDomainError { InvalidCoordinate }
