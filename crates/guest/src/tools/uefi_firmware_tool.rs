// # 📄 Dosya Yolu: /turkuazvm/crates/guest/src/tools/uefi_firmware_tool.rs
// # 📌 Amac: UEFI code ve VM basina writable vars firmware dosyalarini hazirlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: FirmwarePort contractini global code cache ve VM-local vars kopyasi ile uygular
// # Bagimli Oldugu Katman: Tool

use std::fs;
use std::path::PathBuf;

use turkuazvm_core::domain::guest_boot::UefiFirmware;
use turkuazvm_core::domain::virtual_machine::VmId;
use turkuazvm_core::ports::firmware_port::{FirmwareError, FirmwarePort, PreparedUefiFirmware};

const DIR_MACHINES: &str = "machines";

#[derive(Debug, Clone)]
pub struct UefiFirmwareSettings {
    pub data_root: PathBuf,
    pub code_source_path: PathBuf,
    pub vars_template_source_path: PathBuf,
    pub code_relative_path: String,
    pub vars_relative_path: String,
}

pub struct UefiFirmwareTool {
    settings: UefiFirmwareSettings,
}

impl UefiFirmwareTool {
    pub const fn new(settings: UefiFirmwareSettings) -> Self {
        Self { settings }
    }

    fn code_target(&self) -> PathBuf {
        self.settings
            .data_root
            .join(&self.settings.code_relative_path)
    }

    fn vars_target(&self, vm_id: &VmId) -> PathBuf {
        self.settings
            .data_root
            .join(DIR_MACHINES)
            .join(vm_id.as_str())
            .join(&self.settings.vars_relative_path)
    }

    fn ensure_code(&self) -> Result<(), FirmwareError> {
        let target = self.code_target();
        if target.is_file() {
            return Ok(());
        }
        if !self.settings.code_source_path.is_file() {
            return Err(FirmwareError::SourceNotFound(
                self.settings.code_source_path.display().to_string(),
            ));
        }
        let parent = target.parent().ok_or_else(|| {
            FirmwareError::PrepareFailed(String::from("UEFI code target parent is unavailable"))
        })?;
        fs::create_dir_all(parent)
            .map_err(|error| FirmwareError::PrepareFailed(error.to_string()))?;
        fs::copy(&self.settings.code_source_path, &target)
            .map_err(|error| FirmwareError::PrepareFailed(error.to_string()))?;
        Ok(())
    }
}

impl FirmwarePort for UefiFirmwareTool {
    fn prepare_uefi(&self, vm_id: &VmId) -> Result<PreparedUefiFirmware, FirmwareError> {
        self.ensure_code()?;
        if !self.settings.vars_template_source_path.is_file() {
            return Err(FirmwareError::SourceNotFound(
                self.settings.vars_template_source_path.display().to_string(),
            ));
        }

        let vars_target = self.vars_target(vm_id);
        let vars_created = !vars_target.is_file();
        if vars_created {
            let parent = vars_target.parent().ok_or_else(|| {
                FirmwareError::PrepareFailed(String::from(
                    "UEFI vars target parent is unavailable",
                ))
            })?;
            fs::create_dir_all(parent)
                .map_err(|error| FirmwareError::PrepareFailed(error.to_string()))?;
            fs::copy(&self.settings.vars_template_source_path, &vars_target)
                .map_err(|error| FirmwareError::PrepareFailed(error.to_string()))?;
        }

        let firmware = UefiFirmware::create(
            self.settings.code_relative_path.clone(),
            self.settings.vars_relative_path.clone(),
        )
        .map_err(|error| FirmwareError::PrepareFailed(format!("{error:?}")))?;
        Ok(PreparedUefiFirmware {
            firmware,
            vars_created,
        })
    }

    fn cleanup_uefi(
        &self,
        vm_id: &VmId,
        prepared: &PreparedUefiFirmware,
    ) -> Result<(), FirmwareError> {
        if !prepared.vars_created {
            return Ok(());
        }
        let vars_target = self.vars_target(vm_id);
        if !vars_target.exists() {
            return Ok(());
        }
        fs::remove_file(vars_target).map_err(|error| FirmwareError::CleanupFailed(error.to_string()))
    }
}
