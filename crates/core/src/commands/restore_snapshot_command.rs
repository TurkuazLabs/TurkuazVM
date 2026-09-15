// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/commands/restore_snapshot_command.rs
// # 📌 Amac: Snapshot restore use-case girdisini tasir
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: VM ve snapshot kimligini Service katmanina tasir
// # Bagimli Oldugu Katman: Service

pub struct RestoreSnapshotCommand {
    pub vm_id: String,
    pub snapshot_id: String,
}

impl RestoreSnapshotCommand {
    pub fn new(vm_id: impl Into<String>, snapshot_id: impl Into<String>) -> Self {
        Self {
            vm_id: vm_id.into(),
            snapshot_id: snapshot_id.into(),
        }
    }
}
