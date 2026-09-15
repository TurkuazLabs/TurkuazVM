// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/domain/runtime_media.rs
// # 📌 Amac: VM baslatma aninda hypervisor adapterine tasinan gecici runtime medya planini tanimlar
// # 📌 Modul - Rust
// # Version: 0.40.13
// # Aciklama: QEMU composite Android medyasi ile resmi Android SDK Emulator AVD runtime planini typed olarak ayirir
// # Bagimli Oldugu Katman: Service | Tool

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidRuntimeMediaPlan {
    pub bootloader_path: PathBuf,
    pub pflash_path: PathBuf,
    pub os_disk_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidSdkEmulatorRuntimeMediaPlan {
    pub emulator_binary: PathBuf,
    pub avd_home: PathBuf,
    pub avd_name: String,
    pub system_image_dir: PathBuf,
    pub console_port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmRuntimeMediaPlan {
    Android(AndroidRuntimeMediaPlan),
    AndroidSdkEmulator(AndroidSdkEmulatorRuntimeMediaPlan),
}

impl VmRuntimeMediaPlan {
    pub const fn has_boot_disk(&self) -> bool {
        matches!(self, Self::Android(_) | Self::AndroidSdkEmulator(_))
    }

    pub const fn uses_host_network_runtime(&self) -> bool {
        !matches!(self, Self::AndroidSdkEmulator(_))
    }
}
