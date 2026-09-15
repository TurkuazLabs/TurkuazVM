// # 📄 Dosya Yolu: /turkuazvm/apps/engine/src/tools/android_sdk_emulator_runtime_tool.rs
// # 📌 Amac: Resmi Android Emulator process yasam dongusunu TurkuazVM hypervisor runtime contractina uyarlar
// # 📌 Modul - Rust
// # Version: 0.40.13
// # Aciklama: VM-ozel AVD'yi WHPX/auto acceleration ile baslatir, console port readiness kontrol eder ve process maintenance olaylarini uretir
// # Bagimli Oldugu Katman: Tool | Service

use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use turkuazvm_core::domain::hypervisor_runtime_event::{HypervisorRuntimeEvent, HypervisorRuntimeEventKind};
use turkuazvm_core::domain::runtime_media::AndroidSdkEmulatorRuntimeMediaPlan;
use turkuazvm_core::domain::virtual_machine::{VirtualMachine, VmId};
use turkuazvm_core::ports::hypervisor_runtime_port::{
    HypervisorRuntimeError, HypervisorRuntimeInfo, HypervisorStopReport, HypervisorTerminationKind,
};

struct AndroidEmulatorProcessRecord {
    child: Child,
    runtime_info: HypervisorRuntimeInfo,
}

pub struct AndroidSdkEmulatorRuntimeTool {
    data_root: std::path::PathBuf,
    startup_timeout: Duration,
    startup_poll_interval: Duration,
    processes: HashMap<VmId, AndroidEmulatorProcessRecord>,
}

impl AndroidSdkEmulatorRuntimeTool {
    pub fn new(data_root: std::path::PathBuf, startup_timeout: Duration, startup_poll_interval: Duration) -> Self {
        Self { data_root, startup_timeout, startup_poll_interval, processes: HashMap::new() }
    }

    pub fn contains(&self, vm_id: &VmId) -> bool { self.processes.contains_key(vm_id) }

    pub fn start(&mut self, machine: &VirtualMachine, plan: &AndroidSdkEmulatorRuntimeMediaPlan) -> Result<HypervisorRuntimeInfo, HypervisorRuntimeError> {
        if self.processes.contains_key(machine.id()) { return Err(HypervisorRuntimeError::AlreadyRunning(machine.id().clone())); }
        if port_in_use(plan.console_port) { return Err(HypervisorRuntimeError::LaunchFailed(format!("Android Emulator console port {} is already in use", plan.console_port))); }
        let log_dir = self.data_root.join("logs").join("android-emulator");
        fs::create_dir_all(&log_dir).map_err(|error| HypervisorRuntimeError::LaunchFailed(error.to_string()))?;
        let log_path = log_dir.join(format!("{}.log", machine.id().as_str()));
        let mut log_file = OpenOptions::new().create(true).truncate(true).write(true).open(&log_path).map_err(|error| HypervisorRuntimeError::LaunchFailed(error.to_string()))?;
        let _ = writeln!(log_file, "# 📄 Dosya Yolu: /turkuazvm/data/logs/android-emulator/{}.log", machine.id().as_str());
        let _ = writeln!(log_file, "# 📌 Amac: Android Emulator runtime stdout/stderr kaydini saklar");
        let _ = writeln!(log_file, "# 📌 Modul - Log");
        let _ = writeln!(log_file, "# Version: {}", env!("CARGO_PKG_VERSION"));
        let _ = writeln!(log_file, "# Aciklama: AVD process baslatma ve emulator runtime ciktilarini kaydeder");
        let _ = writeln!(log_file, "# Bagimli Oldugu Katman: Tool | View\n");
        let stdout = log_file.try_clone().map_err(|error| HypervisorRuntimeError::LaunchFailed(error.to_string()))?;
        let sdk_root = plan.emulator_binary.parent().and_then(|path| path.parent()).ok_or_else(|| HypervisorRuntimeError::LaunchFailed(String::from("Android SDK root cannot be derived from emulator binary")))?;
        let mut child = Command::new(&plan.emulator_binary)
            .arg("-avd").arg(&plan.avd_name)
            .arg("-port").arg(plan.console_port.to_string())
            .arg("-no-snapshot")
            .arg("-no-boot-anim")
            .arg("-accel").arg("auto")
            .arg("-gpu").arg("auto")
            .arg("-memory").arg(machine.resources().memory_mib.to_string())
            .arg("-cores").arg(machine.resources().vcpu_count.to_string())
            .env("ANDROID_AVD_HOME", &plan.avd_home)
            .env("ANDROID_SDK_ROOT", sdk_root)
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(log_file))
            .spawn()
            .map_err(|error| HypervisorRuntimeError::LaunchFailed(format!("Android Emulator launch failed: {error}; log={}", log_path.display())))?;
        let runtime_info = HypervisorRuntimeInfo { process_id: child.id(), display: None };
        let deadline = Instant::now() + self.startup_timeout;
        loop {
            if port_in_use(plan.console_port) {
                self.processes.insert(machine.id().clone(), AndroidEmulatorProcessRecord { child, runtime_info });
                return Ok(runtime_info);
            }
            if let Some(status) = child.try_wait().map_err(|error| HypervisorRuntimeError::LaunchFailed(error.to_string()))? {
                return Err(HypervisorRuntimeError::LaunchFailed(format!("Android Emulator exited during startup: {status}; log={}", log_path.display())));
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                return Err(HypervisorRuntimeError::LaunchFailed(format!("Android Emulator console port {} did not become ready; log={}", plan.console_port, log_path.display())));
            }
            thread::sleep(self.startup_poll_interval);
        }
    }

    pub fn stop(&mut self, vm_id: &VmId) -> Result<HypervisorStopReport, HypervisorRuntimeError> {
        let mut record = self.processes.remove(vm_id).ok_or_else(|| HypervisorRuntimeError::NotRunning(vm_id.clone()))?;
        if record.child.try_wait().map_err(|error| HypervisorRuntimeError::StopFailed(error.to_string()))?.is_some() {
            return Ok(HypervisorStopReport { termination: HypervisorTerminationKind::Graceful });
        }
        record.child.kill().map_err(|error| HypervisorRuntimeError::StopFailed(error.to_string()))?;
        let _ = record.child.wait();
        Ok(HypervisorStopReport { termination: HypervisorTerminationKind::Forced })
    }

    pub fn runtime_info(&self, vm_id: &VmId) -> Result<HypervisorRuntimeInfo, HypervisorRuntimeError> {
        self.processes.get(vm_id).map(|record| record.runtime_info).ok_or_else(|| HypervisorRuntimeError::NotRunning(vm_id.clone()))
    }

    pub fn maintenance(&mut self) -> Result<Vec<HypervisorRuntimeEvent>, HypervisorRuntimeError> {
        let ids = self.processes.keys().cloned().collect::<Vec<_>>();
        let mut events = Vec::new();
        for vm_id in ids {
            let exited = self.processes.get_mut(&vm_id).map(|record| record.child.try_wait()).transpose().map_err(|error| HypervisorRuntimeError::ControlFailed(error.to_string()))?.flatten().is_some();
            if exited {
                self.processes.remove(&vm_id);
                events.push(HypervisorRuntimeEvent { vm_id, kind: HypervisorRuntimeEventKind::ProcessExited });
            }
        }
        Ok(events)
    }
}

fn port_in_use(port: u16) -> bool {
    let address = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port));
    TcpStream::connect_timeout(&address, Duration::from_millis(100)).is_ok()
}
