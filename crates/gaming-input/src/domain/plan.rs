// # 📄 Dosya Yolu: /turkuazvm/crates/gaming-input/src/domain/plan.rs
// # 📌 Amac: Host inputundan uretilen Android-yonlu semantic input planlarini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: ADB ile uygulanabilir tekil eylemleri persistent multi-touch gerektiren pointer frame planlarindan ayirir
// # Bagimli Oldugu Katman: Service | Tool

use crate::domain::profile::NormalizedPoint;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchPhase {
    Down,
    Move,
    Up,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TouchContact {
    pub pointer_id: u8,
    pub phase: TouchPhase,
    pub position: NormalizedPoint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GamingInputPlan {
    Noop,
    AndroidTap { position: NormalizedPoint },
    AndroidKey { key_code: u32 },
    TouchFrame { contacts: Vec<TouchContact> },
}
