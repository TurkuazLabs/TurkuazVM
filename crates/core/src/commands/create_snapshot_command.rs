// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/commands/create_snapshot_command.rs
// # 📌 Amac: Snapshot create use-case girdisini tasir
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: VM, snapshot kimligi, gorunen ad ve olusum zamanini Service katmanina tasir
// # Bagimli Oldugu Katman: Service

pub struct CreateSnapshotCommand {
    pub vm_id: String,
    pub snapshot_id: String,
    pub name: String,
    pub created_at_unix_ms: u64,
}

impl CreateSnapshotCommand {
    pub fn new(
        vm_id: impl Into<String>,
        snapshot_id: impl Into<String>,
        name: impl Into<String>,
        created_at_unix_ms: u64,
    ) -> Self {
        Self {
            vm_id: vm_id.into(),
            snapshot_id: snapshot_id.into(),
            name: name.into(),
            created_at_unix_ms,
        }
    }
}
