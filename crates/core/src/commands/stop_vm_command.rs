// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/commands/stop_vm_command.rs
// # 📌 Amac: VM durdurma use-case girdisini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Durdurulacak VM kimligini Service katmanina tasir
// # Bagimli Oldugu Katman: Controller | Service

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StopVmCommand {
    pub vm_id: String,
}

impl StopVmCommand {
    pub fn new(vm_id: impl Into<String>) -> Self {
        Self { vm_id: vm_id.into() }
    }
}
