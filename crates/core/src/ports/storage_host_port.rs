// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/ports/storage_host_port.rs
// # 📌 Amac: Host storage kapasite probe adapter contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Core katmanini filesystem kapasite ve host storage API detaylarindan ayirir
// # Bagimli Oldugu Katman: Service | Tool

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageCapacityInfo {
    pub data_root: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageHostError {
    ProbeFailed(String),
}

pub trait StorageHostPort {
    fn capacity(&self) -> Result<StorageCapacityInfo, StorageHostError>;
}
