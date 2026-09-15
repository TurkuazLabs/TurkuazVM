// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/domain/runtime_recovery.rs
// # 📌 Amac: Durable runtime journal ve otomatik recovery queue domain modellerini tanimlar
// # 📌 Modul - Rust
// # Version: 0.40.3
// # Aciklama: Guest reset dahil hypervisor olaylarini sequence'li evidence ve retry kontrollu recovery istegine donusturur
// # Bagimli Oldugu Katman: Service | Repo

use crate::domain::virtual_machine::VmId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeJournalEventKind {
    Qmp { name: String },
    GuestReset,
    ProcessExited,
    ControlReattached,
    ControlUnavailable { detail: String },
    RecoverySucceeded,
    RecoveryFailed { detail: String },
    RecoveryExhausted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeJournalEvent {
    pub vm_id: VmId,
    pub observed_at_unix_ms: u64,
    pub kind: RuntimeJournalEventKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeRecoveryRequest {
    pub source_sequence: u64,
    pub vm_id: VmId,
    pub reason: String,
    pub attempts: u32,
    pub next_attempt_unix_ms: u64,
}
