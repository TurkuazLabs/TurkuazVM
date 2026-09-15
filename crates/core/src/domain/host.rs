// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/domain/host.rs
// # 📌 Amac: Host isletim sistemi ve CPU mimarisi domain modellerini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Platforma ozel magic string kullanmadan host bilgisini temsil eder
// # Bagimli Oldugu Katman: Service

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostPlatform {
    Windows,
    Linux,
    Unsupported(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostArchitecture {
    X86_64,
    Aarch64,
    Unsupported(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostInfo {
    pub platform: HostPlatform,
    pub architecture: HostArchitecture,
}
