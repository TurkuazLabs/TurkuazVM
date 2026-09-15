// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/services/storage_host_service.rs
// # 📌 Amac: Host storage capacity ve portable image policy bilgisini orkestre eder
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Storage host probe sonucunu TurkuazVM runtime ve .tvmimg policy metadata'si ile birlestirir
// # Bagimli Oldugu Katman: Tool | View

use crate::domain::storage_host::{PortableImagePolicy, StorageHostReport};
use crate::ports::storage_host_port::{StorageHostError, StorageHostPort};


pub struct StorageHostService<S>
where
    S: StorageHostPort,
{
    storage_host: S,
    qemu_img_available: bool,
    default_runtime_format: String,
    default_disk_size_gib: u64,
    portable_image_policy: PortableImagePolicy,
}

impl<S> StorageHostService<S>
where
    S: StorageHostPort,
{
    pub fn new(
        storage_host: S,
        qemu_img_available: bool,
        default_runtime_format: String,
        default_disk_size_gib: u64,
        portable_image_policy: PortableImagePolicy,
    ) -> Self {
        Self {
            storage_host,
            qemu_img_available,
            default_runtime_format,
            default_disk_size_gib,
            portable_image_policy,
        }
    }

    pub fn report(&self) -> Result<StorageHostReport, StorageHostError> {
        let capacity = self.storage_host.capacity()?;
        Ok(StorageHostReport {
            data_root: capacity.data_root,
            total_bytes: capacity.total_bytes,
            available_bytes: capacity.available_bytes,
            qemu_img_available: self.qemu_img_available,
            default_runtime_format: self.default_runtime_format.clone(),
            default_disk_size_gib: self.default_disk_size_gib,
            portable_image_extension: self.portable_image_policy.extension.clone(),
            portable_container: self.portable_image_policy.container.clone(),
            private_copy_default: self.portable_image_policy.private_copy_default,
        })
    }
}
