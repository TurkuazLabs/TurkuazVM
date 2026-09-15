// # 📄 Dosya Yolu: /turkuazvm/crates/guest/src/tools/android_guest_agent_tool.rs
// # 📌 Amac: Turkuaz Android Guest Agent portunu ADB provisioning ve TVGB v2 authenticated TCP transportu ile uygular
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: TVGB v2, per-VM secret, Ed25519 signed update, rollback, touch ve clipboard akisini yonetir
// # Bagimli Oldugu Katman: Tool

use std::collections::HashMap;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use ed25519_dalek::{Signature, VerifyingKey};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use turkuazvm_android::domain::guest_agent::{
    AndroidGuestAgentReport, AndroidTouchContact, AndroidTouchPhase,
};
use turkuazvm_android::domain::runtime_profile::AndroidRuntimeProfile;
use turkuazvm_android::ports::android_guest_agent_port::{
    AndroidGuestAgentPort, AndroidGuestAgentPortError,
};
use turkuazvm_guest_agent_protocol::{
    client_handshake, read_frame, write_frame, GuestAgentAction, GuestAgentRequest,
    GuestAgentResponse, GuestAgentResponseData, GuestTouchContact, GuestTouchPhase, ReplayWindow,
    TvgbDirection, TvgbMessageType, TvgbSessionKeys, GUEST_AGENT_PROTOCOL_VERSION,
    TVGB_ROOT_KEY_LEN,
};

const COMMAND_ADB_WINDOWS: &str = "adb.exe";
const COMMAND_ADB_UNIX: &str = "adb";
const ENV_ANDROID_SDK_ROOT: &str = "ANDROID_SDK_ROOT";
const ENV_ANDROID_HOME: &str = "ANDROID_HOME";
const PLATFORM_TOOLS_DIR: &str = "platform-tools";
const AGENT_PACKAGE: &str = "com.turkuazvm.inputagent";
const AGENT_ACCESSIBILITY_COMPONENT: &str =
    "com.turkuazvm.inputagent/com.turkuazvm.inputagent.services.TurkuazInputAccessibilityService";
const SHELL_COMMAND_PM: &str = "pm";
const SHELL_COMMAND_SETTINGS: &str = "settings";
const PM_PATH: &str = "path";
const SETTINGS_GET: &str = "get";
const SETTINGS_PUT: &str = "put";
const SETTINGS_SECURE: &str = "secure";
const SETTING_ENABLED_ACCESSIBILITY_SERVICES: &str = "enabled_accessibility_services";
const SETTING_ACCESSIBILITY_ENABLED: &str = "accessibility_enabled";
const SETTINGS_TRUE: &str = "1";
const SETTINGS_NULL: &str = "null";
const ACCESSIBILITY_SERVICE_SEPARATOR: char = ':';
const HOST_SECRET_EXTENSION: &str = "key";
const GUEST_SECRET_TEMP: &str = "/data/local/tmp/turkuazvm-tvgb-v2.key";
const GUEST_SECRET_TARGET: &str =
    "/data/user_de/0/com.turkuazvm.inputagent/files/tvgb-v2.key";
const GUEST_SECRET_DIRECTORY: &str = "/data/user_de/0/com.turkuazvm.inputagent/files";
const SESSION_RETRY_COUNT: usize = 2;
const SIGNED_AGENT_MANIFEST_SCHEMA_VERSION: u16 = 2;
const SIGNED_AGENT_SIGNATURE_HEX_LEN: usize = 128;
const SIGNED_AGENT_PUBLIC_KEY_HEX_LEN: usize = 64;
const SHA256_HEX_LEN: usize = 64;
const ROLLBACK_APK_FILE: &str = "previous.apk";
const ROLLBACK_SHA256_FILE: &str = "previous.apk.sha256";
const APK_CERT_DIGEST_PREFIX: &str = "Signer #1 certificate SHA-256 digest:";
const COMMAND_APKSIGNER_WINDOWS: &str = "apksigner.bat";
const COMMAND_APKSIGNER_UNIX: &str = "apksigner";
const BUILD_TOOLS_DIR: &str = "build-tools";

#[derive(Debug, Clone)]
pub struct AndroidGuestAgentSettings {
    pub explicit_adb_binary: Option<PathBuf>,
    pub host_ip: Ipv4Addr,
    pub host_port_min: u16,
    pub host_port_max: u16,
    pub guest_port: u16,
    pub command_timeout: Duration,
    pub connect_timeout: Duration,
    pub io_timeout: Duration,
    pub ready_timeout: Duration,
    pub ready_poll_interval: Duration,
    pub secret_root: PathBuf,
    pub require_adb_root_for_provisioning: bool,
    pub update: AndroidGuestAgentUpdateSettings,
}

#[derive(Debug, Clone)]
pub struct AndroidGuestAgentUpdateSettings {
    pub enabled: bool,
    pub manifest_path: PathBuf,
    pub trusted_public_key_hex: String,
    pub trusted_apk_cert_sha256: String,
    pub apksigner_binary: Option<PathBuf>,
    pub rollback_root: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
struct SignedAgentManifest {
    schema_version: u16,
    package_name: String,
    version_code: u64,
    apk_file: String,
    sha256: String,
    apk_signing_cert_sha256: String,
    signature_hex: String,
}

#[derive(Debug, Clone)]
struct RollbackArtifact {
    apk_path: PathBuf,
    sha256_path: PathBuf,
    expected_sha256: String,
}

#[derive(Debug, Clone)]
struct AgentUpdateTransaction {
    rollback: Option<RollbackArtifact>,
}

#[derive(Debug, Clone)]
struct ForwardLease {
    port: u16,
    serial: String,
}

struct AgentSession {
    stream: TcpStream,
    keys: TvgbSessionKeys,
    incoming_replay: ReplayWindow,
    next_outgoing_sequence: u64,
}

impl AgentSession {
    fn connect(
        endpoint: SocketAddrV4,
        connect_timeout: Duration,
        io_timeout: Duration,
        root_key: &[u8; TVGB_ROOT_KEY_LEN],
    ) -> Result<Self, AndroidGuestAgentPortError> {
        let mut stream = TcpStream::connect_timeout(&endpoint.into(), connect_timeout)
            .map_err(|error| AndroidGuestAgentPortError::ConnectFailed(error.to_string()))?;
        stream
            .set_read_timeout(Some(io_timeout))
            .map_err(|error| AndroidGuestAgentPortError::Protocol(error.to_string()))?;
        stream
            .set_write_timeout(Some(io_timeout))
            .map_err(|error| AndroidGuestAgentPortError::Protocol(error.to_string()))?;
        let keys = client_handshake(&mut stream, root_key)
            .map_err(|error| AndroidGuestAgentPortError::Protocol(error.to_string()))?;
        Ok(Self {
            stream,
            keys,
            incoming_replay: ReplayWindow::default(),
            next_outgoing_sequence: 1,
        })
    }

    fn request(
        &mut self,
        request: &GuestAgentRequest,
    ) -> Result<GuestAgentResponse, AndroidGuestAgentPortError> {
        let payload = serde_json::to_vec(request)
            .map_err(|error| AndroidGuestAgentPortError::Protocol(error.to_string()))?;
        let sequence = self.next_outgoing_sequence;
        self.next_outgoing_sequence = self.next_outgoing_sequence.wrapping_add(1).max(1);
        write_frame(
            &mut self.stream,
            &self.keys,
            TvgbDirection::HostToGuest,
            TvgbMessageType::Request,
            sequence,
            request.request_id,
            &payload,
        )
        .map_err(|error| AndroidGuestAgentPortError::Protocol(error.to_string()))?;
        let frame = read_frame(
            &mut self.stream,
            &self.keys,
            TvgbDirection::GuestToHost,
            &mut self.incoming_replay,
        )
        .map_err(|error| AndroidGuestAgentPortError::Protocol(error.to_string()))?;
        if frame.message_type != TvgbMessageType::Response || frame.request_id != request.request_id {
            return Err(AndroidGuestAgentPortError::Protocol(String::from(
                "TVGB response correlation failed",
            )));
        }
        serde_json::from_slice(&frame.payload)
            .map_err(|error| AndroidGuestAgentPortError::Protocol(error.to_string()))
    }
}

pub struct AndroidGuestAgentTool {
    binary: Option<PathBuf>,
    settings: AndroidGuestAgentSettings,
    forwards: HashMap<String, ForwardLease>,
    sessions: HashMap<String, AgentSession>,
    next_request_id: u64,
}

impl AndroidGuestAgentTool {
    pub fn new(settings: AndroidGuestAgentSettings) -> Self {
        let binary = discover_adb_binary(settings.explicit_adb_binary.as_deref());
        Self {
            binary,
            settings,
            forwards: HashMap::new(),
            sessions: HashMap::new(),
            next_request_id: 1,
        }
    }

    fn ensure_forward(
        &mut self,
        profile: &AndroidRuntimeProfile,
    ) -> Result<u16, AndroidGuestAgentPortError> {
        if let Some(lease) = self.forwards.get(profile.vm_id().as_str()) {
            return Ok(lease.port);
        }
        let binary = self
            .binary
            .as_deref()
            .ok_or(AndroidGuestAgentPortError::BridgeUnavailable)?;
        let port = self.allocate_port()?;
        let serial = profile.adb_serial();
        let host_arg = format!("tcp:{port}");
        let guest_arg = format!("tcp:{}", self.settings.guest_port);
        let output = run_command(
            binary,
            &[
                "-s",
                serial.as_str(),
                "forward",
                host_arg.as_str(),
                guest_arg.as_str(),
            ],
            self.settings.command_timeout,
        )?;
        if !output.0 {
            return Err(AndroidGuestAgentPortError::ForwardFailed(output.1));
        }
        self.forwards
            .insert(profile.vm_id().as_str().to_owned(), ForwardLease { port, serial });
        Ok(port)
    }

    fn allocate_port(&self) -> Result<u16, AndroidGuestAgentPortError> {
        for port in self.settings.host_port_min..=self.settings.host_port_max {
            if self.forwards.values().any(|value| value.port == port) {
                continue;
            }
            if TcpListener::bind(SocketAddrV4::new(self.settings.host_ip, port)).is_ok() {
                return Ok(port);
            }
        }
        Err(AndroidGuestAgentPortError::PortExhausted)
    }

    fn remove_forward(&self, lease: &ForwardLease) {
        let Some(binary) = self.binary.as_deref() else {
            return;
        };
        let host_arg = format!("tcp:{}", lease.port);
        let _ = run_command(
            binary,
            &[
                "-s",
                lease.serial.as_str(),
                "forward",
                "--remove",
                host_arg.as_str(),
            ],
            self.settings.command_timeout,
        );
    }

    fn release_transport_for_vm(&mut self, vm_id: &str) {
        self.sessions.remove(vm_id);
        if let Some(lease) = self.forwards.remove(vm_id) {
            self.remove_forward(&lease);
        }
    }

    fn adb_shell(
        &self,
        profile: &AndroidRuntimeProfile,
        args: &[&str],
    ) -> Result<String, AndroidGuestAgentPortError> {
        let binary = self
            .binary
            .as_deref()
            .ok_or(AndroidGuestAgentPortError::BridgeUnavailable)?;
        let serial = profile.adb_serial();
        let mut command_args = vec!["-s", serial.as_str(), "shell"];
        command_args.extend_from_slice(args);
        let output = run_command(binary, &command_args, self.settings.command_timeout)?;
        if output.0 {
            Ok(output.1)
        } else {
            Err(AndroidGuestAgentPortError::ProvisionFailed(output.1))
        }
    }

    fn apply_signed_update_if_configured(
        &mut self,
        profile: &AndroidRuntimeProfile,
    ) -> Result<Option<AgentUpdateTransaction>, AndroidGuestAgentPortError> {
        if !self.settings.update.enabled {
            return Ok(None);
        }
        let (manifest, apk_path) = self.load_and_verify_signed_manifest()?;
        let installed_version = self.installed_agent_version(profile)?;
        if installed_version.is_some_and(|version| version >= manifest.version_code) {
            return Ok(None);
        }

        let rollback = self.stage_rollback_apk(profile, &manifest.apk_signing_cert_sha256)?;
        if let Err(error) = self.install_agent_apk(profile, &apk_path, false) {
            if let Err(rollback_error) = self.rollback_agent_update(profile, rollback.as_ref()) {
                return Err(AndroidGuestAgentPortError::ProvisionFailed(format!(
                    "signed Guest Agent install failed: {error:?}; rollback failed: {rollback_error:?}"
                )));
            }
            return Err(error);
        }
        self.release_transport_for_vm(profile.vm_id().as_str());

        match self.installed_agent_version(profile) {
            Ok(Some(version)) if version == manifest.version_code => Ok(Some(AgentUpdateTransaction { rollback })),
            Ok(observed) => {
                let message = format!(
                    "signed Guest Agent post-install version mismatch: expected={} observed={observed:?}",
                    manifest.version_code
                );
                if let Err(rollback_error) = self.rollback_agent_update(profile, rollback.as_ref()) {
                    return Err(AndroidGuestAgentPortError::ProvisionFailed(format!(
                        "{message}; rollback failed: {rollback_error:?}"
                    )));
                }
                Err(AndroidGuestAgentPortError::ProvisionFailed(message))
            }
            Err(error) => {
                if let Err(rollback_error) = self.rollback_agent_update(profile, rollback.as_ref()) {
                    return Err(AndroidGuestAgentPortError::ProvisionFailed(format!(
                        "signed Guest Agent version probe failed: {error:?}; rollback failed: {rollback_error:?}"
                    )));
                }
                Err(error)
            }
        }
    }

    fn load_and_verify_signed_manifest(
        &self,
    ) -> Result<(SignedAgentManifest, PathBuf), AndroidGuestAgentPortError> {
        let manifest_text = fs::read_to_string(&self.settings.update.manifest_path)
            .map_err(|error| AndroidGuestAgentPortError::ProvisionFailed(format!("signed manifest read failed: {error}")))?;
        let manifest: SignedAgentManifest = serde_yaml_ng::from_str(&manifest_text)
            .map_err(|error| AndroidGuestAgentPortError::ProvisionFailed(format!("signed manifest parse failed: {error}")))?;
        if manifest.schema_version != SIGNED_AGENT_MANIFEST_SCHEMA_VERSION {
            return Err(AndroidGuestAgentPortError::ProvisionFailed(String::from(
                "signed Guest Agent manifest schema is unsupported",
            )));
        }
        if manifest.package_name != AGENT_PACKAGE || manifest.version_code == 0 {
            return Err(AndroidGuestAgentPortError::ProvisionFailed(String::from(
                "signed Guest Agent manifest identity is invalid",
            )));
        }
        if !is_simple_apk_file_name(&manifest.apk_file)
            || !is_lower_hex(&manifest.sha256, SHA256_HEX_LEN)
            || !is_lower_hex(&manifest.apk_signing_cert_sha256, SHA256_HEX_LEN)
            || !is_lower_hex(&manifest.signature_hex, SIGNED_AGENT_SIGNATURE_HEX_LEN)
            || !is_lower_hex(&self.settings.update.trusted_public_key_hex, SIGNED_AGENT_PUBLIC_KEY_HEX_LEN)
            || !is_lower_hex(&self.settings.update.trusted_apk_cert_sha256, SHA256_HEX_LEN)
        {
            return Err(AndroidGuestAgentPortError::ProvisionFailed(String::from(
                "signed Guest Agent manifest cryptographic fields are invalid",
            )));
        }
        let manifest_root = self.settings.update.manifest_path.parent().ok_or_else(|| {
            AndroidGuestAgentPortError::ProvisionFailed(String::from("signed Guest Agent manifest parent is missing"))
        })?;
        let apk_path = manifest_root.join(&manifest.apk_file);
        if !apk_path.is_file() {
            return Err(AndroidGuestAgentPortError::ProvisionFailed(String::from(
                "signed Guest Agent APK is missing",
            )));
        }
        let observed_sha256 = sha256_file_hex(&apk_path)?;
        if observed_sha256 != manifest.sha256 {
            return Err(AndroidGuestAgentPortError::ProvisionFailed(String::from(
                "signed Guest Agent APK SHA-256 mismatch",
            )));
        }
        if manifest.apk_signing_cert_sha256 != self.settings.update.trusted_apk_cert_sha256 {
            return Err(AndroidGuestAgentPortError::ProvisionFailed(String::from(
                "signed Guest Agent APK certificate does not match pinned certificate",
            )));
        }
        let observed_cert = self.apk_signing_cert_sha256(&apk_path)?;
        if observed_cert != manifest.apk_signing_cert_sha256 {
            return Err(AndroidGuestAgentPortError::ProvisionFailed(String::from(
                "signed Guest Agent APK certificate digest mismatch",
            )));
        }

        let public_key_bytes = decode_hex_array::<32>(&self.settings.update.trusted_public_key_hex)?;
        let signature_bytes = decode_hex_array::<64>(&manifest.signature_hex)?;
        let verifying_key = VerifyingKey::from_bytes(&public_key_bytes)
            .map_err(|error| AndroidGuestAgentPortError::ProvisionFailed(format!("Ed25519 public key invalid: {error}")))?;
        let signature = Signature::from_bytes(&signature_bytes);
        let signed_payload = signed_manifest_payload(&manifest);
        verifying_key
            .verify_strict(signed_payload.as_bytes(), &signature)
            .map_err(|_| AndroidGuestAgentPortError::ProvisionFailed(String::from("signed Guest Agent Ed25519 verification failed")))?;
        Ok((manifest, apk_path))
    }

    fn installed_agent_version(
        &self,
        profile: &AndroidRuntimeProfile,
    ) -> Result<Option<u64>, AndroidGuestAgentPortError> {
        let output = self.adb_shell(profile, &["dumpsys", "package", AGENT_PACKAGE])?;
        Ok(parse_version_code(&output))
    }

    fn stage_rollback_apk(
        &self,
        profile: &AndroidRuntimeProfile,
        expected_cert_sha256: &str,
    ) -> Result<Option<RollbackArtifact>, AndroidGuestAgentPortError> {
        let package_path = self.adb_shell(profile, &[SHELL_COMMAND_PM, PM_PATH, AGENT_PACKAGE])?;
        let Some(remote_apk) = package_path
            .lines()
            .filter_map(|line| line.trim().strip_prefix("package:"))
            .find(|path| path.ends_with("base.apk"))
            .or_else(|| package_path.lines().filter_map(|line| line.trim().strip_prefix("package:")).next())
        else {
            return Ok(None);
        };
        fs::create_dir_all(&self.settings.update.rollback_root)
            .map_err(|error| AndroidGuestAgentPortError::ProvisionFailed(format!("rollback root create failed: {error}")))?;
        set_private_directory_permissions(&self.settings.update.rollback_root)
            .map_err(|error| AndroidGuestAgentPortError::ProvisionFailed(format!("rollback root ACL failed: {error}")))?;
        let rollback_dir = self.settings.update.rollback_root.join(profile.vm_id().as_str());
        fs::create_dir_all(&rollback_dir)
            .map_err(|error| AndroidGuestAgentPortError::ProvisionFailed(format!("rollback directory create failed: {error}")))?;
        set_private_directory_permissions(&rollback_dir)
            .map_err(|error| AndroidGuestAgentPortError::ProvisionFailed(format!("rollback directory ACL failed: {error}")))?;
        let rollback_apk = rollback_dir.join(ROLLBACK_APK_FILE);
        let rollback_sha256 = rollback_dir.join(ROLLBACK_SHA256_FILE);
        let binary = self.binary.as_deref().ok_or(AndroidGuestAgentPortError::BridgeUnavailable)?;
        let serial = profile.adb_serial();
        let local = rollback_apk.to_string_lossy().into_owned();
        let pull = run_command(
            binary,
            &["-s", serial.as_str(), "pull", remote_apk, local.as_str()],
            self.settings.command_timeout,
        )?;
        if !pull.0 || !rollback_apk.is_file() {
            return Err(AndroidGuestAgentPortError::ProvisionFailed(format!(
                "Guest Agent rollback staging failed: {}",
                pull.1
            )));
        }
        set_private_permissions(&rollback_apk)
            .map_err(|error| AndroidGuestAgentPortError::ProvisionFailed(format!("rollback APK ACL failed: {error}")))?;
        let expected_sha256 = sha256_file_hex(&rollback_apk)?;
        fs::write(&rollback_sha256, format!("{expected_sha256}\n"))
            .map_err(|error| AndroidGuestAgentPortError::ProvisionFailed(format!("rollback hash write failed: {error}")))?;
        set_private_permissions(&rollback_sha256)
            .map_err(|error| AndroidGuestAgentPortError::ProvisionFailed(format!("rollback hash ACL failed: {error}")))?;
        let rollback_cert = self.apk_signing_cert_sha256(&rollback_apk)?;
        if rollback_cert != expected_cert_sha256 {
            return Err(AndroidGuestAgentPortError::ProvisionFailed(String::from(
                "installed Guest Agent certificate differs from signed update certificate",
            )));
        }
        Ok(Some(RollbackArtifact {
            apk_path: rollback_apk,
            sha256_path: rollback_sha256,
            expected_sha256,
        }))
    }

    fn install_agent_apk(
        &self,
        profile: &AndroidRuntimeProfile,
        apk_path: &Path,
        allow_downgrade: bool,
    ) -> Result<(), AndroidGuestAgentPortError> {
        let binary = self.binary.as_deref().ok_or(AndroidGuestAgentPortError::BridgeUnavailable)?;
        let serial = profile.adb_serial();
        let apk = apk_path.to_string_lossy().into_owned();
        let mut args = vec!["-s", serial.as_str(), "install", "-r"];
        if allow_downgrade {
            args.push("-d");
        }
        args.push(apk.as_str());
        let output = run_command(binary, &args, self.settings.command_timeout)?;
        if output.0 && output.1.contains("Success") {
            Ok(())
        } else {
            Err(AndroidGuestAgentPortError::ProvisionFailed(format!(
                "Guest Agent APK install failed: {}",
                output.1
            )))
        }
    }

    fn rollback_agent_update(
        &mut self,
        profile: &AndroidRuntimeProfile,
        rollback: Option<&RollbackArtifact>,
    ) -> Result<(), AndroidGuestAgentPortError> {
        self.release_transport_for_vm(profile.vm_id().as_str());
        if let Some(rollback) = rollback {
            let sidecar = fs::read_to_string(&rollback.sha256_path)
                .map_err(|error| AndroidGuestAgentPortError::ProvisionFailed(format!("rollback hash read failed: {error}")))?;
            if sidecar.trim() != rollback.expected_sha256 || sha256_file_hex(&rollback.apk_path)? != rollback.expected_sha256 {
                return Err(AndroidGuestAgentPortError::ProvisionFailed(String::from(
                    "Guest Agent rollback APK integrity verification failed",
                )));
            }
            self.install_agent_apk(profile, &rollback.apk_path, true)?;
            self.ensure_agent_package(profile)
        } else {
            let binary = self.binary.as_deref().ok_or(AndroidGuestAgentPortError::BridgeUnavailable)?;
            let serial = profile.adb_serial();
            let output = run_command(
                binary,
                &["-s", serial.as_str(), "uninstall", AGENT_PACKAGE],
                self.settings.command_timeout,
            )?;
            if output.0 {
                Ok(())
            } else {
                Err(AndroidGuestAgentPortError::ProvisionFailed(format!(
                    "Guest Agent rollback uninstall failed: {}",
                    output.1
                )))
            }
        }
    }

    fn commit_agent_update(&self, transaction: &AgentUpdateTransaction) -> Result<(), AndroidGuestAgentPortError> {
        if let Some(rollback) = transaction.rollback.as_ref() {
            let _ = fs::remove_file(&rollback.apk_path);
            let _ = fs::remove_file(&rollback.sha256_path);
            if let Some(parent) = rollback.apk_path.parent() {
                let _ = fs::remove_dir(parent);
            }
        }
        Ok(())
    }

    fn apk_signing_cert_sha256(&self, apk_path: &Path) -> Result<String, AndroidGuestAgentPortError> {
        let binary = discover_apksigner_binary(self.settings.update.apksigner_binary.as_deref())
            .ok_or_else(|| AndroidGuestAgentPortError::ProvisionFailed(String::from("apksigner is required for signed Guest Agent updates")))?;
        let apk = apk_path.to_string_lossy().into_owned();
        let output = run_apksigner(&binary, &["verify", "--print-certs", apk.as_str()], self.settings.command_timeout)?;
        if !output.0 {
            return Err(AndroidGuestAgentPortError::ProvisionFailed(format!("apksigner verify failed: {}", output.1)));
        }
        output.1.lines()
            .find_map(|line| line.trim().strip_prefix(APK_CERT_DIGEST_PREFIX))
            .map(str::trim)
            .map(str::to_ascii_lowercase)
            .filter(|value| is_lower_hex(value, SHA256_HEX_LEN))
            .ok_or_else(|| AndroidGuestAgentPortError::ProvisionFailed(String::from("apksigner certificate SHA-256 digest is missing")))
    }

    fn ensure_agent_package(&self, profile: &AndroidRuntimeProfile) -> Result<(), AndroidGuestAgentPortError> {
        let package_path = self.adb_shell(profile, &[SHELL_COMMAND_PM, PM_PATH, AGENT_PACKAGE])?;
        if package_path
            .lines()
            .any(|line| line.trim_start().starts_with("package:"))
        {
            Ok(())
        } else {
            Err(AndroidGuestAgentPortError::ProvisionFailed(String::from(
                "TurkuazInputAgent package is not installed",
            )))
        }
    }

    fn ensure_host_secret(
        &self,
        profile: &AndroidRuntimeProfile,
    ) -> Result<[u8; TVGB_ROOT_KEY_LEN], AndroidGuestAgentPortError> {
        fs::create_dir_all(&self.settings.secret_root)
            .map_err(|error| AndroidGuestAgentPortError::ProvisionFailed(error.to_string()))?;
        set_private_directory_permissions(&self.settings.secret_root)
            .map_err(|error| AndroidGuestAgentPortError::ProvisionFailed(format!("TVGB secret directory ACL failed: {error}")))?;
        let path = self.secret_path(profile);
        if path.is_file() {
            set_private_permissions(&path)
                .map_err(|error| AndroidGuestAgentPortError::ProvisionFailed(format!("TVGB secret ACL migration failed: {error}")))?;
            let bytes = fs::read(&path)
                .map_err(|error| AndroidGuestAgentPortError::ProvisionFailed(error.to_string()))?;
            return bytes.try_into().map_err(|_| {
                AndroidGuestAgentPortError::ProvisionFailed(String::from(
                    "TVGB root key length is invalid",
                ))
            });
        }

        let mut secret = [0_u8; TVGB_ROOT_KEY_LEN];
        getrandom::getrandom(&mut secret)
            .map_err(|error| AndroidGuestAgentPortError::ProvisionFailed(error.to_string()))?;
        let temp = path.with_extension("key.tmp");
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|error| AndroidGuestAgentPortError::ProvisionFailed(error.to_string()))?;
        file.write_all(&secret)
            .and_then(|()| file.sync_all())
            .map_err(|error| AndroidGuestAgentPortError::ProvisionFailed(error.to_string()))?;
        set_private_permissions(&temp)
            .map_err(|error| AndroidGuestAgentPortError::ProvisionFailed(error.to_string()))?;
        fs::rename(&temp, &path)
            .map_err(|error| AndroidGuestAgentPortError::ProvisionFailed(error.to_string()))?;
        set_private_permissions(&path)
            .map_err(|error| AndroidGuestAgentPortError::ProvisionFailed(format!("TVGB secret final ACL failed: {error}")))?;
        Ok(secret)
    }

    fn secret_path(&self, profile: &AndroidRuntimeProfile) -> PathBuf {
        self.settings
            .secret_root
            .join(profile.vm_id().as_str())
            .with_extension(HOST_SECRET_EXTENSION)
    }

    fn provision_security_secret(
        &self,
        profile: &AndroidRuntimeProfile,
    ) -> Result<(), AndroidGuestAgentPortError> {
        self.ensure_agent_package(profile)?;
        let mut secret = self.ensure_host_secret(profile)?;
        let result = self.provision_security_secret_inner(profile, &secret);
        secret.fill(0);
        result
    }

    fn provision_security_secret_inner(
        &self,
        profile: &AndroidRuntimeProfile,
        secret: &[u8; TVGB_ROOT_KEY_LEN],
    ) -> Result<(), AndroidGuestAgentPortError> {
        let binary = self
            .binary
            .as_deref()
            .ok_or(AndroidGuestAgentPortError::BridgeUnavailable)?;
        let serial = profile.adb_serial();
        if self.settings.require_adb_root_for_provisioning {
            let root = run_command(
                binary,
                &["-s", serial.as_str(), "root"],
                self.settings.command_timeout,
            )?;
            if !root.0 && !root.1.contains("already running as root") {
                return Err(AndroidGuestAgentPortError::ProvisionFailed(format!(
                    "ADB root is required for TVGB key provisioning: {}",
                    root.1
                )));
            }
            let wait = run_command(
                binary,
                &["-s", serial.as_str(), "wait-for-device"],
                self.settings.command_timeout,
            )?;
            if !wait.0 {
                return Err(AndroidGuestAgentPortError::ProvisionFailed(wait.1));
            }
            let uid = self.adb_shell(profile, &["id", "-u"])?;
            if uid.trim() != "0" {
                return Err(AndroidGuestAgentPortError::ProvisionFailed(String::from(
                    "ADB root verification failed",
                )));
            }
        }

        let local_temp = self.secret_path(profile).with_extension("provision.tmp");
        fs::write(&local_temp, secret)
            .map_err(|error| AndroidGuestAgentPortError::ProvisionFailed(error.to_string()))?;
        set_private_permissions(&local_temp)
            .map_err(|error| AndroidGuestAgentPortError::ProvisionFailed(error.to_string()))?;
        let local_temp_text = local_temp.to_string_lossy().into_owned();
        let push = run_command(
            binary,
            &[
                "-s",
                serial.as_str(),
                "push",
                local_temp_text.as_str(),
                GUEST_SECRET_TEMP,
            ],
            self.settings.command_timeout,
        );
        let _ = fs::remove_file(&local_temp);
        let push = push?;
        if !push.0 {
            return Err(AndroidGuestAgentPortError::ProvisionFailed(push.1));
        }

        let uid_output = self.adb_shell(
            profile,
            &["cmd", "package", "list", "packages", "-U", AGENT_PACKAGE],
        )?;
        let uid = parse_package_uid(&uid_output).ok_or_else(|| {
            AndroidGuestAgentPortError::ProvisionFailed(String::from(
                "TurkuazInputAgent UID could not be resolved",
            ))
        })?;
        let owner = format!("{uid}:{uid}");
        self.adb_shell(profile, &["mkdir", "-p", GUEST_SECRET_DIRECTORY])?;
        self.adb_shell(profile, &["cp", GUEST_SECRET_TEMP, GUEST_SECRET_TARGET])?;
        self.adb_shell(profile, &["chown", owner.as_str(), GUEST_SECRET_TARGET])?;
        self.adb_shell(profile, &["chmod", "600", GUEST_SECRET_TARGET])?;
        self.adb_shell(profile, &["restorecon", "-F", GUEST_SECRET_TARGET])?;
        self.adb_shell(profile, &["rm", "-f", GUEST_SECRET_TEMP])?;
        Ok(())
    }

    fn enable_accessibility_service(
        &self,
        profile: &AndroidRuntimeProfile,
    ) -> Result<(), AndroidGuestAgentPortError> {
        self.ensure_agent_package(profile)?;
        let current = self.adb_shell(
            profile,
            &[
                SHELL_COMMAND_SETTINGS,
                SETTINGS_GET,
                SETTINGS_SECURE,
                SETTING_ENABLED_ACCESSIBILITY_SERVICES,
            ],
        )?;
        let mut services = current
            .trim()
            .split(ACCESSIBILITY_SERVICE_SEPARATOR)
            .map(str::trim)
            .filter(|value| !value.is_empty() && *value != SETTINGS_NULL)
            .map(str::to_owned)
            .collect::<Vec<_>>();
        if !services.iter().any(|value| value == AGENT_ACCESSIBILITY_COMPONENT) {
            services.push(String::from(AGENT_ACCESSIBILITY_COMPONENT));
        }
        let enabled_services = services.join(&ACCESSIBILITY_SERVICE_SEPARATOR.to_string());
        self.adb_shell(
            profile,
            &[
                SHELL_COMMAND_SETTINGS,
                SETTINGS_PUT,
                SETTINGS_SECURE,
                SETTING_ENABLED_ACCESSIBILITY_SERVICES,
                enabled_services.as_str(),
            ],
        )?;
        self.adb_shell(
            profile,
            &[
                SHELL_COMMAND_SETTINGS,
                SETTINGS_PUT,
                SETTINGS_SECURE,
                SETTING_ACCESSIBILITY_ENABLED,
                SETTINGS_TRUE,
            ],
        )?;
        Ok(())
    }

    fn wait_for_agent_ping(
        &mut self,
        profile: &AndroidRuntimeProfile,
    ) -> Result<(), AndroidGuestAgentPortError> {
        let started_at = Instant::now();
        loop {
            if matches!(
                self.request(profile, GuestAgentAction::Ping),
                Ok(GuestAgentResponseData::Pong)
            ) {
                return Ok(());
            }
            if started_at.elapsed() >= self.settings.ready_timeout {
                self.release_transport_for_vm(profile.vm_id().as_str());
                return Err(AndroidGuestAgentPortError::Timeout);
            }
            thread::sleep(self.settings.ready_poll_interval);
        }
    }

    fn request(
        &mut self,
        profile: &AndroidRuntimeProfile,
        action: GuestAgentAction,
    ) -> Result<GuestAgentResponseData, AndroidGuestAgentPortError> {
        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.wrapping_add(1).max(1);
        let request = GuestAgentRequest {
            request_id,
            protocol_version: GUEST_AGENT_PROTOCOL_VERSION,
            action,
        };
        let vm_id = profile.vm_id().as_str().to_owned();
        let mut last_error = None;

        for _ in 0..SESSION_RETRY_COUNT {
            match self.request_once(profile, &vm_id, &request) {
                Ok(response) => {
                    if response.request_id != request_id
                        || response.protocol_version != GUEST_AGENT_PROTOCOL_VERSION
                    {
                        return Err(AndroidGuestAgentPortError::Protocol(String::from(
                            "Guest Agent response correlation failed",
                        )));
                    }
                    if !response.ok {
                        return Err(AndroidGuestAgentPortError::Protocol(
                            response.error.unwrap_or_else(|| {
                                String::from("Guest Agent rejected request")
                            }),
                        ));
                    }
                    return response.data.ok_or_else(|| {
                        AndroidGuestAgentPortError::Protocol(String::from(
                            "Guest Agent response data missing",
                        ))
                    });
                }
                Err(error) => {
                    last_error = Some(error);
                    self.sessions.remove(&vm_id);
                }
            }
        }
        self.release_transport_for_vm(&vm_id);
        Err(last_error.unwrap_or_else(|| {
            AndroidGuestAgentPortError::Protocol(String::from("TVGB session failed"))
        }))
    }

    fn request_once(
        &mut self,
        profile: &AndroidRuntimeProfile,
        vm_id: &str,
        request: &GuestAgentRequest,
    ) -> Result<GuestAgentResponse, AndroidGuestAgentPortError> {
        let port = self.ensure_forward(profile)?;
        if !self.sessions.contains_key(vm_id) {
            let mut root_key = self.ensure_host_secret(profile)?;
            let endpoint = SocketAddrV4::new(self.settings.host_ip, port);
            let session = AgentSession::connect(
                endpoint,
                self.settings.connect_timeout,
                self.settings.io_timeout,
                &root_key,
            );
            root_key.fill(0);
            self.sessions.insert(vm_id.to_owned(), session?);
        }
        self.sessions
            .get_mut(vm_id)
            .ok_or_else(|| {
                AndroidGuestAgentPortError::Protocol(String::from("TVGB session missing"))
            })?
            .request(request)
    }
}

impl Drop for AndroidGuestAgentTool {
    fn drop(&mut self) {
        self.sessions.clear();
        let leases = self.forwards.values().cloned().collect::<Vec<_>>();
        for lease in &leases {
            self.remove_forward(lease);
        }
        self.forwards.clear();
    }
}

impl AndroidGuestAgentPort for AndroidGuestAgentTool {
    fn provision_and_wait_ready(
        &mut self,
        profile: &AndroidRuntimeProfile,
    ) -> Result<AndroidGuestAgentReport, AndroidGuestAgentPortError> {
        let update = self.apply_signed_update_if_configured(profile)?;
        let result = (|| {
            self.provision_security_secret(profile)?;
            self.enable_accessibility_service(profile)?;
            self.wait_for_agent_ping(profile)?;
            self.inspect(profile)
        })();
        match result {
            Ok(report) => {
                if let Some(transaction) = update.as_ref() {
                    self.commit_agent_update(transaction)?;
                }
                Ok(report)
            }
            Err(error) => {
                if let Some(transaction) = update {
                    if let Err(rollback_error) = self.rollback_agent_update(profile, transaction.rollback.as_ref()) {
                        return Err(AndroidGuestAgentPortError::ProvisionFailed(format!(
                            "Guest Agent health-check failed: {error:?}; rollback failed: {rollback_error:?}"
                        )));
                    }
                }
                Err(error)
            }
        }
    }

    fn inspect(
        &mut self,
        profile: &AndroidRuntimeProfile,
    ) -> Result<AndroidGuestAgentReport, AndroidGuestAgentPortError> {
        let response = match self.request(profile, GuestAgentAction::Capabilities) {
            Ok(value) => value,
            Err(AndroidGuestAgentPortError::ConnectFailed(_)) => {
                return Ok(AndroidGuestAgentReport::unavailable());
            }
            Err(error) => return Err(error),
        };
        match response {
            GuestAgentResponseData::Capabilities(value) => Ok(AndroidGuestAgentReport {
                available: true,
                persistent_multi_touch: value.persistent_multi_touch,
                max_contacts: value.max_contacts,
                continuation_api: value.continuation_api,
                input_backend: Some(value.input_backend),
                endpoint: self
                    .forwards
                    .get(profile.vm_id().as_str())
                    .map(|lease| format!("{}:{}", self.settings.host_ip, lease.port)),
                secure_transport_v2: value.secure_transport_v2,
                clipboard: value.clipboard,
            }),
            _ => Err(AndroidGuestAgentPortError::Protocol(String::from(
                "Guest Agent capability response type mismatch",
            ))),
        }
    }

    fn apply_touch_frame(
        &mut self,
        profile: &AndroidRuntimeProfile,
        contacts: &[AndroidTouchContact],
    ) -> Result<(), AndroidGuestAgentPortError> {
        let contacts = contacts
            .iter()
            .map(|contact| GuestTouchContact {
                pointer_id: contact.pointer_id,
                phase: match contact.phase {
                    AndroidTouchPhase::Down => GuestTouchPhase::Down,
                    AndroidTouchPhase::Move => GuestTouchPhase::Move,
                    AndroidTouchPhase::Up => GuestTouchPhase::Up,
                },
                x: contact.x,
                y: contact.y,
            })
            .collect();
        match self.request(profile, GuestAgentAction::ApplyTouchFrame { contacts })? {
            GuestAgentResponseData::Ack => Ok(()),
            _ => Err(AndroidGuestAgentPortError::Protocol(String::from(
                "Guest Agent touch response type mismatch",
            ))),
        }
    }

    fn reset_input(
        &mut self,
        profile: &AndroidRuntimeProfile,
    ) -> Result<(), AndroidGuestAgentPortError> {
        match self.request(profile, GuestAgentAction::ResetInput)? {
            GuestAgentResponseData::Ack => Ok(()),
            _ => Err(AndroidGuestAgentPortError::Protocol(String::from(
                "Guest Agent reset response type mismatch",
            ))),
        }
    }

    fn read_clipboard(
        &mut self,
        profile: &AndroidRuntimeProfile,
    ) -> Result<String, AndroidGuestAgentPortError> {
        match self.request(profile, GuestAgentAction::ClipboardGet)? {
            GuestAgentResponseData::ClipboardText { text } => Ok(text),
            _ => Err(AndroidGuestAgentPortError::Protocol(String::from(
                "Guest Agent clipboard response type mismatch",
            ))),
        }
    }

    fn write_clipboard(
        &mut self,
        profile: &AndroidRuntimeProfile,
        text: &str,
    ) -> Result<(), AndroidGuestAgentPortError> {
        match self.request(
            profile,
            GuestAgentAction::ClipboardSet {
                text: text.to_owned(),
            },
        )? {
            GuestAgentResponseData::Ack => Ok(()),
            _ => Err(AndroidGuestAgentPortError::Protocol(String::from(
                "Guest Agent clipboard response type mismatch",
            ))),
        }
    }
}

fn signed_manifest_payload(manifest: &SignedAgentManifest) -> String {
    format!(
        "TVM-GUEST-AGENT-V2\npackage_name={}\nversion_code={}\napk_file={}\nsha256={}\napk_signing_cert_sha256={}\n",
        manifest.package_name, manifest.version_code, manifest.apk_file, manifest.sha256, manifest.apk_signing_cert_sha256
    )
}

fn parse_version_code(output: &str) -> Option<u64> {
    output.lines().find_map(|line| {
        let marker = line.trim().strip_prefix("versionCode=")?;
        marker.split_whitespace().next()?.parse().ok()
    })
}

fn is_simple_apk_file_name(value: &str) -> bool {
    !value.is_empty()
        && value.ends_with(".apk")
        && !value.contains('/')
        && !value.contains('\\')
        && value != "."
        && value != ".."
}

fn is_lower_hex(value: &str, expected_len: usize) -> bool {
    value.len() == expected_len
        && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn decode_hex_array<const N: usize>(value: &str) -> Result<[u8; N], AndroidGuestAgentPortError> {
    if !is_lower_hex(value, N * 2) {
        return Err(AndroidGuestAgentPortError::ProvisionFailed(String::from("hex field is invalid")));
    }
    let mut output = [0_u8; N];
    let bytes = value.as_bytes();
    for (index, target) in output.iter_mut().enumerate() {
        let high = hex_nibble(bytes[index * 2])?;
        let low = hex_nibble(bytes[index * 2 + 1])?;
        *target = (high << 4) | low;
    }
    Ok(output)
}

fn hex_nibble(value: u8) -> Result<u8, AndroidGuestAgentPortError> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        _ => Err(AndroidGuestAgentPortError::ProvisionFailed(String::from("hex field is invalid"))),
    }
}

fn sha256_file_hex(path: &Path) -> Result<String, AndroidGuestAgentPortError> {
    let mut file = fs::File::open(path)
        .map_err(|error| AndroidGuestAgentPortError::ProvisionFailed(format!("APK read failed: {error}")))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer)
            .map_err(|error| AndroidGuestAgentPortError::ProvisionFailed(format!("APK read failed: {error}")))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn parse_package_uid(output: &str) -> Option<u32> {
    output
        .split_whitespace()
        .find_map(|part| part.strip_prefix("uid:"))
        .and_then(|value| value.parse().ok())
}

fn discover_adb_binary(explicit: Option<&Path>) -> Option<PathBuf> {
    if let Some(path) = explicit.filter(|path| path.is_file()) {
        return Some(path.to_path_buf());
    }
    for variable in [ENV_ANDROID_SDK_ROOT, ENV_ANDROID_HOME] {
        if let Some(root) = env::var_os(variable).map(PathBuf::from) {
            let candidate = root.join(PLATFORM_TOOLS_DIR).join(adb_file_name());
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    let path = env::var_os("PATH")?;
    env::split_paths(&path)
        .map(|directory| directory.join(adb_file_name()))
        .find(|candidate| candidate.is_file())
}

fn adb_file_name() -> &'static str {
    if cfg!(windows) {
        COMMAND_ADB_WINDOWS
    } else {
        COMMAND_ADB_UNIX
    }
}

fn run_command(
    binary: &Path,
    args: &[&str],
    timeout: Duration,
) -> Result<(bool, String), AndroidGuestAgentPortError> {
    let mut child = Command::new(binary)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| AndroidGuestAgentPortError::ForwardFailed(error.to_string()))?;
    let started = Instant::now();
    loop {
        if child
            .try_wait()
            .map_err(|error| AndroidGuestAgentPortError::ForwardFailed(error.to_string()))?
            .is_some()
        {
            let output = child
                .wait_with_output()
                .map_err(|error| AndroidGuestAgentPortError::ForwardFailed(error.to_string()))?;
            let detail = format!(
                "{} {}",
                String::from_utf8_lossy(&output.stdout).trim(),
                String::from_utf8_lossy(&output.stderr).trim()
            )
            .trim()
            .to_owned();
            return Ok((output.status.success(), detail));
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(AndroidGuestAgentPortError::Timeout);
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn discover_apksigner_binary(explicit: Option<&Path>) -> Option<PathBuf> {
    if let Some(path) = explicit.filter(|path| path.is_file()) {
        return Some(path.to_path_buf());
    }
    for variable in [ENV_ANDROID_SDK_ROOT, ENV_ANDROID_HOME] {
        let Some(root) = env::var_os(variable).map(PathBuf::from) else { continue; };
        let build_tools = root.join(BUILD_TOOLS_DIR);
        let Ok(entries) = fs::read_dir(build_tools) else { continue; };
        let mut candidates = entries.filter_map(Result::ok)
            .map(|entry| entry.path().join(apksigner_file_name()))
            .filter(|path| path.is_file())
            .collect::<Vec<_>>();
        candidates.sort();
        if let Some(path) = candidates.pop() {
            return Some(path);
        }
    }
    let path = env::var_os("PATH")?;
    env::split_paths(&path)
        .map(|directory| directory.join(apksigner_file_name()))
        .find(|candidate| candidate.is_file())
}

fn apksigner_file_name() -> &'static str {
    if cfg!(windows) { COMMAND_APKSIGNER_WINDOWS } else { COMMAND_APKSIGNER_UNIX }
}

fn run_apksigner(
    binary: &Path,
    args: &[&str],
    timeout: Duration,
) -> Result<(bool, String), AndroidGuestAgentPortError> {
    #[cfg(windows)]
    {
        if binary.extension().and_then(|value| value.to_str()).is_some_and(|value| value.eq_ignore_ascii_case("bat")) {
            let binary_text = binary.to_string_lossy().into_owned();
            let mut command_args = vec!["/C", binary_text.as_str()];
            command_args.extend_from_slice(args);
            return run_command(Path::new("cmd.exe"), &command_args, timeout);
        }
    }
    run_command(binary, args, timeout)
}

#[cfg(unix)]
fn set_private_directory_permissions(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(path, permissions)
}

#[cfg(windows)]
fn set_private_directory_permissions(path: &Path) -> std::io::Result<()> {
    set_windows_private_acl(path, true)
}

#[cfg(windows)]
fn set_windows_private_acl(path: &Path, directory: bool) -> std::io::Result<()> {
    const SCRIPT: &str = r#"$p=$args[0]; $isDir=$args[1] -eq '1'; $sid=[System.Security.Principal.WindowsIdentity]::GetCurrent().User; if($isDir){$acl=New-Object System.Security.AccessControl.DirectorySecurity; $flags=[System.Security.AccessControl.InheritanceFlags]'ContainerInherit,ObjectInherit'}else{$acl=New-Object System.Security.AccessControl.FileSecurity; $flags=[System.Security.AccessControl.InheritanceFlags]::None}; $acl.SetAccessRuleProtection($true,$false); $rule=New-Object System.Security.AccessControl.FileSystemAccessRule($sid,[System.Security.AccessControl.FileSystemRights]::FullControl,$flags,[System.Security.AccessControl.PropagationFlags]::None,[System.Security.AccessControl.AccessControlType]::Allow); $acl.AddAccessRule($rule); Set-Acl -LiteralPath $p -AclObject $acl"#;
    let status = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", SCRIPT])
        .arg(path)
        .arg(if directory { "1" } else { "0" })
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    if status.success() { Ok(()) } else { Err(std::io::Error::other("Windows private ACL command failed")) }
}

#[cfg(unix)]
fn set_private_permissions(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o600);
    fs::set_permissions(path, permissions)
}

#[cfg(windows)]
fn set_private_permissions(path: &Path) -> std::io::Result<()> {
    set_windows_private_acl(path, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_uid_parser_accepts_android_cmd_output() {
        assert_eq!(
            parse_package_uid("package:com.turkuazvm.inputagent uid:10123"),
            Some(10123)
        );
    }
}
