// # 📄 Dosya Yolu: /turkuazvm/crates/platform/src/tools/native_gpu_probe_tool.rs
// # 📌 Amac: Windows/Linux host grafik runtime capability bilgisini native dosya ve tool kontrolleriyle toplar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Vulkan loader, vulkaninfo ve OpenGL runtime varligini Gaming GPU portuna adapte eder
// # Bagimli Oldugu Katman: Service | Tool

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use turkuazvm_gpu::domain::capability::{GpuHostPlatform, HostGpuCapabilities};
use turkuazvm_gpu::ports::host_gpu_probe_port::{HostGpuProbeError, HostGpuProbePort};

const VULKANINFO_WINDOWS: &str = "vulkaninfo.exe";
const VULKANINFO_UNIX: &str = "vulkaninfo";

#[derive(Debug, Default, Clone, Copy)]
pub struct NativeGpuProbeTool;

impl NativeGpuProbeTool {
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

    fn platform() -> GpuHostPlatform {
        match std::env::consts::OS {
            "windows" => GpuHostPlatform::Windows,
            "linux" => GpuHostPlatform::Linux,
            _ => GpuHostPlatform::Unsupported,
        }
    }

    fn vulkan_loader_available(platform: GpuHostPlatform) -> bool {
        match platform {
            GpuHostPlatform::Windows => std::env::var_os("SystemRoot")
                .map(PathBuf::from)
                .map(|root| root.join("System32").join("vulkan-1.dll").is_file())
                .unwrap_or(false),
            GpuHostPlatform::Linux => [
                "/usr/lib/x86_64-linux-gnu/libvulkan.so.1",
                "/usr/lib64/libvulkan.so.1",
                "/usr/lib/libvulkan.so.1",
            ]
            .iter()
            .any(|path| Path::new(path).is_file()),
            GpuHostPlatform::Unsupported => false,
        }
    }

    fn opengl_runtime_available(platform: GpuHostPlatform) -> bool {
        match platform {
            GpuHostPlatform::Windows => std::env::var_os("SystemRoot")
                .map(PathBuf::from)
                .map(|root| root.join("System32").join("opengl32.dll").is_file())
                .unwrap_or(false),
            GpuHostPlatform::Linux => [
                "/usr/lib/x86_64-linux-gnu/libGL.so.1",
                "/usr/lib64/libGL.so.1",
                "/usr/lib/libGL.so.1",
            ]
            .iter()
            .any(|path| Path::new(path).is_file()),
            GpuHostPlatform::Unsupported => false,
        }
    }

    fn vulkan_summary(binary: &Path) -> Option<String> {
        let output = Command::new(binary).arg("--summary").output().ok()?;
        if !output.status.success() {
            return None;
        }
        let text = String::from_utf8_lossy(&output.stdout);
        let summary = text
            .lines()
            .filter(|line| !line.trim().is_empty())
            .take(12)
            .collect::<Vec<_>>()
            .join(" | ");
        (!summary.is_empty()).then_some(summary)
    }
}

impl HostGpuProbePort for NativeGpuProbeTool {
    fn probe_host_gpu(&self) -> Result<HostGpuCapabilities, HostGpuProbeError> {
        let platform = Self::platform();
        let vulkaninfo_name = if platform == GpuHostPlatform::Windows {
            VULKANINFO_WINDOWS
        } else {
            VULKANINFO_UNIX
        };
        let vulkaninfo = Self::find_in_path(vulkaninfo_name);
        let vulkan_summary = vulkaninfo
            .as_deref()
            .and_then(Self::vulkan_summary);

        Ok(HostGpuCapabilities {
            platform,
            vulkan_loader_available: Self::vulkan_loader_available(platform) || vulkaninfo.is_some(),
            vulkan_probe_available: vulkaninfo.is_some(),
            vulkan_summary,
            opengl_probe_available: Self::opengl_runtime_available(platform),
        })
    }
}
