// # 📄 Dosya Yolu: /turkuazvm/crates/repositories/src/repositories/yaml_runtime_registry_repository.rs
// # 📌 Amac: QEMU runtime registration kayitlarini atomik YAML dosyalariyla saklar ve bozuk kayitlari quarantine eder
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: PID/start-token/executable/QMP/display state persistence, atomic replace ve corrupt-record isolation uygular
// # Bagimli Oldugu Katman: Repo

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use turkuazvm_core::domain::display::{DisplayCapabilities, DisplayRuntimeInfo, DisplayTransport};
use turkuazvm_core::domain::runtime_registration::RuntimeRegistration;
use turkuazvm_core::domain::virtual_machine::VmId;
use turkuazvm_core::ports::runtime_registry_port::{RuntimeRegistryError, RuntimeRegistryPort};

const SCHEMA_VERSION: u16 = 1;
const RUNTIME_REGISTRY_DIR: &str = "runtime-registry";
const YAML_EXTENSION: &str = "yml";

#[derive(Debug, Clone)]
pub struct YamlRuntimeRegistryRepository {
    root: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct RuntimeRegistrationDocument {
    schema_version: u16,
    vm_id: String,
    process_id: u32,
    process_start_token: String,
    executable_path: String,
    qmp_endpoint: String,
    display: Option<DisplayDocument>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DisplayDocument {
    transport: String,
    endpoint: String,
    local_only: bool,
    native_window: bool,
    fullscreen: bool,
    absolute_pointer: bool,
    relative_pointer_capture: bool,
    keyboard: bool,
    gamepad_observation: bool,
}

impl YamlRuntimeRegistryRepository {
    pub fn new(data_root: PathBuf) -> Self {
        Self { root: data_root.join(RUNTIME_REGISTRY_DIR) }
    }

    fn path_for(&self, vm_id: &VmId) -> PathBuf {
        self.root.join(vm_id.as_str()).with_extension(YAML_EXTENSION)
    }

    fn quarantine(path: &Path) -> Result<(), RuntimeRegistryError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| RuntimeRegistryError::Io(error.to_string()))?
            .as_millis();
        let bad = PathBuf::from(format!("{}.bad.{timestamp}", path.display()));
        fs::rename(path, bad).map_err(|error| RuntimeRegistryError::Io(error.to_string()))
    }

    fn decode(path: &Path) -> Result<RuntimeRegistration, RuntimeRegistryError> {
        let text = fs::read_to_string(path).map_err(|error| RuntimeRegistryError::Io(error.to_string()))?;
        let document: RuntimeRegistrationDocument = serde_yaml_ng::from_str(&text)
            .map_err(|error| RuntimeRegistryError::Parse(error.to_string()))?;
        if document.schema_version != SCHEMA_VERSION {
            return Err(RuntimeRegistryError::Invalid(format!(
                "unsupported runtime registry schema {}",
                document.schema_version
            )));
        }
        let vm_id = VmId::parse(document.vm_id)
            .map_err(|error| RuntimeRegistryError::Invalid(format!("{error:?}")))?;
        let qmp_endpoint = document.qmp_endpoint.parse::<SocketAddr>()
            .map_err(|error| RuntimeRegistryError::Invalid(error.to_string()))?;
        let display = document.display.map(decode_display).transpose()?;
        Ok(RuntimeRegistration {
            vm_id,
            process_id: document.process_id,
            process_start_token: document.process_start_token,
            executable_path: PathBuf::from(document.executable_path),
            qmp_endpoint,
            display,
        })
    }
}

impl RuntimeRegistryPort for YamlRuntimeRegistryRepository {
    fn list(&self) -> Result<Vec<RuntimeRegistration>, RuntimeRegistryError> {
        if !self.root.exists() {
            return Ok(Vec::new());
        }
        let mut registrations = Vec::new();
        for entry in fs::read_dir(&self.root).map_err(|error| RuntimeRegistryError::Io(error.to_string()))? {
            let entry = entry.map_err(|error| RuntimeRegistryError::Io(error.to_string()))?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some(YAML_EXTENSION) {
                continue;
            }
            match Self::decode(&path) {
                Ok(registration) => registrations.push(registration),
                Err(RuntimeRegistryError::Parse(_)) => {
                    Self::quarantine(&path)?;
                }
                Err(RuntimeRegistryError::Invalid(message)) if message.starts_with("unsupported runtime registry schema") => {
                    return Err(RuntimeRegistryError::Invalid(message));
                }
                Err(_) => {
                    Self::quarantine(&path)?;
                }
            }
        }
        Ok(registrations)
    }

    fn save(&self, registration: &RuntimeRegistration) -> Result<(), RuntimeRegistryError> {
        fs::create_dir_all(&self.root).map_err(|error| RuntimeRegistryError::Io(error.to_string()))?;
        let path = self.path_for(&registration.vm_id);
        let temp = path.with_extension("yml.tmp");
        let document = RuntimeRegistrationDocument {
            schema_version: SCHEMA_VERSION,
            vm_id: registration.vm_id.as_str().to_owned(),
            process_id: registration.process_id,
            process_start_token: registration.process_start_token.clone(),
            executable_path: registration.executable_path.to_string_lossy().into_owned(),
            qmp_endpoint: registration.qmp_endpoint.to_string(),
            display: registration.display.map(encode_display),
        };
        let text = serde_yaml_ng::to_string(&document)
            .map_err(|error| RuntimeRegistryError::Parse(error.to_string()))?;
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temp)
            .map_err(|error| RuntimeRegistryError::Io(error.to_string()))?;
        file.write_all(text.as_bytes())
            .and_then(|()| file.sync_all())
            .map_err(|error| RuntimeRegistryError::Io(error.to_string()))?;
        fs::rename(temp, path).map_err(|error| RuntimeRegistryError::Io(error.to_string()))
    }

    fn remove(&self, vm_id: &VmId) -> Result<(), RuntimeRegistryError> {
        let path = self.path_for(vm_id);
        if path.exists() {
            fs::remove_file(path).map_err(|error| RuntimeRegistryError::Io(error.to_string()))?;
        }
        Ok(())
    }
}

fn encode_display(value: DisplayRuntimeInfo) -> DisplayDocument {
    DisplayDocument {
        transport: match value.transport { DisplayTransport::Rfb => String::from("rfb") },
        endpoint: value.endpoint.to_string(),
        local_only: value.local_only,
        native_window: value.capabilities.native_window,
        fullscreen: value.capabilities.fullscreen,
        absolute_pointer: value.capabilities.absolute_pointer,
        relative_pointer_capture: value.capabilities.relative_pointer_capture,
        keyboard: value.capabilities.keyboard,
        gamepad_observation: value.capabilities.gamepad_observation,
    }
}

fn decode_display(value: DisplayDocument) -> Result<DisplayRuntimeInfo, RuntimeRegistryError> {
    let transport = match value.transport.as_str() {
        "rfb" => DisplayTransport::Rfb,
        other => return Err(RuntimeRegistryError::Invalid(format!("unsupported display transport {other}"))),
    };
    let endpoint = value.endpoint.parse::<SocketAddr>()
        .map_err(|error| RuntimeRegistryError::Invalid(error.to_string()))?;
    Ok(DisplayRuntimeInfo {
        transport,
        endpoint,
        local_only: value.local_only,
        capabilities: DisplayCapabilities {
            native_window: value.native_window,
            fullscreen: value.fullscreen,
            absolute_pointer: value.absolute_pointer,
            relative_pointer_capture: value.relative_pointer_capture,
            keyboard: value.keyboard,
            gamepad_observation: value.gamepad_observation,
        },
    })
}
