// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/ports/host_probe_port.rs
// # 📌 Amac: Host bilgisini saglayan adapter icin port contractini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Application service katmanini std platform detaylarindan ayirir
// # Bagimli Oldugu Katman: Service | Tool

use crate::domain::host::HostInfo;

pub trait HostProbePort {
    fn probe_host(&self) -> HostInfo;
}
