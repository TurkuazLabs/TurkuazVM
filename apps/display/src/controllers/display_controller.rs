// # 📄 Dosya Yolu: /turkuazvm/apps/display/src/controllers/display_controller.rs
// # 📌 Amac: Display process requestini DisplayService use-case'ine aktarir
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Controller logic tutmadan typed config'i service'e yonlendirir
// # Bagimli Oldugu Katman: Service

use crate::config::display_config::DisplayConfig;
use crate::services::display_service::DisplayService;

pub struct DisplayController;

impl DisplayController {
    pub fn run(config: DisplayConfig) -> Result<(), String> {
        DisplayService::run(config)
    }
}
