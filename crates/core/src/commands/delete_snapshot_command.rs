// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/commands/delete_snapshot_command.rs
// # 📌 Amac: Snapshot delete use-case girdisini tasir
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: VM ve snapshot kimligini Service katmanina tasir
// # Bagimli Oldugu Katman: Service

pub struct DeleteSnapshotCommand {
    pub vm_id: String,
    pub snapshot_id: String,
}

impl DeleteSnapshotCommand {
    pub fn new(vm_id: impl Into<String>, snapshot_id: impl Into<String>) -> Self {
        Self {
            vm_id: vm_id.into(),
            snapshot_id: snapshot_id.into(),
        }
    }
}
