// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/domain/mod.rs
// # 📌 Amac: TurkuazVM domain tiplerini tek public modul altinda toplar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Disk, display, guest boot, network, host, hypervisor, kontrol kanali ve VM lifecycle modellerini disariya acar
// # Bagimli Oldugu Katman: Service

pub mod clone;
pub mod disk;
pub mod display;
pub mod guest_boot;
pub mod host;
pub mod hypervisor;
pub mod hypervisor_control;
pub mod network;
pub mod snapshot;
pub mod virtual_machine;
pub mod vm_state;

pub mod runtime_media;
pub mod storage_host;

pub mod runtime_registration;
pub mod hypervisor_runtime_event;
pub mod runtime_recovery;
