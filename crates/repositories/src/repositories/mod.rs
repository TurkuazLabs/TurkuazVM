// # 📄 Dosya Yolu: /turkuazvm/crates/repositories/src/repositories/mod.rs
// # 📌 Amac: Repository adapterlerini tek public modul altinda toplar
// # 📌 Modul - Rust
// # Version: 0.40.0
// # Aciklama: VM, Android, runtime recovery ve Artifact Cache YAML repository adapterlerini disariya acar
// # Bagimli Oldugu Katman: Repo

pub mod in_memory_vm_repository;
pub mod yaml_android_profile_repository;
pub mod yaml_vm_repository;

pub mod shared_vm_repository;
pub mod yaml_game_input_profile_repository;

pub mod yaml_game_catalog_repository;
pub mod yaml_guest_catalog_repository;

pub mod yaml_android_image_repository;

pub mod yaml_runtime_registry_repository;
pub mod yaml_runtime_recovery_repository;
pub mod yaml_artifact_cache_repository;
pub mod yaml_android_distribution_source_cache_repository;
pub mod yaml_installer_media_source_cache_repository;
