// # 📄 Dosya Yolu: /turkuazvm/apps/display/src/views/console_view.rs
// # 📌 Amac: TurkuazDisplay bootstrap hatalarini terminal cikisina donusturur
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Process baslangic hatalarinin formatlama ve stderr cikisini View katmaninda tutar
// # Bagimli Oldugu Katman: View

pub struct ConsoleView;

impl ConsoleView {
    pub fn error(message: &str) {
        eprintln!("TurkuazDisplay v{} | ERROR | {}", env!("CARGO_PKG_VERSION"), message);
    }
}
