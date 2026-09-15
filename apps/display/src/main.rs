// # 📄 Dosya Yolu: /turkuazvm/apps/display/src/main.rs
// # 📌 Amac: TurkuazDisplay native process bootstrap giris noktasini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: CLI config'i yukler ve display controller'i baslatir
// # Bagimli Oldugu Katman: Controller | Service | Tool | View

mod config;
mod controllers;
mod services;
mod tools;
mod views;

use config::display_config::DisplayConfig;
use controllers::display_controller::DisplayController;
use views::console_view::ConsoleView;

fn main() {
    let result = DisplayConfig::from_args().and_then(DisplayController::run);
    if let Err(error) = result {
        ConsoleView::error(&error);
        std::process::exit(1);
    }
}
