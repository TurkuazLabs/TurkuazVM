// # 📄 Dosya Yolu: /turkuazvm/crates/repositories/src/repositories/yaml_runtime_recovery_repository.rs
// # 📌 Amac: Runtime event journal, sequence cursor ve recovery queue verilerini atomik YAML ile saklar
// # 📌 Modul - Rust
// # Version: 0.40.3
// # Aciklama: Guest reset dahil per-event journal, corrupt quarantine, retention ve VM bazli dedupe queue persistence uygular
// # Bagimli Oldugu Katman: Repo

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use turkuazvm_core::domain::runtime_recovery::{
    RuntimeJournalEvent, RuntimeJournalEventKind, RuntimeRecoveryRequest,
};
use turkuazvm_core::domain::virtual_machine::VmId;
use turkuazvm_core::ports::runtime_recovery_repository_port::{
    RuntimeRecoveryRepositoryError, RuntimeRecoveryRepositoryPort,
};

const SCHEMA_VERSION: u16 = 1;
const CURSOR_FILE_NAME: &str = "cursor.yml";
const JOURNAL_DIRECTORY_NAME: &str = "journal";
const QUEUE_DIRECTORY_NAME: &str = "queue";
fn yaml_header() -> String {
    format!("# 📄 Dosya Yolu: runtime-generated\n# 📌 Amac: TurkuazVM runtime recovery persistence verisi\n# 📌 Modul - YAML\n# Version: {}\n# Aciklama: Engine tarafindan atomik olarak uretilir; elle duzenlenmemelidir\n# Bagimli Oldugu Katman: Repo\n\n", env!("CARGO_PKG_VERSION"))
}

#[derive(Debug, Clone)]
pub struct YamlRuntimeRecoveryRepository {
    root: PathBuf,
    max_journal_entries: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct CursorDocument {
    schema_version: u16,
    last_sequence: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct JournalDocument {
    schema_version: u16,
    sequence: u64,
    vm_id: String,
    observed_at_unix_ms: u64,
    kind: String,
    detail: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct QueueDocument {
    schema_version: u16,
    source_sequence: u64,
    vm_id: String,
    reason: String,
    attempts: u32,
    next_attempt_unix_ms: u64,
}

impl YamlRuntimeRecoveryRepository {
    pub fn new(data_root: PathBuf, max_journal_entries: usize) -> Self {
        Self {
            root: data_root.join("runtime-recovery"),
            max_journal_entries: max_journal_entries.max(1),
        }
    }

    fn journal_root(&self) -> PathBuf { self.root.join(JOURNAL_DIRECTORY_NAME) }
    fn queue_root(&self) -> PathBuf { self.root.join(QUEUE_DIRECTORY_NAME) }
    fn cursor_path(&self) -> PathBuf { self.root.join(CURSOR_FILE_NAME) }

    fn ensure_layout(&self) -> Result<(), RuntimeRecoveryRepositoryError> {
        fs::create_dir_all(self.journal_root()).map_err(io_error)?;
        fs::create_dir_all(self.queue_root()).map_err(io_error)
    }

    fn current_sequence(&self) -> Result<u64, RuntimeRecoveryRepositoryError> {
        let mut last_sequence = self.scan_max_journal_sequence()?;
        let cursor_path = self.cursor_path();
        if !cursor_path.exists() {
            return Ok(last_sequence);
        }
        match read_yaml::<CursorDocument>(&cursor_path) {
            Ok(document) if document.schema_version == SCHEMA_VERSION => {
                last_sequence = last_sequence.max(document.last_sequence);
            }
            Ok(_) => {
                return Err(RuntimeRecoveryRepositoryError::Invalid(String::from(
                    "runtime recovery cursor schema is unsupported",
                )));
            }
            Err(_) => {
                quarantine(&cursor_path)?;
            }
        }
        Ok(last_sequence)
    }

    fn scan_max_journal_sequence(&self) -> Result<u64, RuntimeRecoveryRepositoryError> {
        let root = self.journal_root();
        if !root.exists() {
            return Ok(0);
        }
        let mut maximum = 0_u64;
        for entry in fs::read_dir(root).map_err(io_error)? {
            let entry = entry.map_err(io_error)?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("yml") {
                continue;
            }
            if let Some(sequence) = path.file_stem().and_then(|value| value.to_str()).and_then(|value| value.parse::<u64>().ok()) {
                maximum = maximum.max(sequence);
            }
        }
        Ok(maximum)
    }

    fn prune_journal(&self) -> Result<(), RuntimeRecoveryRepositoryError> {
        let root = self.journal_root();
        let mut entries = fs::read_dir(root)
            .map_err(io_error)?
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let path = entry.path();
                let sequence = path.file_stem()?.to_str()?.parse::<u64>().ok()?;
                Some((sequence, path))
            })
            .collect::<Vec<_>>();
        entries.sort_by_key(|(sequence, _)| *sequence);
        let remove_count = entries.len().saturating_sub(self.max_journal_entries);
        for (_, path) in entries.into_iter().take(remove_count) {
            fs::remove_file(path).map_err(io_error)?;
        }
        Ok(())
    }
}

impl RuntimeRecoveryRepositoryPort for YamlRuntimeRecoveryRepository {
    fn append_event(&mut self, event: &RuntimeJournalEvent) -> Result<u64, RuntimeRecoveryRepositoryError> {
        self.ensure_layout()?;
        let sequence = self.current_sequence()?.saturating_add(1);
        let (kind, detail) = encode_event_kind(&event.kind);
        let document = JournalDocument {
            schema_version: SCHEMA_VERSION,
            sequence,
            vm_id: event.vm_id.as_str().to_owned(),
            observed_at_unix_ms: event.observed_at_unix_ms,
            kind,
            detail,
        };
        let event_path = self.journal_root().join(format!("{sequence:020}.yml"));
        write_yaml_atomic(&event_path, &document)?;
        write_yaml_atomic(&self.cursor_path(), &CursorDocument {
            schema_version: SCHEMA_VERSION,
            last_sequence: sequence,
        })?;
        self.prune_journal()?;
        Ok(sequence)
    }

    fn pending_requests(&self) -> Result<Vec<RuntimeRecoveryRequest>, RuntimeRecoveryRepositoryError> {
        self.ensure_layout()?;
        let mut requests = Vec::new();
        for entry in fs::read_dir(self.queue_root()).map_err(io_error)? {
            let entry = entry.map_err(io_error)?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("yml") {
                continue;
            }
            let document = match read_yaml::<QueueDocument>(&path) {
                Ok(document) => document,
                Err(_) => {
                    quarantine(&path)?;
                    continue;
                }
            };
            if document.schema_version != SCHEMA_VERSION {
                return Err(RuntimeRecoveryRepositoryError::Invalid(format!(
                    "runtime recovery queue schema {} is unsupported",
                    document.schema_version
                )));
            }
            let vm_id = VmId::parse(document.vm_id).map_err(|error| {
                RuntimeRecoveryRepositoryError::Invalid(format!("runtime recovery VM id invalid: {error:?}"))
            })?;
            requests.push(RuntimeRecoveryRequest {
                source_sequence: document.source_sequence,
                vm_id,
                reason: document.reason,
                attempts: document.attempts,
                next_attempt_unix_ms: document.next_attempt_unix_ms,
            });
        }
        requests.sort_by_key(|request| request.source_sequence);
        Ok(requests)
    }

    fn save_request(&mut self, request: &RuntimeRecoveryRequest) -> Result<(), RuntimeRecoveryRepositoryError> {
        self.ensure_layout()?;
        let path = self.queue_root().join(format!("{}.yml", request.vm_id.as_str()));
        write_yaml_atomic(&path, &QueueDocument {
            schema_version: SCHEMA_VERSION,
            source_sequence: request.source_sequence,
            vm_id: request.vm_id.as_str().to_owned(),
            reason: request.reason.clone(),
            attempts: request.attempts,
            next_attempt_unix_ms: request.next_attempt_unix_ms,
        })
    }

    fn remove_request(&mut self, vm_id: &VmId) -> Result<(), RuntimeRecoveryRepositoryError> {
        let path = self.queue_root().join(format!("{}.yml", vm_id.as_str()));
        if path.exists() {
            fs::remove_file(path).map_err(io_error)?;
        }
        Ok(())
    }
}

fn encode_event_kind(kind: &RuntimeJournalEventKind) -> (String, Option<String>) {
    match kind {
        RuntimeJournalEventKind::Qmp { name } => (String::from("qmp"), Some(name.clone())),
        RuntimeJournalEventKind::GuestReset => (String::from("guest_reset"), None),
        RuntimeJournalEventKind::ProcessExited => (String::from("process_exited"), None),
        RuntimeJournalEventKind::ControlReattached => (String::from("control_reattached"), None),
        RuntimeJournalEventKind::ControlUnavailable { detail } => (String::from("control_unavailable"), Some(detail.clone())),
        RuntimeJournalEventKind::RecoverySucceeded => (String::from("recovery_succeeded"), None),
        RuntimeJournalEventKind::RecoveryFailed { detail } => (String::from("recovery_failed"), Some(detail.clone())),
        RuntimeJournalEventKind::RecoveryExhausted => (String::from("recovery_exhausted"), None),
    }
}

fn read_yaml<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, RuntimeRecoveryRepositoryError> {
    let content = fs::read_to_string(path).map_err(io_error)?;
    serde_yaml_ng::from_str(&content).map_err(|error| RuntimeRecoveryRepositoryError::Parse(error.to_string()))
}

fn write_yaml_atomic<T: Serialize>(path: &Path, value: &T) -> Result<(), RuntimeRecoveryRepositoryError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(io_error)?;
    }
    let body = serde_yaml_ng::to_string(value).map_err(|error| RuntimeRecoveryRepositoryError::Parse(error.to_string()))?;
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, format!("{}{body}", yaml_header())).map_err(io_error)?;
    fs::rename(&temp_path, path).map_err(io_error)
}

fn quarantine(path: &Path) -> Result<(), RuntimeRecoveryRepositoryError> {
    if !path.exists() {
        return Ok(());
    }
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|error| RuntimeRecoveryRepositoryError::Invalid(error.to_string()))?.as_millis();
    let bad_path = PathBuf::from(format!("{}.bad.{timestamp}", path.to_string_lossy()));
    fs::rename(path, bad_path).map_err(io_error)
}

fn io_error(error: std::io::Error) -> RuntimeRecoveryRepositoryError {
    RuntimeRecoveryRepositoryError::Io(error.to_string())
}
