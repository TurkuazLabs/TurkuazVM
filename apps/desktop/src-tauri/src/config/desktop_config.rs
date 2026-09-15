// # 📄 Dosya Yolu: /turkuazvm/apps/desktop/src-tauri/src/config/desktop_config.rs
// # 📌 Amac: Merkezi config dosyasindan Desktop local/remote host profillerini yukler
// # 📌 Modul - Rust
// # Version: 0.36.0
// # Aciklama: Host secimi, endpoint, auth token, TLS CA, local Engine/TurkuazDisplay bootstrap ve refresh ayarlarini tanimlar
// # Bagimli Oldugu Katman: Service | Tool

use std::fmt;
use std::fs;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Deserialize;
use turkuazvm_transport::tools::tls_stream_tool::TlsClientSettings;

const CONFIG_ENV: &str = "TURKUAZVM_CONFIG";
const CONFIG_RELATIVE_PATH: &str = "config/turkuazvm.yml";
const MIN_REMOTE_TOKEN_LENGTH: usize = 32;
const MIN_LONG_REQUEST_TIMEOUT_MS: u64 = 30_000;
const EXPECTED_CONFIG_SCHEMA_VERSION: u16 = 22;

#[derive(Debug, Clone)]
pub struct DesktopConfig {
    pub config_path: PathBuf,
    pub project_root: PathBuf,
    pub active_host_id: String,
    pub hosts: Vec<DesktopHostProfile>,
    pub display_executable: PathBuf,
    pub refresh_interval: Duration,
}

#[derive(Debug, Clone)]
pub struct DesktopHostProfile {
    pub id: String,
    pub label: String,
    pub mode: DesktopHostMode,
    pub endpoint: SocketAddr,
    pub engine_executable: Option<PathBuf>,
    pub auto_start_engine: bool,
    pub startup_timeout: Duration,
    pub startup_poll_interval: Duration,
    pub request_timeout: Duration,
    pub long_request_timeout: Duration,
    pub auth_token: Option<String>,
    pub tls: Option<TlsClientSettings>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopHostMode {
    Local,
    Remote,
}

#[derive(Debug)]
pub enum DesktopConfigError {
    NotFound,
    Read(String),
    Parse(String),
    Invalid(String),
}

impl fmt::Display for DesktopConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(formatter, "config file not found"),
            Self::Read(message) => write!(formatter, "config read failed: {message}"),
            Self::Parse(message) => write!(formatter, "config parse failed: {message}"),
            Self::Invalid(message) => write!(formatter, "config invalid: {message}"),
        }
    }
}

impl std::error::Error for DesktopConfigError {}

#[derive(Debug, Deserialize)]
struct RootConfig {
    schema_version: u16,
    desktop: DesktopSection,
}

#[derive(Debug, Deserialize)]
struct DesktopSection {
    display_executable_path: String,
    refresh_interval_ms: u64,
    active_host: String,
    hosts: Vec<DesktopHostSection>,
}

#[derive(Debug, Deserialize)]
struct DesktopHostSection {
    id: String,
    label: String,
    mode: String,
    host: String,
    port: u16,
    auto_start: bool,
    executable_path: String,
    startup_timeout_ms: u64,
    startup_poll_interval_ms: u64,
    request_timeout_ms: u64,
    long_request_timeout_ms: u64,
    authentication: DesktopAuthenticationSection,
    tls: DesktopTlsSection,
}

#[derive(Debug, Deserialize)]
struct DesktopAuthenticationSection {
    mode: String,
    token_file: String,
}

#[derive(Debug, Deserialize)]
struct DesktopTlsSection {
    enabled: bool,
    ca_certificate_path: String,
    server_name: String,
}

impl DesktopConfig {
    pub fn load() -> Result<Self, DesktopConfigError> {
        let config_path = locate_config().ok_or(DesktopConfigError::NotFound)?;
        let content = fs::read_to_string(&config_path)
            .map_err(|error| DesktopConfigError::Read(error.to_string()))?;
        let config: RootConfig = serde_yaml_ng::from_str(&content)
            .map_err(|error| DesktopConfigError::Parse(error.to_string()))?;
        if config.schema_version != EXPECTED_CONFIG_SCHEMA_VERSION {
            return Err(DesktopConfigError::Invalid(format!(
                "unsupported config schema {}; expected {}",
                config.schema_version, EXPECTED_CONFIG_SCHEMA_VERSION
            )));
        }
        let project_root = config_path
            .parent()
            .and_then(Path::parent)
            .ok_or_else(|| DesktopConfigError::Invalid(String::from("Invalid config path")))?
            .to_path_buf();
        if config.desktop.hosts.is_empty() {
            return Err(DesktopConfigError::Invalid(String::from(
                "desktop.hosts must contain at least one host profile",
            )));
        }

        let mut hosts = Vec::with_capacity(config.desktop.hosts.len());
        for host in config.desktop.hosts {
            hosts.push(parse_host_profile(host, &project_root)?);
        }
        if !hosts.iter().any(|host| host.id == config.desktop.active_host) {
            return Err(DesktopConfigError::Invalid(format!(
                "desktop.active_host not found: {}",
                config.desktop.active_host
            )));
        }
        if hosts.iter().enumerate().any(|(index, host)| {
            hosts
                .iter()
                .skip(index + 1)
                .any(|other| other.id == host.id)
        }) {
            return Err(DesktopConfigError::Invalid(String::from(
                "desktop.hosts contains duplicate id",
            )));
        }

        let display_executable = non_empty_path(&config.desktop.display_executable_path)
            .map(|value| resolve_path(&project_root, value))
            .ok_or_else(|| DesktopConfigError::Invalid(String::from(
                "desktop.display_executable_path must not be empty",
            )))?;

        Ok(Self {
            config_path,
            project_root,
            active_host_id: config.desktop.active_host,
            hosts,
            display_executable,
            refresh_interval: Duration::from_millis(config.desktop.refresh_interval_ms),
        })
    }
}

fn parse_host_profile(
    config: DesktopHostSection,
    project_root: &Path,
) -> Result<DesktopHostProfile, DesktopConfigError> {
    validate_host_id(&config.id)?;
    if config.label.trim().is_empty() {
        return Err(DesktopConfigError::Invalid(String::from(
            "desktop host label must not be empty",
        )));
    }
    let ip = config.host.parse::<IpAddr>().map_err(|_| {
        DesktopConfigError::Invalid(format!("Invalid desktop host IP: {}", config.host))
    })?;
    let mode = match config.mode.as_str() {
        "local" => DesktopHostMode::Local,
        "remote" => DesktopHostMode::Remote,
        other => {
            return Err(DesktopConfigError::Invalid(format!(
                "Unsupported desktop host mode: {other}"
            )))
        }
    };
    if mode == DesktopHostMode::Local && !ip.is_loopback() {
        return Err(DesktopConfigError::Invalid(format!(
            "Local host profile {} must use loopback",
            config.id
        )));
    }
    let auth_token = load_auth_token(&config.authentication, project_root)?;
    let tls = load_tls_settings(&config.tls, project_root)?;
    if mode == DesktopHostMode::Remote && (auth_token.is_none() || tls.is_none()) {
        return Err(DesktopConfigError::Invalid(format!(
            "Remote host profile {} requires TLS and token authentication",
            config.id
        )));
    }
    let executable = non_empty_path(&config.executable_path)
        .map(|value| resolve_path(project_root, value));
    if mode == DesktopHostMode::Local && config.auto_start && executable.is_none() {
        return Err(DesktopConfigError::Invalid(format!(
            "Local auto-start host {} requires executable_path",
            config.id
        )));
    }
    if config.long_request_timeout_ms < MIN_LONG_REQUEST_TIMEOUT_MS {
        return Err(DesktopConfigError::Invalid(format!(
            "desktop host {} long_request_timeout_ms must be at least {MIN_LONG_REQUEST_TIMEOUT_MS}",
            config.id
        )));
    }
    if config.long_request_timeout_ms < config.request_timeout_ms {
        return Err(DesktopConfigError::Invalid(format!(
            "desktop host {} long_request_timeout_ms must be >= request_timeout_ms",
            config.id
        )));
    }

    Ok(DesktopHostProfile {
        id: config.id,
        label: config.label,
        mode,
        endpoint: SocketAddr::new(ip, config.port),
        engine_executable: executable,
        auto_start_engine: mode == DesktopHostMode::Local && config.auto_start,
        startup_timeout: Duration::from_millis(config.startup_timeout_ms),
        startup_poll_interval: Duration::from_millis(config.startup_poll_interval_ms),
        request_timeout: Duration::from_millis(config.request_timeout_ms),
        long_request_timeout: Duration::from_millis(config.long_request_timeout_ms),
        auth_token,
        tls,
    })
}

fn load_auth_token(
    config: &DesktopAuthenticationSection,
    project_root: &Path,
) -> Result<Option<String>, DesktopConfigError> {
    match config.mode.as_str() {
        "none" => Ok(None),
        "token" => {
            let path = non_empty_path(&config.token_file).ok_or_else(|| {
                DesktopConfigError::Invalid(String::from(
                    "Token authentication requires token_file",
                ))
            })?;
            let token = fs::read_to_string(resolve_path(project_root, path))
                .map_err(|error| DesktopConfigError::Read(error.to_string()))?
                .trim()
                .to_owned();
            if token.len() < MIN_REMOTE_TOKEN_LENGTH {
                return Err(DesktopConfigError::Invalid(format!(
                    "Remote authentication token must be at least {MIN_REMOTE_TOKEN_LENGTH} characters"
                )));
            }
            Ok(Some(token))
        }
        other => Err(DesktopConfigError::Invalid(format!(
            "Unsupported desktop authentication mode: {other}"
        ))),
    }
}

fn load_tls_settings(
    config: &DesktopTlsSection,
    project_root: &Path,
) -> Result<Option<TlsClientSettings>, DesktopConfigError> {
    if !config.enabled {
        return Ok(None);
    }
    let ca_certificate = non_empty_path(&config.ca_certificate_path).ok_or_else(|| {
        DesktopConfigError::Invalid(String::from("TLS requires ca_certificate_path"))
    })?;
    if config.server_name.trim().is_empty() {
        return Err(DesktopConfigError::Invalid(String::from(
            "TLS requires server_name",
        )));
    }
    Ok(Some(TlsClientSettings {
        ca_certificate_path: resolve_path(project_root, ca_certificate),
        server_name: config.server_name.trim().to_owned(),
    }))
}

fn validate_host_id(value: &str) -> Result<(), DesktopConfigError> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(DesktopConfigError::Invalid(format!(
            "Invalid desktop host id: {value}"
        )));
    }
    Ok(())
}

fn locate_config() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os(CONFIG_ENV).map(PathBuf::from) {
        if path.is_file() {
            return Some(path);
        }
    }

    let current = std::env::current_dir().ok()?;
    for directory in current.ancestors() {
        let candidate = directory.join(CONFIG_RELATIVE_PATH);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn non_empty_path(value: &str) -> Option<PathBuf> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(PathBuf::from(trimmed))
    }
}

fn resolve_path(root: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        root.join(path)
    }
}
