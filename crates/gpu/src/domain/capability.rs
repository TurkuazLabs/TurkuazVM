// # 📄 Dosya Yolu: /turkuazvm/crates/gpu/src/domain/capability.rs
// # 📌 Amac: Host ve hypervisor GPU capability domain modellerini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Vulkan/OpenGL ve virtio-gpu backend desteklerini typed capability raporu olarak tasir
// # Bagimli Oldugu Katman: Service | Tool

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuHostPlatform {
    Windows,
    Linux,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostGpuCapabilities {
    pub platform: GpuHostPlatform,
    pub vulkan_loader_available: bool,
    pub vulkan_probe_available: bool,
    pub vulkan_summary: Option<String>,
    pub opengl_probe_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HypervisorGpuCapabilities {
    pub virtio_gpu_2d: bool,
    pub virgl: bool,
    pub venus: bool,
    pub rutabaga: bool,
    pub gfxstream_vulkan: bool,
    pub android_gfxstream_experimental: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GamingGpuCapabilityReport {
    pub host: HostGpuCapabilities,
    pub hypervisor: HypervisorGpuCapabilities,
}
