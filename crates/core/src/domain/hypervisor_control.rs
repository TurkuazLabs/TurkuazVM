// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/domain/hypervisor_control.rs
// # 📌 Amac: Hypervisor kontrol kanali runtime bilgilerini domain seviyesinde temsil eder
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: QMP gibi dis protokollerin wire detaylarini core katmanindan uzak tutar
// # Bagimli Oldugu Katman: Service | Tool

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HypervisorRuntimeVersion {
    pub major: u64,
    pub minor: u64,
    pub micro: u64,
    pub package: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HypervisorControlReport {
    pub version: HypervisorRuntimeVersion,
    pub supported_commands: Vec<String>,
}
