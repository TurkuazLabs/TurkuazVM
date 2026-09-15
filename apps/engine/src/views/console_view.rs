// # 📄 Dosya Yolu: /turkuazvm/apps/engine/src/views/console_view.rs
// # 📌 Amac: Engine terminal status ve hata ciktilarini formatlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Composition root loglarini Controller ve Service katmanlarindan ayirir
// # Bagimli Oldugu Katman: View


const ENGINE_VERSION: &str = "0.12.3";

pub struct ConsoleView;

impl ConsoleView {
    pub fn render_engine_listening(endpoint: &str) {
        println!("TurkuazVM Engine v{ENGINE_VERSION}");
        println!("Engine API: READY | {endpoint}");
    }

    pub fn render_engine_error(message: &str) {
        eprintln!("TurkuazVM Engine v{ENGINE_VERSION} | ERROR | {message}");
    }
}
