// # 📄 Dosya Yolu: /turkuazvm/crates/gpu/src/services/gaming_gpu_service.rs
// # 📌 Amac: Host/hypervisor GPU capability raporunu birlestirir ve guvenli runtime backendini secer
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Experimental GfxStream'i explicit policy olmadan secmez; destek yoksa typed fallback uygular
// # Bagimli Oldugu Katman: Tool

use crate::domain::capability::{GamingGpuCapabilityReport, GpuHostPlatform};
use crate::domain::profile::{
    GpuBackend, GpuBackendPreference, GpuPolicyError, GamingGpuPolicy, ResolvedGamingGpuProfile,
};
use crate::ports::host_gpu_probe_port::{HostGpuProbeError, HostGpuProbePort};
use crate::ports::hypervisor_gpu_probe_port::{
    HypervisorGpuProbeError, HypervisorGpuProbePort,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GamingGpuServiceError {
    Host(HostGpuProbeError),
    Hypervisor(HypervisorGpuProbeError),
    Policy(GpuPolicyError),
}

pub struct GamingGpuService<H, V>
where
    H: HostGpuProbePort,
    V: HypervisorGpuProbePort,
{
    host_probe: H,
    hypervisor_probe: V,
}

impl<H, V> GamingGpuService<H, V>
where
    H: HostGpuProbePort,
    V: HypervisorGpuProbePort,
{
    pub const fn new(host_probe: H, hypervisor_probe: V) -> Self {
        Self {
            host_probe,
            hypervisor_probe,
        }
    }

    pub fn inspect(&self) -> Result<GamingGpuCapabilityReport, GamingGpuServiceError> {
        Ok(GamingGpuCapabilityReport {
            host: self
                .host_probe
                .probe_host_gpu()
                .map_err(GamingGpuServiceError::Host)?,
            hypervisor: self
                .hypervisor_probe
                .probe_hypervisor_gpu()
                .map_err(GamingGpuServiceError::Hypervisor)?,
        })
    }

    pub fn resolve(
        &self,
        policy: GamingGpuPolicy,
    ) -> Result<(GamingGpuCapabilityReport, ResolvedGamingGpuProfile), GamingGpuServiceError> {
        let report = self.inspect()?;
        let backend = match policy.preference {
            GpuBackendPreference::Auto => Self::auto_backend(&report, policy),
            GpuBackendPreference::Software => Ok(GpuBackend::Software),
            GpuBackendPreference::Virtio2d => {
                Self::require(report.hypervisor.virtio_gpu_2d, GpuBackendPreference::Virtio2d)
                    .map(|()| GpuBackend::Virtio2d)
            }
            GpuBackendPreference::VirglVenus => {
                let supported = report.host.platform == GpuHostPlatform::Linux
                    && report.host.vulkan_loader_available
                    && report.hypervisor.virgl
                    && report.hypervisor.venus;
                Self::require(supported, GpuBackendPreference::VirglVenus)
                    .map(|()| GpuBackend::VirglVenus)
            }
            GpuBackendPreference::Gfxstream => {
                if !policy.allow_experimental_android_gfxstream {
                    Err(GpuPolicyError::ExperimentalBackendDisabled)
                } else {
                    let supported = report.host.platform == GpuHostPlatform::Linux
                        && report.host.vulkan_loader_available
                        && report.hypervisor.rutabaga
                        && report.hypervisor.gfxstream_vulkan
                        && report.hypervisor.android_gfxstream_experimental;
                    Self::require(supported, GpuBackendPreference::Gfxstream)
                        .map(|()| GpuBackend::Gfxstream)
                }
            }
        }
        .map_err(GamingGpuServiceError::Policy)?;

        Ok((
            report,
            ResolvedGamingGpuProfile {
                backend,
                hostmem_mib: policy.hostmem_mib,
                experimental: backend == GpuBackend::Gfxstream,
            },
        ))
    }

    fn auto_backend(
        report: &GamingGpuCapabilityReport,
        policy: GamingGpuPolicy,
    ) -> Result<GpuBackend, GpuPolicyError> {
        if policy.allow_experimental_android_gfxstream
            && report.host.platform == GpuHostPlatform::Linux
            && report.host.vulkan_loader_available
            && report.hypervisor.rutabaga
            && report.hypervisor.gfxstream_vulkan
            && report.hypervisor.android_gfxstream_experimental
        {
            return Ok(GpuBackend::Gfxstream);
        }
        if report.host.platform == GpuHostPlatform::Linux
            && report.host.vulkan_loader_available
            && report.hypervisor.virgl
            && report.hypervisor.venus
        {
            return Ok(GpuBackend::VirglVenus);
        }
        if report.hypervisor.virtio_gpu_2d {
            return Ok(GpuBackend::Virtio2d);
        }
        Ok(GpuBackend::Software)
    }

    fn require(available: bool, requested: GpuBackendPreference) -> Result<(), GpuPolicyError> {
        if available {
            Ok(())
        } else {
            Err(GpuPolicyError::RequestedBackendUnavailable(requested))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::capability::{HostGpuCapabilities, HypervisorGpuCapabilities};
    use crate::ports::host_gpu_probe_port::HostGpuProbePort;
    use crate::ports::hypervisor_gpu_probe_port::HypervisorGpuProbePort;

    struct FakeHost;
    impl HostGpuProbePort for FakeHost {
        fn probe_host_gpu(&self) -> Result<HostGpuCapabilities, HostGpuProbeError> {
            Ok(HostGpuCapabilities {
                platform: GpuHostPlatform::Linux,
                vulkan_loader_available: true,
                vulkan_probe_available: true,
                vulkan_summary: Some(String::from("Vulkan test")),
                opengl_probe_available: true,
            })
        }
    }

    struct FakeHypervisor;
    impl HypervisorGpuProbePort for FakeHypervisor {
        fn probe_hypervisor_gpu(&self) -> Result<HypervisorGpuCapabilities, HypervisorGpuProbeError> {
            Ok(HypervisorGpuCapabilities {
                virtio_gpu_2d: true,
                virgl: true,
                venus: true,
                rutabaga: true,
                gfxstream_vulkan: true,
                android_gfxstream_experimental: true,
            })
        }
    }

    #[test]
    fn auto_does_not_select_experimental_gfxstream_without_policy() {
        let service = GamingGpuService::new(FakeHost, FakeHypervisor);
        let policy = GamingGpuPolicy::create(GpuBackendPreference::Auto, 1024, false)
            .expect("policy must be valid");
        let (_, resolved) = service.resolve(policy).expect("resolution must succeed");
        assert_eq!(resolved.backend, GpuBackend::VirglVenus);
    }

    #[test]
    fn explicit_gfxstream_requires_experimental_policy() {
        let service = GamingGpuService::new(FakeHost, FakeHypervisor);
        let policy = GamingGpuPolicy::create(GpuBackendPreference::Gfxstream, 1024, false)
            .expect("policy must be valid");
        let error = service.resolve(policy).expect_err("gfxstream must be blocked");
        assert_eq!(
            error,
            GamingGpuServiceError::Policy(GpuPolicyError::ExperimentalBackendDisabled)
        );
    }
}
