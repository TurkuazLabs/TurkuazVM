// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/domain/runtime_registration.rs
// # 📌 Amac: Engine restart sonrasi hypervisor process reattach icin kalici runtime kimlik modelini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: PID, process start token, executable, QMP endpoint ve display session bilgisini Tool detayindan bagimsiz tutar
// # Bagimli Oldugu Katman: Service | Repo | Tool

use std::net::SocketAddr;
use std::path::PathBuf;

use crate::domain::display::DisplayRuntimeInfo;
use crate::domain::virtual_machine::VmId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeRegistration {
    pub vm_id: VmId,
    pub process_id: u32,
    pub process_start_token: String,
    pub executable_path: PathBuf,
    pub qmp_endpoint: SocketAddr,
    pub display: Option<DisplayRuntimeInfo>,
}
