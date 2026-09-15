// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/commands/eject_installer_media_command.rs
// # 📌 Amac: VM kurulum ISO'sunu ayirip kalici disk boot duzenine gecis istegini tasir
// # 📌 Modul - Rust
// # Version: 0.32.1
// # Aciklama: Stopped VM icin ISO dosyasini silmeden medya baglantisini kaldiran use-case girdisini tanimlar
// # Bagimli Oldugu Katman: Controller | Service

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EjectInstallerMediaCommand {
    pub vm_id: String,
}

impl EjectInstallerMediaCommand {
    pub fn new(vm_id: impl Into<String>) -> Self {
        Self {
            vm_id: vm_id.into(),
        }
    }
}
