// # 📄 Dosya Yolu: /turkuazvm/crates/qemu/src/tools/qemu_gpu_probe_tool.rs
// # 📌 Amac: Kurulu QEMU binary'sindeki virtio-gpu accelerated backend capability bilgisini probe eder
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Device help ve property help ciktilarindan 2D, VirGL/Venus ve rutabaga/GfxStream destegini typed rapora cevirir
// # Bagimli Oldugu Katman: Service | Tool

use std::path::PathBuf;
use std::process::Command;

use turkuazvm_gpu::domain::capability::HypervisorGpuCapabilities;
use turkuazvm_gpu::ports::hypervisor_gpu_probe_port::{
    HypervisorGpuProbeError, HypervisorGpuProbePort,
};

const DEVICE_VIRTIO_VGA: &str = "virtio-vga";
const DEVICE_VIRTIO_GPU: &str = "virtio-gpu";
const DEVICE_VIRTIO_VGA_GL: &str = "virtio-vga-gl";
const DEVICE_VIRTIO_GPU_GL: &str = "virtio-gpu-gl";
const DEVICE_VIRTIO_GPU_RUTABAGA: &str = "virtio-gpu-rutabaga";
const PROPERTY_VENUS: &str = "venus";
const PROPERTY_GFXSTREAM_VULKAN: &str = "gfxstream-vulkan";
const PROPERTY_GFXSTREAM_GLES: &str = "x-gfxstream-gles";
const PROPERTY_GFXSTREAM_COMPOSER: &str = "x-gfxstream-composer";

#[derive(Debug, Clone)]
pub struct QemuGpuProbeTool {
    system_binary: Option<PathBuf>,
}

impl QemuGpuProbeTool {
    pub fn new(system_binary: Option<PathBuf>) -> Self {
        Self { system_binary }
    }

    fn command_text(&self, args: &[&str]) -> Result<String, HypervisorGpuProbeError> {
        let binary = self
            .system_binary
            .as_ref()
            .ok_or(HypervisorGpuProbeError::HypervisorUnavailable)?;
        let output = Command::new(binary)
            .args(args)
            .output()
            .map_err(|error| HypervisorGpuProbeError::ProbeFailed(error.to_string()))?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        Ok(format!("{stdout}\n{stderr}"))
    }

    fn device_properties(&self, device: &str) -> Result<String, HypervisorGpuProbeError> {
        let specification = format!("{device},help");
        self.command_text(&["-device", specification.as_str()])
    }
}

impl HypervisorGpuProbePort for QemuGpuProbeTool {
    fn probe_hypervisor_gpu(&self) -> Result<HypervisorGpuCapabilities, HypervisorGpuProbeError> {
        let device_help = self.command_text(&["-device", "help"])?;
        let virtio_gpu_2d = device_help.contains(DEVICE_VIRTIO_VGA) || device_help.contains(DEVICE_VIRTIO_GPU);
        let virgl_device = if device_help.contains(DEVICE_VIRTIO_VGA_GL) {
            Some(DEVICE_VIRTIO_VGA_GL)
        } else if device_help.contains(DEVICE_VIRTIO_GPU_GL) {
            Some(DEVICE_VIRTIO_GPU_GL)
        } else {
            None
        };
        let virgl = virgl_device.is_some();
        let rutabaga = device_help.contains(DEVICE_VIRTIO_GPU_RUTABAGA);

        let virgl_properties = if let Some(device) = virgl_device {
            self.device_properties(device).unwrap_or_default()
        } else {
            String::new()
        };
        let rutabaga_properties = if rutabaga {
            self.device_properties(DEVICE_VIRTIO_GPU_RUTABAGA)
                .unwrap_or_default()
        } else {
            String::new()
        };

        Ok(HypervisorGpuCapabilities {
            virtio_gpu_2d,
            virgl,
            venus: virgl && virgl_properties.contains(PROPERTY_VENUS),
            rutabaga,
            gfxstream_vulkan: rutabaga && rutabaga_properties.contains(PROPERTY_GFXSTREAM_VULKAN),
            android_gfxstream_experimental: rutabaga
                && rutabaga_properties.contains(PROPERTY_GFXSTREAM_GLES)
                && rutabaga_properties.contains(PROPERTY_GFXSTREAM_COMPOSER),
        })
    }
}
