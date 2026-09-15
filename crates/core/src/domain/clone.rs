// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/domain/clone.rs
// # 📌 Amac: VM clone modlarini typed domain modeli olarak tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Full ve linked clone davranislarini magic string kullanmadan temsil eder
// # Bagimli Oldugu Katman: Service | Tool

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloneMode {
    Full,
    Linked,
}
