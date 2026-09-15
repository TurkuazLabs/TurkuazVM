// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/ports/mod.rs
// # 📌 Amac: TurkuazVM core tarafindan dis dunyaya acilan port traitlerini toplar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Platform, firmware, media, hypervisor, network, storage ve repository adapter contractlarini tanimlar
// # Bagimli Oldugu Katman: Service | Repo | Tool

pub mod clone_storage_port;
pub mod firmware_port;
pub mod host_probe_port;
pub mod hypervisor_monitor_port;
pub mod hypervisor_probe_port;
pub mod hypervisor_runtime_port;
pub mod media_port;
pub mod network_port;
pub mod snapshot_port;
pub mod storage_port;
pub mod vm_repository_port;
pub mod storage_host_port;

pub mod runtime_registry_port;
pub mod runtime_recovery_repository_port;
