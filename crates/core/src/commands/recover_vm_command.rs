// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/commands/recover_vm_command.rs
// # 📌 Amac: Guvenli runtime start hatasindan sonra VM recovery use-case girdisini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Error durumundaki VM kimligini Service katmanina tasir ve kontrollu Stopped recovery akisini baslatir
// # Bagimli Oldugu Katman: Controller | Service

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoverVmCommand {
    pub vm_id: String,
}

impl RecoverVmCommand {
    pub fn new(vm_id: impl Into<String>) -> Self {
        Self { vm_id: vm_id.into() }
    }
}
