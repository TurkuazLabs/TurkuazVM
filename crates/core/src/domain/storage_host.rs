// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/domain/storage_host.rs
// # 📌 Amac: Host storage kapasite ve portable image format bilgisini domain modelinde tutar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Data root kapasitesi, qemu-img hazirligi ve TurkuazVM portable image format metadata'sini modeller
// # Bagimli Oldugu Katman: Service | Tool | View


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortableImagePolicy {
    pub extension: String,
    pub container: String,
    pub private_copy_default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageHostReport {
    pub data_root: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub qemu_img_available: bool,
    pub default_runtime_format: String,
    pub default_disk_size_gib: u64,
    pub portable_image_extension: String,
    pub portable_container: String,
    pub private_copy_default: bool,
}
