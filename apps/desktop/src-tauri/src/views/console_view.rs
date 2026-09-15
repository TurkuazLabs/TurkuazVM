// # 📄 Dosya Yolu: /turkuazvm/apps/desktop/src-tauri/src/views/console_view.rs
// # 📌 Amac: Desktop bootstrap hata ciktilarini terminale formatlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Config ve Tauri bootstrap hatalarinin composition root icinde inline formatlanmasini engeller
// # Bagimli Oldugu Katman: View

pub struct ConsoleView;

impl ConsoleView {
    pub fn render_error(message: &str) {
        eprintln!("TurkuazVM Desktop v{} | ERROR | {}", env!("CARGO_PKG_VERSION"), message);
    }
}
