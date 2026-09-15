// # 📄 Dosya Yolu: /turkuazvm/crates/platform/src/tools/native_host_probe_tool.rs
// # 📌 Amac: Calisan host isletim sistemi ve mimarisini Rust std sabitlerinden okur
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: HostProbePort icin Windows ve Linux uyumlu native adapter saglar
// # Bagimli Oldugu Katman: Service | Tool

use turkuazvm_core::domain::host::{HostArchitecture, HostInfo, HostPlatform};
use turkuazvm_core::ports::host_probe_port::HostProbePort;

#[derive(Debug, Default, Clone, Copy)]
pub struct NativeHostProbeTool;

impl HostProbePort for NativeHostProbeTool {
    fn probe_host(&self) -> HostInfo {
        let platform = match std::env::consts::OS {
            "windows" => HostPlatform::Windows,
            "linux" => HostPlatform::Linux,
            other => HostPlatform::Unsupported(other.to_owned()),
        };

        let architecture = match std::env::consts::ARCH {
            "x86_64" => HostArchitecture::X86_64,
            "aarch64" => HostArchitecture::Aarch64,
            other => HostArchitecture::Unsupported(other.to_owned()),
        };

        HostInfo {
            platform,
            architecture,
        }
    }
}
