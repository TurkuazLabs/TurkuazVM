// # 📄 Dosya Yolu: /turkuazvm/crates/qemu/src/tools/qemu_discovery_tool.rs
// # 📌 Amac: QEMU system ve qemu-img executable dosyalarini host PATH uzerinden bulur
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: HypervisorProbePort uygulayarak QEMU surum bilgisini core katmanina tasir
// # Bagimli Oldugu Katman: Service | Tool

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use turkuazvm_core::domain::hypervisor::{HypervisorInstallation, HypervisorKind};
use turkuazvm_core::ports::hypervisor_probe_port::{
    HypervisorProbeError, HypervisorProbePort,
};

const QEMU_SYSTEM_WINDOWS: &str = "qemu-system-x86_64.exe";
const QEMU_SYSTEM_UNIX: &str = "qemu-system-x86_64";
const QEMU_IMG_WINDOWS: &str = "qemu-img.exe";
const QEMU_IMG_UNIX: &str = "qemu-img";

#[derive(Debug, Default, Clone, Copy)]
pub struct QemuDiscoveryTool;

impl QemuDiscoveryTool {
    fn executable_name(windows_name: &'static str, unix_name: &'static str) -> &'static str {
        if cfg!(windows) {
            windows_name
        } else {
            unix_name
        }
    }

    fn find_in_path<S: AsRef<OsStr>>(executable: S) -> Option<PathBuf> {
        let executable = Path::new(executable.as_ref());

        if executable.components().count() > 1 && executable.is_file() {
            return Some(executable.to_path_buf());
        }

        let path = std::env::var_os("PATH")?;

        std::env::split_paths(&path)
            .map(|directory| directory.join(executable))
            .find(|candidate| candidate.is_file())
    }

    fn version_text(system_binary: &Path) -> Result<String, HypervisorProbeError> {
        let output = Command::new(system_binary)
            .arg("--version")
            .output()
            .map_err(|error| HypervisorProbeError::ExecutionFailed(error.to_string()))?;

        if !output.status.success() {
            return Err(HypervisorProbeError::ExecutionFailed(
                String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    }
}

impl HypervisorProbePort for QemuDiscoveryTool {
    fn probe(&self) -> Result<HypervisorInstallation, HypervisorProbeError> {
        let system_name = Self::executable_name(QEMU_SYSTEM_WINDOWS, QEMU_SYSTEM_UNIX);
        let img_name = Self::executable_name(QEMU_IMG_WINDOWS, QEMU_IMG_UNIX);

        let system_binary = Self::find_in_path(system_name).ok_or(HypervisorProbeError::NotFound)?;
        let disk_binary = Self::find_in_path(img_name);
        let version_text = Self::version_text(&system_binary)?;

        Ok(HypervisorInstallation {
            kind: HypervisorKind::Qemu,
            system_binary,
            disk_binary,
            version_text,
        })
    }
}
