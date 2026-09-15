// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/commands/detach_network_command.rs
// # 📌 Amac: VM network detach use-case girdisini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Controller veya orkestrasyon servisi tarafindan NetworkService katmanina tasinan detach verisini modeller
// # Bagimli Oldugu Katman: Controller | Service

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetachNetworkCommand {
    pub vm_id: String,
    pub network_id: String,
}

impl DetachNetworkCommand {
    pub fn new(vm_id: impl Into<String>, network_id: impl Into<String>) -> Self {
        Self {
            vm_id: vm_id.into(),
            network_id: network_id.into(),
        }
    }
}
