// # 📄 Dosya Yolu: /turkuazvm/crates/gpu/src/domain/profile.rs
// # 📌 Amac: Gaming GPU backend secimi ve runtime profile kurallarini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Software, virtio 2D, VirGL/Venus ve experimental GfxStream backendlerini magic string olmadan tasir
// # Bagimli Oldugu Katman: Service | Tool

const MIN_HOSTMEM_MIB: u64 = 256;
const MAX_HOSTMEM_MIB: u64 = 8192;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuBackendPreference {
    Auto,
    Software,
    Virtio2d,
    VirglVenus,
    Gfxstream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuBackend {
    Software,
    Virtio2d,
    VirglVenus,
    Gfxstream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GamingGpuPolicy {
    pub preference: GpuBackendPreference,
    pub hostmem_mib: u64,
    pub allow_experimental_android_gfxstream: bool,
}

impl GamingGpuPolicy {
    pub fn create(
        preference: GpuBackendPreference,
        hostmem_mib: u64,
        allow_experimental_android_gfxstream: bool,
    ) -> Result<Self, GpuPolicyError> {
        if !(MIN_HOSTMEM_MIB..=MAX_HOSTMEM_MIB).contains(&hostmem_mib) {
            return Err(GpuPolicyError::InvalidHostMemory(hostmem_mib));
        }
        Ok(Self {
            preference,
            hostmem_mib,
            allow_experimental_android_gfxstream,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedGamingGpuProfile {
    pub backend: GpuBackend,
    pub hostmem_mib: u64,
    pub experimental: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuPolicyError {
    InvalidHostMemory(u64),
    RequestedBackendUnavailable(GpuBackendPreference),
    ExperimentalBackendDisabled,
}
