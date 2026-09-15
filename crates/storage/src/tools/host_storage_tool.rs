// # 📄 Dosya Yolu: /turkuazvm/crates/storage/src/tools/host_storage_tool.rs
// # 📌 Amac: Host filesystem kapasitesini safe cross-platform API ile olcer
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: PowerShell/df parsing yerine fs2 ile total ve available byte degerlerini uretir
// # Bagimli Oldugu Katman: Tool

use std::fs;
use std::path::PathBuf;

use turkuazvm_core::ports::storage_host_port::{
    StorageCapacityInfo, StorageHostError, StorageHostPort,
};

#[derive(Clone)]
pub struct HostStorageTool {
    data_root: PathBuf,
}

impl HostStorageTool {
    pub fn new(data_root: PathBuf) -> Self {
        Self { data_root }
    }
}

impl StorageHostPort for HostStorageTool {
    fn capacity(&self) -> Result<StorageCapacityInfo, StorageHostError> {
        fs::create_dir_all(&self.data_root)
            .map_err(|error| StorageHostError::ProbeFailed(error.to_string()))?;
        let total_bytes = fs2::total_space(&self.data_root)
            .map_err(|error| StorageHostError::ProbeFailed(error.to_string()))?;
        let available_bytes = fs2::available_space(&self.data_root)
            .map_err(|error| StorageHostError::ProbeFailed(error.to_string()))?;
        Ok(StorageCapacityInfo {
            data_root: self.data_root.display().to_string(),
            total_bytes,
            available_bytes,
        })
    }
}
