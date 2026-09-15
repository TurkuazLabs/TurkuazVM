// # 📄 Dosya Yolu: /turkuazvm/apps/desktop/src-tauri/src/services/app_state.rs
// # 📌 Amac: Tauri command ve runtime update akislarinin paylastigi Desktop Service state'ini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: DesktopService instance'ini Mutex ile Tauri managed state icinde korur
// # Bagimli Oldugu Katman: Service

use std::sync::Mutex;

use super::desktop_service::DesktopService;

pub const EVENT_RUNTIME_UPDATE: &str = "turkuazvm://runtime-update";

pub struct DesktopAppState {
    pub service: Mutex<DesktopService>,
}

impl DesktopAppState {
    pub fn new(service: DesktopService) -> Self {
        Self {
            service: Mutex::new(service),
        }
    }
}
