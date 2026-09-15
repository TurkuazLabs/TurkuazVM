// # 📄 Dosya Yolu: /turkuazvm/crates/repositories/src/repositories/yaml_artifact_cache_repository.rs
// # 📌 Amac: Content-addressed Artifact Cache payload ve source index persistence adapterini uygular
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: SHA-256 dedupe, cross-process lock, future-schema fail-closed, transactional index ve payload GC saglar
// # Bagimli Oldugu Katman: Repo

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use turkuazvm_artifact_cache::domain::artifact_cache::{ArtifactCacheRecord, ArtifactCacheRequest, ArtifactSourceValidators};
use turkuazvm_artifact_cache::ports::artifact_cache_repository_port::{ArtifactCacheRepositoryError, ArtifactCacheRepositoryPort};

const SCHEMA_VERSION: u16 = 2;
const MIN_SUPPORTED_SCHEMA_VERSION: u16 = 1;
fn yaml_header() -> String {
    format!("# 📄 Dosya Yolu: runtime-generated\n# 📌 Amac: TurkuazVM Artifact Cache source index metadata verisi\n# 📌 Modul - YAML\n# Version: {}\n# Aciklama: Engine tarafindan atomik olarak uretilir; elle duzenlenmemelidir\n# Bagimli Oldugu Katman: Repo\n\n", env!("CARGO_PKG_VERSION"))
}

#[derive(Debug)]
pub struct YamlArtifactCacheRepository {
    root: PathBuf,
    held_source_locks: HashMap<String, File>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RecordDocument {
    schema_version: u16,
    source_key: String,
    source_url: String,
    sha256: String,
    size_bytes: u64,
    immutable: bool,
    pinned: bool,
    #[serde(default)]
    etag: Option<String>,
    #[serde(default)]
    last_modified: Option<String>,
    created_at_unix_ms: u64,
    last_access_unix_ms: u64,
    #[serde(default)]
    last_revalidated_unix_ms: Option<u64>,
}

impl YamlArtifactCacheRepository {
    pub fn new(root: PathBuf) -> Self {
        Self { root, held_source_locks: HashMap::new() }
    }

    fn index_root(&self) -> PathBuf { self.root.join("indexes") }
    fn payload_root(&self) -> PathBuf { self.root.join("sha256") }
    fn lock_root(&self) -> PathBuf { self.root.join("locks") }

    fn ensure_layout(&self) -> Result<(), ArtifactCacheRepositoryError> {
        fs::create_dir_all(self.index_root()).map_err(io_error)?;
        fs::create_dir_all(self.payload_root()).map_err(io_error)?;
        fs::create_dir_all(self.lock_root()).map_err(io_error)
    }

    fn index_path(&self, source_key: &str) -> PathBuf {
        self.index_root().join(format!("{}.yml", sha256_bytes(source_key.as_bytes())))
    }

    fn payload_path(&self, sha256: &str) -> Result<PathBuf, ArtifactCacheRepositoryError> {
        validate_sha256(sha256)?;
        Ok(self.payload_root().join(&sha256[..2]).join(sha256).join("payload"))
    }

    fn record_from_document(document: RecordDocument) -> Result<ArtifactCacheRecord, ArtifactCacheRepositoryError> {
        if document.schema_version > SCHEMA_VERSION {
            return Err(ArtifactCacheRepositoryError::UnsupportedFutureSchema(document.schema_version));
        }
        if document.schema_version < MIN_SUPPORTED_SCHEMA_VERSION {
            return Err(ArtifactCacheRepositoryError::Invalid(format!("artifact cache schema {} is unsupported", document.schema_version)));
        }
        validate_sha256(&document.sha256)?;
        Ok(ArtifactCacheRecord {
            source_key: document.source_key,
            source_url: document.source_url,
            sha256: document.sha256,
            size_bytes: document.size_bytes,
            immutable: document.immutable,
            pinned: document.pinned,
            validators: ArtifactSourceValidators {
                etag: document.etag,
                last_modified: document.last_modified,
            },
            created_at_unix_ms: document.created_at_unix_ms,
            last_access_unix_ms: document.last_access_unix_ms,
            last_revalidated_unix_ms: document.last_revalidated_unix_ms,
        })
    }

    fn source_lock_path(&self, source_key: &str) -> PathBuf {
        self.lock_root().join(format!("source-{}.lock", sha256_bytes(source_key.as_bytes())))
    }

    fn digest_lock(&self, sha256: &str) -> Result<File, ArtifactCacheRepositoryError> {
        self.ensure_layout()?;
        validate_sha256(sha256)?;
        let path = self.lock_root().join(format!("digest-{sha256}.lock"));
        let file = OpenOptions::new().create(true).read(true).write(true).open(path).map_err(io_error)?;
        file.lock_exclusive().map_err(|error| ArtifactCacheRepositoryError::Lock(error.to_string()))?;
        Ok(file)
    }

    fn quarantine_index(&self, path: &Path) -> Result<(), ArtifactCacheRepositoryError> {
        if !path.exists() {
            return Ok(());
        }
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|value| value.as_millis())
            .unwrap_or(0);
        let file_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("artifact-index.yml");
        let quarantine = path.with_file_name(format!("{file_name}.bad.{timestamp}"));
        fs::rename(path, quarantine).map_err(io_error)
    }

    fn write_record(&self, record: &ArtifactCacheRecord) -> Result<(), ArtifactCacheRepositoryError> {
        self.ensure_layout()?;
        let document = RecordDocument {
            schema_version: SCHEMA_VERSION,
            source_key: record.source_key.clone(),
            source_url: record.source_url.clone(),
            sha256: record.sha256.clone(),
            size_bytes: record.size_bytes,
            immutable: record.immutable,
            pinned: record.pinned,
            etag: record.validators.etag.clone(),
            last_modified: record.validators.last_modified.clone(),
            created_at_unix_ms: record.created_at_unix_ms,
            last_access_unix_ms: record.last_access_unix_ms,
            last_revalidated_unix_ms: record.last_revalidated_unix_ms,
        };
        write_yaml_atomic(&self.index_path(&record.source_key), &document)
    }

    fn payload_is_referenced(&self, sha256: &str) -> Result<bool, ArtifactCacheRepositoryError> {
        for record in self.list()? {
            if record.sha256 == sha256 {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

impl ArtifactCacheRepositoryPort for YamlArtifactCacheRepository {
    fn acquire_source_lock(&mut self, source_key: &str) -> Result<(), ArtifactCacheRepositoryError> {
        self.ensure_layout()?;
        if self.held_source_locks.contains_key(source_key) {
            return Ok(());
        }
        let file = OpenOptions::new().create(true).read(true).write(true)
            .open(self.source_lock_path(source_key)).map_err(io_error)?;
        file.lock_exclusive().map_err(|error| ArtifactCacheRepositoryError::Lock(error.to_string()))?;
        self.held_source_locks.insert(source_key.to_owned(), file);
        Ok(())
    }

    fn release_source_lock(&mut self, source_key: &str) -> Result<(), ArtifactCacheRepositoryError> {
        if let Some(file) = self.held_source_locks.remove(source_key) {
            FileExt::unlock(&file).map_err(|error| ArtifactCacheRepositoryError::Lock(error.to_string()))?;
        }
        Ok(())
    }

    fn find_by_source(&self, source_key: &str) -> Result<Option<ArtifactCacheRecord>, ArtifactCacheRepositoryError> {
        self.ensure_layout()?;
        let path = self.index_path(source_key);
        if !path.is_file() { return Ok(None); }
        let record = match read_yaml::<RecordDocument>(&path).and_then(Self::record_from_document) {
            Ok(record) => record,
            Err(ArtifactCacheRepositoryError::UnsupportedFutureSchema(version)) => {
                return Err(ArtifactCacheRepositoryError::UnsupportedFutureSchema(version));
            }
            Err(ArtifactCacheRepositoryError::Parse(_)) | Err(ArtifactCacheRepositoryError::Invalid(_)) => {
                self.quarantine_index(&path)?;
                return Ok(None);
            }
            Err(error) => return Err(error),
        };
        if record.source_key != source_key {
            self.quarantine_index(&path)?;
            return Ok(None);
        }
        Ok(Some(record))
    }

    fn list(&self) -> Result<Vec<ArtifactCacheRecord>, ArtifactCacheRepositoryError> {
        self.ensure_layout()?;
        let mut records = Vec::new();
        for entry in fs::read_dir(self.index_root()).map_err(io_error)? {
            let entry = entry.map_err(io_error)?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("yml") { continue; }
            match read_yaml::<RecordDocument>(&path).and_then(Self::record_from_document) {
                Ok(record) => records.push(record),
                Err(ArtifactCacheRepositoryError::UnsupportedFutureSchema(version)) => {
                    return Err(ArtifactCacheRepositoryError::UnsupportedFutureSchema(version));
                }
                Err(ArtifactCacheRepositoryError::Parse(_)) | Err(ArtifactCacheRepositoryError::Invalid(_)) => {
                    self.quarantine_index(&path)?;
                }
                Err(error) => return Err(error),
            }
        }
        records.sort_by(|left, right| left.source_key.cmp(&right.source_key));
        Ok(records)
    }

    fn import_file(
        &mut self,
        request: &ArtifactCacheRequest,
        source_path: &Path,
        now_unix_ms: u64,
    ) -> Result<ArtifactCacheRecord, ArtifactCacheRepositoryError> {
        self.ensure_layout()?;
        if !source_path.is_file() {
            return Err(ArtifactCacheRepositoryError::Invalid(String::from("artifact source file is missing")));
        }
        let (sha256, size_bytes) = sha256_file(source_path)?;
        let digest_lock = self.digest_lock(&sha256)?;
        let payload_path = self.payload_path(&sha256)?;
        if !payload_path.is_file() {
            let parent = payload_path.parent().ok_or_else(|| ArtifactCacheRepositoryError::Invalid(String::from("artifact payload parent missing")))?;
            fs::create_dir_all(parent).map_err(io_error)?;
            let temp = parent.join(format!("payload.tmp.{}.{}", std::process::id(), now_unix_ms));
            copy_file_fsync(source_path, &temp)?;
            if payload_path.is_file() {
                let _ = fs::remove_file(&temp);
            } else {
                fs::rename(&temp, &payload_path).map_err(io_error)?;
            }
        }
        FileExt::unlock(&digest_lock).map_err(|error| ArtifactCacheRepositoryError::Lock(error.to_string()))?;
        let existing = self.find_by_source(&request.source_key)?;
        if let Some(record) = existing.as_ref() {
            if record.source_url != request.source_url {
                return Err(ArtifactCacheRepositoryError::Invalid(String::from("artifact source_key is already bound to a different URL")));
            }
        }
        let created_at_unix_ms = existing.as_ref().map(|record| record.created_at_unix_ms).unwrap_or(now_unix_ms);
        let pinned = existing.as_ref().map(|record| record.pinned).unwrap_or(request.pinned) || request.pinned;
        let record = ArtifactCacheRecord {
            source_key: request.source_key.clone(),
            source_url: request.source_url.clone(),
            sha256,
            size_bytes,
            immutable: request.immutable,
            pinned,
            validators: request.validators.clone(),
            created_at_unix_ms,
            last_access_unix_ms: now_unix_ms,
            last_revalidated_unix_ms: None,
        };
        self.write_record(&record)?;
        Ok(record)
    }

    fn restore_file(
        &self,
        record: &ArtifactCacheRecord,
        destination: &Path,
        full_hash_verify: bool,
    ) -> Result<bool, ArtifactCacheRepositoryError> {
        let payload = self.payload_path(&record.sha256)?;
        let metadata = match fs::metadata(&payload) { Ok(value) => value, Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false), Err(error) => return Err(io_error(error)) };
        if metadata.len() != record.size_bytes { return Ok(false); }
        if full_hash_verify && sha256_file(&payload)?.0 != record.sha256 { return Ok(false); }
        if let Some(parent) = destination.parent() { fs::create_dir_all(parent).map_err(io_error)?; }
        if destination.exists() { fs::remove_file(destination).map_err(io_error)?; }
        if fs::hard_link(&payload, destination).is_err() {
            copy_file_fsync(&payload, destination)?;
        }
        Ok(true)
    }

    fn touch(&mut self, record: &ArtifactCacheRecord, now_unix_ms: u64) -> Result<(), ArtifactCacheRepositoryError> {
        let mut updated = record.clone();
        updated.last_access_unix_ms = now_unix_ms;
        self.write_record(&updated)
    }

    fn save_record(&mut self, record: &ArtifactCacheRecord) -> Result<(), ArtifactCacheRepositoryError> { self.write_record(record) }

    fn remove(&mut self, record: &ArtifactCacheRecord) -> Result<(), ArtifactCacheRepositoryError> {
        let index = self.index_path(&record.source_key);
        if index.is_file() { fs::remove_file(index).map_err(io_error)?; }
        if !self.payload_is_referenced(&record.sha256)? {
            let payload = self.payload_path(&record.sha256)?;
            if payload.is_file() { fs::remove_file(&payload).map_err(io_error)?; }
            if let Some(parent) = payload.parent() { let _ = fs::remove_dir(parent); }
        }
        Ok(())
    }

    fn verify(&self, record: &ArtifactCacheRecord) -> Result<bool, ArtifactCacheRepositoryError> {
        let payload = self.payload_path(&record.sha256)?;
        if !payload.is_file() { return Ok(false); }
        let (sha256, size_bytes) = sha256_file(&payload)?;
        Ok(sha256 == record.sha256 && size_bytes == record.size_bytes)
    }
}

fn validate_sha256(value: &str) -> Result<(), ArtifactCacheRepositoryError> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()) {
        return Err(ArtifactCacheRepositoryError::Invalid(String::from("artifact SHA-256 is invalid")));
    }
    Ok(())
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn sha256_file(path: &Path) -> Result<(String, u64), ArtifactCacheRepositoryError> {
    let mut file = fs::File::open(path).map_err(io_error)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    let mut size = 0_u64;
    loop {
        let read = file.read(&mut buffer).map_err(io_error)?;
        if read == 0 { break; }
        hasher.update(&buffer[..read]);
        size = size.saturating_add(read as u64);
    }
    Ok((format!("{:x}", hasher.finalize()), size))
}

fn copy_file_fsync(source: &Path, destination: &Path) -> Result<(), ArtifactCacheRepositoryError> {
    let mut input = fs::File::open(source).map_err(io_error)?;
    let mut output = fs::File::create(destination).map_err(io_error)?;
    std::io::copy(&mut input, &mut output).map_err(io_error)?;
    output.flush().map_err(io_error)?;
    output.sync_all().map_err(io_error)
}

fn read_yaml<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, ArtifactCacheRepositoryError> {
    let content = fs::read_to_string(path).map_err(io_error)?;
    serde_yaml_ng::from_str(&content).map_err(|error| ArtifactCacheRepositoryError::Parse(error.to_string()))
}

fn write_yaml_atomic<T: Serialize>(path: &Path, value: &T) -> Result<(), ArtifactCacheRepositoryError> {
    if let Some(parent) = path.parent() { fs::create_dir_all(parent).map_err(io_error)?; }
    let yaml = serde_yaml_ng::to_string(value).map_err(|error| ArtifactCacheRepositoryError::Parse(error.to_string()))?;
    let nonce = format!("{}.{}", std::process::id(), SystemTime::now().duration_since(UNIX_EPOCH).map(|v| v.as_nanos()).unwrap_or(0));
    let temp = path.with_extension(format!("tmp.{nonce}"));
    let backup = path.with_extension(format!("bak.{nonce}"));
    {
        let mut file = fs::File::create(&temp).map_err(io_error)?;
        file.write_all(format!("{}{yaml}", yaml_header()).as_bytes()).map_err(io_error)?;
        file.flush().map_err(io_error)?;
        file.sync_all().map_err(io_error)?;
    }
    let had_existing = path.exists();
    if had_existing {
        fs::rename(path, &backup).map_err(io_error)?;
    }
    match fs::rename(&temp, path) {
        Ok(()) => {
            if had_existing { let _ = fs::remove_file(&backup); }
            Ok(())
        }
        Err(error) => {
            let _ = fs::remove_file(&temp);
            if had_existing && backup.exists() { let _ = fs::rename(&backup, path); }
            Err(io_error(error))
        }
    }
}

fn io_error(error: std::io::Error) -> ArtifactCacheRepositoryError { ArtifactCacheRepositoryError::Io(error.to_string()) }
