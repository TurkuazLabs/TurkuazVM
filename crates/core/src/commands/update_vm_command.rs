// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/commands/update_vm_command.rs
// # 📌 Amac: VM kimlik disi temel kaynak ayarlarini guncelleme use-case girdisini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Ad, vCPU ve RAM degerlerini VM Configuration Service katmanina tasir
// # Bagimli Oldugu Katman: Controller | Service

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateVmCommand {
    pub vm_id: String,
    pub name: String,
    pub vcpu_count: u16,
    pub memory_mib: u64,
}

impl UpdateVmCommand {
    pub fn new(
        vm_id: impl Into<String>,
        name: impl Into<String>,
        vcpu_count: u16,
        memory_mib: u64,
    ) -> Self {
        Self {
            vm_id: vm_id.into(),
            name: name.into(),
            vcpu_count,
            memory_mib,
        }
    }
}
