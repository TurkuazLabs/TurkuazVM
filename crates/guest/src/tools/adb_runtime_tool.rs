// # 📄 Dosya Yolu: /turkuazvm/crates/guest/src/tools/adb_runtime_tool.rs
// # 📌 Amac: Android Runtime portlarini resmi adb command-line araci uzerinden uygular
// # 📌 Modul - Rust
// # Version: 0.40.13
// # Aciklama: ADB discovery, device readiness, package lifecycle, display override ve input injection adapteridir
// # Bagimli Oldugu Katman: Tool

use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use turkuazvm_android::domain::device::{
    AndroidAbi, AndroidBridgeCapabilities, AndroidConnectionState, AndroidDeviceReport,
};
use turkuazvm_android::domain::input::AndroidInputAction;
use turkuazvm_android::domain::package::{AndroidPackageInfo, AndroidPackageName};
use turkuazvm_android::domain::runtime_profile::AndroidRuntimeProfile;
use turkuazvm_android::ports::android_device_port::{AndroidDeviceError, AndroidDevicePort};
use turkuazvm_android::ports::android_input_port::{AndroidInputPort, AndroidInputPortError};
use turkuazvm_android::ports::android_package_port::{AndroidPackagePort, AndroidPackagePortError};

const COMMAND_ADB_WINDOWS: &str = "adb.exe";
const COMMAND_ADB_UNIX: &str = "adb";
const ENV_ANDROID_SDK_ROOT: &str = "ANDROID_SDK_ROOT";
const ENV_ANDROID_HOME: &str = "ANDROID_HOME";
const PLATFORM_TOOLS_DIR: &str = "platform-tools";
const BOOT_COMPLETED_VALUE: &str = "1";
const PACKAGE_PREFIX: &str = "package:";
const TEMP_PREFIX: &str = "turkuazvm-adb";
const EMULATOR_SERIAL_PREFIX: &str = "emulator-";
const EMULATOR_ADB_PORT_MIN: u16 = 5555;
const EMULATOR_ADB_PORT_MAX: u16 = 5681;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
pub struct AdbRuntimeSettings {
    pub explicit_binary: Option<PathBuf>,
    pub command_timeout: Duration,
    pub ready_timeout: Duration,
    pub ready_poll_interval: Duration,
}

#[derive(Debug, Clone)]
pub struct AdbRuntimeTool {
    binary: Option<PathBuf>,
    settings: AdbRuntimeSettings,
    version_text: Option<String>,
}

#[derive(Debug)]
struct AdbCommandOutput {
    success: bool,
    stdout: String,
    stderr: String,
}

impl AdbRuntimeTool {
    pub fn new(settings: AdbRuntimeSettings) -> Self {
        let binary = discover_adb_binary(settings.explicit_binary.as_deref());
        let version_text = binary
            .as_ref()
            .and_then(|path| run_version(path, settings.command_timeout).ok());
        Self {
            binary,
            settings,
            version_text,
        }
    }

    pub fn binary_path(&self) -> Option<&Path> {
        self.binary.as_deref()
    }

    fn require_binary(&self) -> Result<&Path, AndroidDeviceError> {
        self.binary
            .as_deref()
            .ok_or(AndroidDeviceError::BridgeUnavailable)
    }

    fn ensure_connected(&self, profile: &AndroidRuntimeProfile) -> Result<(), AndroidDeviceError> {
        let binary = self.require_binary()?;
        if emulator_serial(profile).is_some() {
            return Ok(());
        }
        let serial = profile.adb_serial();
        let output = run_adb_command(binary, &["connect", serial.as_str()], self.settings.command_timeout)
            .map_err(map_device_exec_error)?;
        if output.success || output.stdout.contains("already connected") || output.stdout.contains("connected to") {
            Ok(())
        } else {
            Err(AndroidDeviceError::ConnectFailed(command_detail(&output)))
        }
    }

    fn target_command(
        &self,
        profile: &AndroidRuntimeProfile,
        args: &[&str],
    ) -> Result<AdbCommandOutput, AdbExecError> {
        let binary = self.binary.as_deref().ok_or(AdbExecError::Unavailable)?;
        let serial = target_serial(profile);
        let mut full_args = vec!["-s", serial.as_str()];
        full_args.extend_from_slice(args);
        run_adb_command(binary, &full_args, self.settings.command_timeout)
    }

    fn shell_command(
        &self,
        profile: &AndroidRuntimeProfile,
        args: &[&str],
    ) -> Result<AdbCommandOutput, AdbExecError> {
        let mut shell_args = vec!["shell"];
        shell_args.extend_from_slice(args);
        self.target_command(profile, &shell_args)
    }

    fn getprop(
        &self,
        profile: &AndroidRuntimeProfile,
        key: &str,
    ) -> Result<String, AndroidDeviceError> {
        let output = self
            .shell_command(profile, &["getprop", key])
            .map_err(map_device_exec_error)?;
        if !output.success {
            return Err(AndroidDeviceError::CommandFailed(command_detail(&output)));
        }
        Ok(output.stdout.trim().to_owned())
    }

    fn current_state(
        &self,
        profile: &AndroidRuntimeProfile,
    ) -> Result<AndroidConnectionState, AndroidDeviceError> {
        let output = self
            .target_command(profile, &["get-state"])
            .map_err(map_device_exec_error)?;
        if output.success && output.stdout.trim() == "device" {
            return Ok(AndroidConnectionState::Booting);
        }
        let detail = command_detail(&output).to_ascii_lowercase();
        if detail.contains("unauthorized") {
            Ok(AndroidConnectionState::Unauthorized)
        } else if detail.contains("offline") {
            Ok(AndroidConnectionState::Offline)
        } else {
            Ok(AndroidConnectionState::Disconnected)
        }
    }

    fn inspect_connected(
        &self,
        profile: &AndroidRuntimeProfile,
    ) -> Result<AndroidDeviceReport, AndroidDeviceError> {
        let state = self.current_state(profile)?;
        if state != AndroidConnectionState::Booting {
            return Ok(AndroidDeviceReport {
                serial: target_serial(profile),
                connection_state: state,
                boot_completed: false,
                sdk_level: None,
                abi: None,
                model: None,
            });
        }

        let boot_completed = self.getprop(profile, "sys.boot_completed")? == BOOT_COMPLETED_VALUE;
        let sdk_level = self
            .getprop(profile, "ro.build.version.sdk")?
            .parse::<u32>()
            .ok();
        let abi_text = self.getprop(profile, "ro.product.cpu.abi")?;
        let abi = (!abi_text.is_empty()).then(|| AndroidAbi::parse(&abi_text));
        let model = self.getprop(profile, "ro.product.model")?;
        Ok(AndroidDeviceReport {
            serial: target_serial(profile),
            connection_state: if boot_completed {
                AndroidConnectionState::Ready
            } else {
                AndroidConnectionState::Booting
            },
            boot_completed,
            sdk_level,
            abi,
            model: (!model.is_empty()).then_some(model),
        })
    }

    fn require_package_success(output: AdbCommandOutput) -> Result<(), AndroidPackagePortError> {
        if output.success && !output.stdout.to_ascii_lowercase().contains("failure") {
            Ok(())
        } else {
            Err(AndroidPackagePortError::CommandFailed(command_detail(&output)))
        }
    }
}

fn emulator_serial(profile: &AndroidRuntimeProfile) -> Option<String> {
    let port = profile.adb_host_port();
    if cfg!(windows) && (EMULATOR_ADB_PORT_MIN..=EMULATOR_ADB_PORT_MAX).contains(&port) && port % 2 == 1 {
        Some(format!("{EMULATOR_SERIAL_PREFIX}{}", port - 1))
    } else {
        None
    }
}

fn target_serial(profile: &AndroidRuntimeProfile) -> String {
    emulator_serial(profile).unwrap_or_else(|| profile.adb_serial())
}

impl AndroidDevicePort for AdbRuntimeTool {
    fn capabilities(&self) -> AndroidBridgeCapabilities {
        match (&self.binary, &self.version_text) {
            (Some(_), version) => AndroidBridgeCapabilities {
                available: true,
                version_text: version.clone(),
                package_management: true,
                display_override: true,
                input_injection: true,
            },
            (None, _) => AndroidBridgeCapabilities::unavailable(),
        }
    }

    fn inspect(
        &self,
        profile: &AndroidRuntimeProfile,
    ) -> Result<AndroidDeviceReport, AndroidDeviceError> {
        self.ensure_connected(profile)?;
        self.inspect_connected(profile)
    }

    fn wait_until_ready(
        &self,
        profile: &AndroidRuntimeProfile,
    ) -> Result<AndroidDeviceReport, AndroidDeviceError> {
        self.ensure_connected(profile)?;
        let started_at = Instant::now();
        loop {
            let report = self.inspect_connected(profile)?;
            if report.connection_state == AndroidConnectionState::Ready {
                return Ok(report);
            }
            if matches!(
                report.connection_state,
                AndroidConnectionState::Unauthorized | AndroidConnectionState::Offline
            ) {
                return Ok(report);
            }
            if started_at.elapsed() >= self.settings.ready_timeout {
                return Err(AndroidDeviceError::DeviceNotReady);
            }
            thread::sleep(self.settings.ready_poll_interval);
        }
    }

    fn apply_display(&self, profile: &AndroidRuntimeProfile) -> Result<(), AndroidDeviceError> {
        self.ensure_connected(profile)?;
        let display = profile.display();
        let size = format!("{}x{}", display.width(), display.height());
        let density = display.density_dpi().to_string();
        for args in [
            vec!["wm", "size", size.as_str()],
            vec!["wm", "density", density.as_str()],
        ] {
            let output = self
                .shell_command(profile, &args)
                .map_err(map_device_exec_error)?;
            if !output.success {
                return Err(AndroidDeviceError::CommandFailed(command_detail(&output)));
            }
        }
        Ok(())
    }
}

impl AndroidPackagePort for AdbRuntimeTool {
    fn list_packages(
        &self,
        profile: &AndroidRuntimeProfile,
    ) -> Result<Vec<AndroidPackageInfo>, AndroidPackagePortError> {
        self.ensure_connected(profile).map_err(map_package_device_error)?;
        let output = self
            .shell_command(profile, &["pm", "list", "packages"])
            .map_err(map_package_exec_error)?;
        if !output.success {
            return Err(AndroidPackagePortError::CommandFailed(command_detail(&output)));
        }
        let mut packages = Vec::new();
        for line in output.stdout.lines() {
            let value = line.trim().strip_prefix(PACKAGE_PREFIX).unwrap_or_default();
            if value.is_empty() {
                continue;
            }
            if let Ok(package_name) = AndroidPackageName::parse(value.to_owned()) {
                packages.push(AndroidPackageInfo { package_name });
            }
        }
        packages.sort_by(|left, right| left.package_name.as_str().cmp(right.package_name.as_str()));
        Ok(packages)
    }

    fn install_apk(
        &self,
        profile: &AndroidRuntimeProfile,
        apk_path: &Path,
    ) -> Result<(), AndroidPackagePortError> {
        self.ensure_connected(profile).map_err(map_package_device_error)?;
        if !apk_path.is_file() {
            return Err(AndroidPackagePortError::ApkNotFound);
        }
        let apk = apk_path
            .to_str()
            .ok_or_else(|| AndroidPackagePortError::CommandFailed(String::from("APK path is not valid UTF-8")))?;
        let output = self
            .target_command(profile, &["install", "-r", apk])
            .map_err(map_package_exec_error)?;
        Self::require_package_success(output)
    }

    fn uninstall_package(
        &self,
        profile: &AndroidRuntimeProfile,
        package: &AndroidPackageName,
    ) -> Result<(), AndroidPackagePortError> {
        self.ensure_connected(profile).map_err(map_package_device_error)?;
        let output = self
            .target_command(profile, &["uninstall", package.as_str()])
            .map_err(map_package_exec_error)?;
        Self::require_package_success(output)
    }

    fn launch_package(
        &self,
        profile: &AndroidRuntimeProfile,
        package: &AndroidPackageName,
    ) -> Result<(), AndroidPackagePortError> {
        self.ensure_connected(profile).map_err(map_package_device_error)?;
        let resolved = self
            .shell_command(
                profile,
                &["cmd", "package", "resolve-activity", "--brief", package.as_str()],
            )
            .map_err(map_package_exec_error)?;
        if !resolved.success {
            return Err(AndroidPackagePortError::LaunchActivityNotFound);
        }
        let component = resolved
            .stdout
            .lines()
            .map(str::trim)
            .find(|line| line.contains('/') && !line.contains("No activity"))
            .ok_or(AndroidPackagePortError::LaunchActivityNotFound)?;
        let output = self
            .shell_command(profile, &["am", "start", "-n", component])
            .map_err(map_package_exec_error)?;
        Self::require_package_success(output)
    }

    fn stop_package(
        &self,
        profile: &AndroidRuntimeProfile,
        package: &AndroidPackageName,
    ) -> Result<(), AndroidPackagePortError> {
        self.ensure_connected(profile).map_err(map_package_device_error)?;
        let output = self
            .shell_command(profile, &["am", "force-stop", package.as_str()])
            .map_err(map_package_exec_error)?;
        Self::require_package_success(output)
    }
}

impl AndroidInputPort for AdbRuntimeTool {
    fn inject(
        &self,
        profile: &AndroidRuntimeProfile,
        action: &AndroidInputAction,
    ) -> Result<(), AndroidInputPortError> {
        self.ensure_connected(profile).map_err(map_input_device_error)?;
        let args = input_action_args(action);
        let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
        let output = self
            .shell_command(profile, &refs)
            .map_err(map_input_exec_error)?;
        if output.success {
            Ok(())
        } else {
            Err(AndroidInputPortError::CommandFailed(command_detail(&output)))
        }
    }
}

fn input_action_args(action: &AndroidInputAction) -> Vec<String> {
    match action {
        AndroidInputAction::Tap { x, y } => vec![
            String::from("input"),
            String::from("tap"),
            x.to_string(),
            y.to_string(),
        ],
        AndroidInputAction::Swipe {
            from_x,
            from_y,
            to_x,
            to_y,
            duration_ms,
        } => vec![
            String::from("input"),
            String::from("swipe"),
            from_x.to_string(),
            from_y.to_string(),
            to_x.to_string(),
            to_y.to_string(),
            duration_ms.to_string(),
        ],
        AndroidInputAction::KeyEvent { key_code } => vec![
            String::from("input"),
            String::from("keyevent"),
            key_code.to_string(),
        ],
        AndroidInputAction::Text { value } => vec![
            String::from("input"),
            String::from("text"),
            escape_input_text(value),
        ],
    }
}

#[derive(Debug)]
enum AdbExecError {
    Unavailable,
    Io(String),
    Timeout,
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
    find_on_path(adb_file_name())
}

fn adb_file_name() -> &'static str {
    if cfg!(windows) {
        COMMAND_ADB_WINDOWS
    } else {
        COMMAND_ADB_UNIX
    }
}

fn find_on_path(file_name: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    env::split_paths(&path)
        .map(|directory| directory.join(file_name))
        .find(|candidate| candidate.is_file())
}

fn run_version(binary: &Path, timeout: Duration) -> Result<String, AdbExecError> {
    let output = run_adb_command(binary, &["version"], timeout)?;
    if !output.success {
        return Err(AdbExecError::Io(command_detail(&output)));
    }
    Ok(output.stdout.lines().next().unwrap_or_default().trim().to_owned())
}

fn run_adb_command(
    binary: &Path,
    args: &[&str],
    timeout: Duration,
) -> Result<AdbCommandOutput, AdbExecError> {
    let sequence = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let stamp = format!("{}-{}-{sequence}", TEMP_PREFIX, std::process::id());
    let stdout_path = env::temp_dir().join(format!("{stamp}.out"));
    let stderr_path = env::temp_dir().join(format!("{stamp}.err"));
    let stdout_file = File::create(&stdout_path).map_err(|error| AdbExecError::Io(error.to_string()))?;
    let stderr_file = File::create(&stderr_path).map_err(|error| AdbExecError::Io(error.to_string()))?;
    let child_result = Command::new(binary)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file))
        .spawn();
    let mut child = match child_result {
        Ok(child) => child,
        Err(error) => {
            let _ = fs::remove_file(&stdout_path);
            let _ = fs::remove_file(&stderr_path);
            return Err(AdbExecError::Io(error.to_string()));
        }
    };
    let started_at = Instant::now();
    let status = loop {
        match child.try_wait().map_err(|error| AdbExecError::Io(error.to_string()))? {
            Some(status) => break status,
            None if started_at.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = fs::remove_file(&stdout_path);
                let _ = fs::remove_file(&stderr_path);
                return Err(AdbExecError::Timeout);
            }
            None => thread::sleep(Duration::from_millis(20)),
        }
    };
    let stdout = read_temp_text(&stdout_path);
    let stderr = read_temp_text(&stderr_path);
    let _ = fs::remove_file(stdout_path);
    let _ = fs::remove_file(stderr_path);
    Ok(AdbCommandOutput {
        success: status.success(),
        stdout,
        stderr,
    })
}

fn read_temp_text(path: &Path) -> String {
    let mut value = String::new();
    if let Ok(mut file) = File::open(path) {
        let _ = file.read_to_string(&mut value);
    }
    value
}

fn command_detail(output: &AdbCommandOutput) -> String {
    let stdout = output.stdout.trim();
    let stderr = output.stderr.trim();
    match (stdout.is_empty(), stderr.is_empty()) {
        (false, false) => format!("{stdout} | {stderr}"),
        (false, true) => stdout.to_owned(),
        (true, false) => stderr.to_owned(),
        (true, true) => String::from("ADB command failed without output"),
    }
}

fn escape_input_text(value: &str) -> String {
    value.replace(' ', "%s")
}

fn map_device_exec_error(error: AdbExecError) -> AndroidDeviceError {
    match error {
        AdbExecError::Unavailable => AndroidDeviceError::BridgeUnavailable,
        AdbExecError::Io(detail) => AndroidDeviceError::CommandFailed(detail),
        AdbExecError::Timeout => AndroidDeviceError::CommandTimeout,
    }
}

fn map_package_exec_error(error: AdbExecError) -> AndroidPackagePortError {
    match error {
        AdbExecError::Unavailable => AndroidPackagePortError::BridgeUnavailable,
        AdbExecError::Io(detail) => AndroidPackagePortError::CommandFailed(detail),
        AdbExecError::Timeout => AndroidPackagePortError::CommandTimeout,
    }
}

fn map_input_exec_error(error: AdbExecError) -> AndroidInputPortError {
    match error {
        AdbExecError::Unavailable => AndroidInputPortError::BridgeUnavailable,
        AdbExecError::Io(detail) => AndroidInputPortError::CommandFailed(detail),
        AdbExecError::Timeout => AndroidInputPortError::CommandTimeout,
    }
}

fn map_package_device_error(error: AndroidDeviceError) -> AndroidPackagePortError {
    match error {
        AndroidDeviceError::BridgeUnavailable => AndroidPackagePortError::BridgeUnavailable,
        AndroidDeviceError::CommandTimeout => AndroidPackagePortError::CommandTimeout,
        other => AndroidPackagePortError::CommandFailed(format!("{other:?}")),
    }
}

fn map_input_device_error(error: AndroidDeviceError) -> AndroidInputPortError {
    match error {
        AndroidDeviceError::BridgeUnavailable => AndroidInputPortError::BridgeUnavailable,
        AndroidDeviceError::CommandTimeout => AndroidInputPortError::CommandTimeout,
        other => AndroidInputPortError::CommandFailed(format!("{other:?}")),
    }
}
