// # 📄 Dosya Yolu: /turkuazvm/crates/qemu/src/ports/qmp_transport_port.rs
// # 📌 Amac: QMP client ile fiziksel socket transportu arasindaki contracti tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: QMP protocol testlerini gercek TCP baglantisindan bagimsiz hale getirir
// # Bagimli Oldugu Katman: Tool

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QmpTransportError {
    ConnectFailed(String),
    ReadFailed(String),
    WriteFailed(String),
    ConnectionClosed,
}

pub trait QmpTransportPort {
    fn read_message(&mut self) -> Result<String, QmpTransportError>;
    fn write_message(&mut self, message: &str) -> Result<(), QmpTransportError>;
}
