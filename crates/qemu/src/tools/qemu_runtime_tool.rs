// # 📄 Dosya Yolu: /turkuazvm/crates/qemu/src/tools/qemu_runtime_tool.rs
// # 📌 Amac: QEMU process, persistent reattach, QMP, Gaming GPU ve native display lifecycle adapterini yonetir
// # 📌 Modul - Rust
// # Version: 0.40.3
// # Aciklama: PID/start-token/executable kimligiyle restart reattach, QMP guest-reset event mapping, live ISO eject ve graceful/forced stop uygular
// # Bagimli Oldugu Katman: Repo | Tool

use std::collections::HashMap;
use std::fs;
use std::net::{SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Instant;

use turkuazvm_core::domain::display::{
    DisplayCapabilities, DisplayRuntimeInfo, DisplayTransport,
};
use turkuazvm_core::domain::network::NetworkRuntimePlan;
use turkuazvm_core::domain::hypervisor_runtime_event::{HypervisorRuntimeEvent, HypervisorRuntimeEventKind};
use turkuazvm_core::domain::runtime_media::VmRuntimeMediaPlan;
use turkuazvm_core::domain::runtime_registration::RuntimeRegistration;
use turkuazvm_core::domain::virtual_machine::{VirtualMachine, VmId};
use turkuazvm_core::ports::hypervisor_monitor_port::HypervisorMonitorPort;
use turkuazvm_core::ports::hypervisor_runtime_port::{
    HypervisorRuntimeError, HypervisorRuntimeInfo, HypervisorRuntimePort, HypervisorStopReport,
    HypervisorTerminationKind,
};
use turkuazvm_core::ports::runtime_registry_port::RuntimeRegistryPort;

use crate::domain::qemu_runtime::{QemuDisplayMode, QemuDisplayRuntimePlan, QemuRuntimeSettings};
use crate::tools::qemu_command_builder::QemuCommandBuilder;
use crate::tools::qmp_client_tool::QmpClientTool;
use crate::tools::qmp_tcp_transport_tool::{QmpTcpTransportSettings, QmpTcpTransportTool};

const RFB_BASE_PORT: u16 = 5900;
const QMP_EVENT_RESET: &str = "RESET";
#[cfg(target_os = "windows")]
const PROCESS_QUERY_POWERSHELL_PREFIX: &str = "$p=Get-Process -Id ";
#[cfg(target_os = "windows")]
const PROCESS_QUERY_POWERSHELL_SUFFIX: &str = " -ErrorAction Stop; [Console]::WriteLine($p.Path); [Console]::WriteLine($p.StartTime.ToFileTimeUtc())";

struct AttachedProcess {
    process_id: u32,
    start_token: String,
}

enum QemuProcessHandle {
    Owned(Child),
    Attached(AttachedProcess),
}

struct QemuProcessRecord {
    process: QemuProcessHandle,
    qmp: Option<QmpClientTool<QmpTcpTransportTool>>,
    qmp_endpoint: SocketAddr,
    control_unavailable_reported: bool,
    runtime_info: HypervisorRuntimeInfo,
}

pub struct QemuRuntimeTool<R>
where
    R: RuntimeRegistryPort,
{
    system_binary: PathBuf,
    settings: QemuRuntimeSettings,
    registry: R,
    processes: HashMap<VmId, QemuProcessRecord>,
    startup_events: Vec<HypervisorRuntimeEvent>,
}

impl<R> QemuRuntimeTool<R>
where
    R: RuntimeRegistryPort,
{
    pub fn new(system_binary: PathBuf, settings: QemuRuntimeSettings, registry: R) -> Self {
        let mut tool = Self {
            system_binary,
            settings,
            registry,
            processes: HashMap::new(),
            startup_events: Vec::new(),
        };
        tool.recover_registered_processes();
        tool
    }

    fn recover_registered_processes(&mut self) {
        let Ok(registrations) = self.registry.list() else {
            return;
        };
        for registration in registrations {
            if self.processes.contains_key(&registration.vm_id) {
                continue;
            }
            if !self.registration_matches_process(&registration) {
                let _ = self.registry.remove(&registration.vm_id);
                self.startup_events.push(HypervisorRuntimeEvent {
                    vm_id: registration.vm_id,
                    kind: HypervisorRuntimeEventKind::ProcessExited,
                });
                continue;
            }
            let (qmp, control_unavailable_reported) = match self.connect_existing_qmp(registration.qmp_endpoint) {
                Ok(client) => {
                    self.startup_events.push(HypervisorRuntimeEvent {
                        vm_id: registration.vm_id.clone(),
                        kind: HypervisorRuntimeEventKind::ControlReattached,
                    });
                    (Some(client), false)
                }
                Err(error) => {
                    self.startup_events.push(HypervisorRuntimeEvent {
                        vm_id: registration.vm_id.clone(),
                        kind: HypervisorRuntimeEventKind::ControlUnavailable {
                            detail: format!("{error:?}"),
                        },
                    });
                    (None, true)
                }
            };
            self.processes.insert(
                registration.vm_id.clone(),
                QemuProcessRecord {
                    process: QemuProcessHandle::Attached(AttachedProcess {
                        process_id: registration.process_id,
                        start_token: registration.process_start_token.clone(),
                    }),
                    qmp,
                    qmp_endpoint: registration.qmp_endpoint,
                    control_unavailable_reported,
                    runtime_info: HypervisorRuntimeInfo {
                        process_id: registration.process_id,
                        display: registration.display,
                    },
                },
            );
        }
    }

    fn registration_matches_process(&self, registration: &RuntimeRegistration) -> bool {
        let Ok((executable, start_token)) = process_identity(registration.process_id) else {
            return false;
        };
        paths_equivalent(&executable, &registration.executable_path)
            && paths_equivalent(&executable, &self.system_binary)
            && start_token == registration.process_start_token
    }

    fn allocate_qmp_endpoint(&self) -> Result<SocketAddr, HypervisorRuntimeError> {
        let listener = TcpListener::bind(SocketAddr::new(self.settings.qmp_bind_ip, 0))
            .map_err(|error| HypervisorRuntimeError::LaunchFailed(error.to_string()))?;
        listener
            .local_addr()
            .map_err(|error| HypervisorRuntimeError::LaunchFailed(error.to_string()))
    }

    fn allocate_display_plan(
        &self,
    ) -> Result<(QemuDisplayRuntimePlan, Option<DisplayRuntimeInfo>), HypervisorRuntimeError> {
        match self.settings.display_mode {
            QemuDisplayMode::NativeRfb => {
                for display_number in self.settings.rfb_display_min..=self.settings.rfb_display_max {
                    let port = RFB_BASE_PORT.checked_add(display_number).ok_or_else(|| {
                        HypervisorRuntimeError::LaunchFailed(String::from(
                            "RFB display port range overflow",
                        ))
                    })?;
                    let endpoint = SocketAddr::new(self.settings.rfb_bind_ip, port);
                    if TcpListener::bind(endpoint).is_ok() {
                        return Ok((
                            QemuDisplayRuntimePlan {
                                mode: QemuDisplayMode::NativeRfb,
                                rfb_display_number: Some(display_number),
                            },
                            Some(DisplayRuntimeInfo {
                                transport: DisplayTransport::Rfb,
                                endpoint,
                                local_only: endpoint.ip().is_loopback(),
                                capabilities: DisplayCapabilities {
                                    native_window: true,
                                    fullscreen: true,
                                    absolute_pointer: true,
                                    relative_pointer_capture: true,
                                    keyboard: true,
                                    gamepad_observation: false,
                                },
                            }),
                        ));
                    }
                }
                Err(HypervisorRuntimeError::LaunchFailed(format!(
                    "No free RFB display in range {}..={}",
                    self.settings.rfb_display_min, self.settings.rfb_display_max
                )))
            }
            mode => Ok((
                QemuDisplayRuntimePlan {
                    mode,
                    rfb_display_number: None,
                },
                None,
            )),
        }
    }

    fn connect_qmp(
        &self,
        endpoint: SocketAddr,
        child: &mut Child,
    ) -> Result<QmpClientTool<QmpTcpTransportTool>, HypervisorRuntimeError> {
        let started_at = Instant::now();
        loop {
            if let Some(status) = child
                .try_wait()
                .map_err(|error| HypervisorRuntimeError::LaunchFailed(error.to_string()))?
            {
                return Err(HypervisorRuntimeError::LaunchFailed(format!(
                    "QEMU exited before QMP became ready: {status}"
                )));
            }
            if let Ok(client) = self.try_connect_qmp(endpoint) {
                return Ok(client);
            }
            if started_at.elapsed() >= self.settings.startup_timeout {
                return Err(HypervisorRuntimeError::ControlFailed(String::from(
                    "QMP startup timeout",
                )));
            }
            thread::sleep(self.settings.startup_poll_interval);
        }
    }

    fn connect_existing_qmp(
        &self,
        endpoint: SocketAddr,
    ) -> Result<QmpClientTool<QmpTcpTransportTool>, HypervisorRuntimeError> {
        let started_at = Instant::now();
        loop {
            if let Ok(client) = self.try_connect_qmp(endpoint) {
                return Ok(client);
            }
            if started_at.elapsed() >= self.settings.startup_timeout {
                return Err(HypervisorRuntimeError::ControlFailed(String::from(
                    "QMP reattach timeout",
                )));
            }
            thread::sleep(self.settings.startup_poll_interval);
        }
    }

    fn try_connect_qmp(
        &self,
        endpoint: SocketAddr,
    ) -> Result<QmpClientTool<QmpTcpTransportTool>, HypervisorRuntimeError> {
        let transport_settings = QmpTcpTransportSettings {
            connect_timeout: self.settings.qmp_connect_timeout,
            read_timeout: self.settings.qmp_read_timeout,
            write_timeout: self.settings.qmp_write_timeout,
        };
        let transport = QmpTcpTransportTool::connect(endpoint, &transport_settings)
            .map_err(|error| HypervisorRuntimeError::ControlFailed(format!("{error:?}")))?;
        let mut client = QmpClientTool::new(transport);
        client
            .inspect_control_plane()
            .map_err(|error| HypervisorRuntimeError::ControlFailed(format!("{error:?}")))?;
        Ok(client)
    }

    fn persist_registration(
        &self,
        vm_id: &VmId,
        process_id: u32,
        qmp_endpoint: SocketAddr,
        runtime_info: HypervisorRuntimeInfo,
    ) -> Result<RuntimeRegistration, HypervisorRuntimeError> {
        let (executable_path, process_start_token) = process_identity(process_id)
            .map_err(HypervisorRuntimeError::ControlFailed)?;
        if !paths_equivalent(&executable_path, &self.system_binary) {
            return Err(HypervisorRuntimeError::ControlFailed(String::from(
                "QEMU process executable identity mismatch",
            )));
        }
        let registration = RuntimeRegistration {
            vm_id: vm_id.clone(),
            process_id,
            process_start_token,
            executable_path,
            qmp_endpoint,
            display: runtime_info.display,
        };
        self.registry
            .save(&registration)
            .map_err(|error| HypervisorRuntimeError::ControlFailed(format!("{error:?}")))?;
        Ok(registration)
    }

    fn stop_process(
        &self,
        mut record: QemuProcessRecord,
    ) -> Result<HypervisorStopReport, HypervisorRuntimeError> {
        let graceful_requested = record.qmp.as_mut().map(|qmp| qmp.request_quit().is_ok()).unwrap_or(false);
        let started_at = Instant::now();
        loop {
            let stopped = match &mut record.process {
                QemuProcessHandle::Owned(child) => match child.try_wait() {
                    Ok(Some(_)) => true,
                    Ok(None) => false,
                    Err(error) => {
                        return Err(HypervisorRuntimeError::StopFailed(error.to_string()));
                    }
                },
                QemuProcessHandle::Attached(process) => {
                    !process_identity_matches(process.process_id, &process.start_token, &self.system_binary)
                }
            };
            if stopped {
                return Ok(HypervisorStopReport {
                    termination: if graceful_requested {
                        HypervisorTerminationKind::Graceful
                    } else {
                        HypervisorTerminationKind::Forced
                    },
                });
            }
            if started_at.elapsed() >= self.settings.shutdown_timeout {
                match &mut record.process {
                    QemuProcessHandle::Owned(child) => {
                        child
                            .kill()
                            .and_then(|()| child.wait().map(|_| ()))
                            .map_err(|error| HypervisorRuntimeError::StopFailed(error.to_string()))?;
                    }
                    QemuProcessHandle::Attached(process) => {
                        force_terminate_process(process.process_id, &process.start_token, &self.system_binary)
                            .map_err(HypervisorRuntimeError::StopFailed)?;
                    }
                }
                return Ok(HypervisorStopReport {
                    termination: HypervisorTerminationKind::Forced,
                });
            }
            thread::sleep(self.settings.shutdown_poll_interval);
        }
    }
}

impl<R> HypervisorRuntimePort for QemuRuntimeTool<R>
where
    R: RuntimeRegistryPort,
{
    fn start(
        &mut self,
        machine: &VirtualMachine,
        network_plan: &NetworkRuntimePlan,
        runtime_media: Option<&VmRuntimeMediaPlan>,
    ) -> Result<HypervisorRuntimeInfo, HypervisorRuntimeError> {
        if self.processes.contains_key(machine.id()) {
            return Err(HypervisorRuntimeError::AlreadyRunning(machine.id().clone()));
        }

        let endpoint = self.allocate_qmp_endpoint()?;
        let (display_plan, display) = self.allocate_display_plan()?;
        let arguments = QemuCommandBuilder::build_arguments_with_gpu_and_image_root(
            machine,
            endpoint,
            display_plan,
            self.settings.gpu,
            &self.settings.data_root,
            &self.settings.image_root,
            network_plan,
            runtime_media,
        )
        .map_err(|error| HypervisorRuntimeError::LaunchFailed(format!("{error:?}")))?;
        let mut child = Command::new(&self.system_binary)
            .args(arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| HypervisorRuntimeError::LaunchFailed(error.to_string()))?;
        let runtime_info = HypervisorRuntimeInfo {
            process_id: child.id(),
            display,
        };

        match self.connect_qmp(endpoint, &mut child) {
            Ok(qmp) => {
                if let Err(error) = self.persist_registration(
                    machine.id(),
                    child.id(),
                    endpoint,
                    runtime_info,
                ) {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(error);
                }
                self.processes.insert(
                    machine.id().clone(),
                    QemuProcessRecord {
                        process: QemuProcessHandle::Owned(child),
                        qmp: Some(qmp),
                        qmp_endpoint: endpoint,
                        control_unavailable_reported: false,
                        runtime_info,
                    },
                );
                Ok(runtime_info)
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                Err(error)
            }
        }
    }

    fn stop(&mut self, vm_id: &VmId) -> Result<HypervisorStopReport, HypervisorRuntimeError> {
        let record = self
            .processes
            .remove(vm_id)
            .ok_or_else(|| HypervisorRuntimeError::NotRunning(vm_id.clone()))?;
        let report = self.stop_process(record)?;
        self.registry
            .remove(vm_id)
            .map_err(|error| HypervisorRuntimeError::StopFailed(format!("{error:?}")))?;
        Ok(report)
    }

    fn eject_installer_media(
        &mut self,
        vm_id: &VmId,
        media_id: &str,
    ) -> Result<(), HypervisorRuntimeError> {
        let record = self
            .processes
            .get_mut(vm_id)
            .ok_or_else(|| HypervisorRuntimeError::NotRunning(vm_id.clone()))?;
        let qmp = record.qmp.as_mut().ok_or_else(|| {
            HypervisorRuntimeError::ControlFailed(String::from(
                "QMP control is unavailable for live installer media eject",
            ))
        })?;
        qmp.request_media_eject(media_id)
            .map_err(|error| HypervisorRuntimeError::ControlFailed(format!("{error:?}")))
    }

    fn maintenance(&mut self) -> Result<Vec<HypervisorRuntimeEvent>, HypervisorRuntimeError> {
        let mut events = std::mem::take(&mut self.startup_events);
        let vm_ids = self.processes.keys().cloned().collect::<Vec<_>>();
        for vm_id in vm_ids {
            let exited = {
                let record = self.processes.get_mut(&vm_id).expect("runtime record must exist");
                match &mut record.process {
                    QemuProcessHandle::Owned(child) => child
                        .try_wait()
                        .map_err(|error| HypervisorRuntimeError::ControlFailed(error.to_string()))?
                        .is_some(),
                    QemuProcessHandle::Attached(process) => {
                        !process_identity_matches(process.process_id, &process.start_token, &self.system_binary)
                    }
                }
            };
            if exited {
                self.registry
                    .remove(&vm_id)
                    .map_err(|error| HypervisorRuntimeError::ControlFailed(format!("{error:?}")))?;
                self.processes.remove(&vm_id);
                events.push(HypervisorRuntimeEvent {
                    vm_id,
                    kind: HypervisorRuntimeEventKind::ProcessExited,
                });
                continue;
            }

            let needs_reconnect = self
                .processes
                .get(&vm_id)
                .map(|record| record.qmp.is_none())
                .unwrap_or(false);
            if needs_reconnect {
                let endpoint = self.processes.get(&vm_id).expect("runtime record must exist").qmp_endpoint;
                if let Ok(client) = self.try_connect_qmp(endpoint) {
                    let record = self.processes.get_mut(&vm_id).expect("runtime record must exist");
                    record.qmp = Some(client);
                    record.control_unavailable_reported = false;
                    events.push(HypervisorRuntimeEvent {
                        vm_id: vm_id.clone(),
                        kind: HypervisorRuntimeEventKind::ControlReattached,
                    });
                }
                continue;
            }

            let heartbeat_result = {
                let record = self.processes.get_mut(&vm_id).expect("runtime record must exist");
                record.qmp.as_mut().expect("QMP must exist").heartbeat()
            };
            match heartbeat_result {
                Ok(qmp_events) => {
                    for name in qmp_events {
                        let kind = if name == QMP_EVENT_RESET {
                            HypervisorRuntimeEventKind::GuestReset
                        } else {
                            HypervisorRuntimeEventKind::Qmp { name }
                        };
                        events.push(HypervisorRuntimeEvent {
                            vm_id: vm_id.clone(),
                            kind,
                        });
                    }
                }
                Err(error) => {
                    let record = self.processes.get_mut(&vm_id).expect("runtime record must exist");
                    record.qmp = None;
                    if !record.control_unavailable_reported {
                        record.control_unavailable_reported = true;
                        events.push(HypervisorRuntimeEvent {
                            vm_id: vm_id.clone(),
                            kind: HypervisorRuntimeEventKind::ControlUnavailable {
                                detail: format!("{error:?}"),
                            },
                        });
                    }
                }
            }
        }
        Ok(events)
    }

    fn runtime_info(&self, vm_id: &VmId) -> Result<HypervisorRuntimeInfo, HypervisorRuntimeError> {
        self.processes
            .get(vm_id)
            .map(|record| record.runtime_info)
            .ok_or_else(|| HypervisorRuntimeError::NotRunning(vm_id.clone()))
    }
}

fn paths_equivalent(left: &Path, right: &Path) -> bool {
    let left = fs::canonicalize(left).unwrap_or_else(|_| left.to_path_buf());
    let right = fs::canonicalize(right).unwrap_or_else(|_| right.to_path_buf());
    if cfg!(windows) {
        left.to_string_lossy().eq_ignore_ascii_case(&right.to_string_lossy())
    } else {
        left == right
    }
}

fn process_identity_matches(process_id: u32, start_token: &str, expected_executable: &Path) -> bool {
    process_identity(process_id)
        .map(|(executable, token)| token == start_token && paths_equivalent(&executable, expected_executable))
        .unwrap_or(false)
}

#[cfg(target_os = "linux")]
fn process_identity(process_id: u32) -> Result<(PathBuf, String), String> {
    let proc_root = PathBuf::from("/proc").join(process_id.to_string());
    let executable = fs::read_link(proc_root.join("exe")).map_err(|error| error.to_string())?;
    let stat = fs::read_to_string(proc_root.join("stat")).map_err(|error| error.to_string())?;
    let close = stat.rfind(')').ok_or_else(|| String::from("/proc stat command delimiter missing"))?;
    let fields = stat[close + 1..].split_whitespace().collect::<Vec<_>>();
    let start_time = fields.get(19).ok_or_else(|| String::from("/proc stat starttime missing"))?;
    Ok((executable, (*start_time).to_owned()))
}

#[cfg(target_os = "windows")]
fn process_identity(process_id: u32) -> Result<(PathBuf, String), String> {
    let command = format!(
        "{PROCESS_QUERY_POWERSHELL_PREFIX}{process_id}{PROCESS_QUERY_POWERSHELL_SUFFIX}"
    );
    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", command.as_str()])
        .stdin(Stdio::null())
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut lines = text.lines().map(str::trim).filter(|line| !line.is_empty());
    let executable = lines.next().ok_or_else(|| String::from("process executable missing"))?;
    let start_token = lines.next().ok_or_else(|| String::from("process start token missing"))?;
    Ok((PathBuf::from(executable), start_token.to_owned()))
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn process_identity(_process_id: u32) -> Result<(PathBuf, String), String> {
    Err(String::from("persistent process identity is unsupported on this host"))
}

#[cfg(target_os = "linux")]
fn force_terminate_process(process_id: u32, start_token: &str, expected_executable: &Path) -> Result<(), String> {
    if !process_identity_matches(process_id, start_token, expected_executable) {
        return Err(String::from("process identity changed before forced termination"));
    }
    let process_id_text = process_id.to_string();
    let status = Command::new("kill")
        .args(["-KILL", process_id_text.as_str()])
        .stdin(Stdio::null())
        .status()
        .map_err(|error| error.to_string())?;
    if status.success() { Ok(()) } else { Err(format!("kill failed with {status}")) }
}

#[cfg(target_os = "windows")]
fn force_terminate_process(process_id: u32, start_token: &str, expected_executable: &Path) -> Result<(), String> {
    if !process_identity_matches(process_id, start_token, expected_executable) {
        return Err(String::from("process identity changed before forced termination"));
    }
    let process_id_text = process_id.to_string();
    let status = Command::new("taskkill.exe")
        .args(["/PID", process_id_text.as_str(), "/T", "/F"])
        .stdin(Stdio::null())
        .status()
        .map_err(|error| error.to_string())?;
    if status.success() { Ok(()) } else { Err(format!("taskkill failed with {status}")) }
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn force_terminate_process(_process_id: u32, _start_token: &str, _expected_executable: &Path) -> Result<(), String> {
    Err(String::from("forced reattached process termination is unsupported on this host"))
}
