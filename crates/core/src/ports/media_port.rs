// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/ports/media_port.rs
// # 📌 Amac: Guest ISO media storage adapter contractini atomik import ve rollback destegiyle tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: ISO import islemlerini Created, Unchanged ve Replaced durumlariyla takip ederek tekrar baglama ve guvenli degistirme akislarini destekler
// # Bagimli Oldugu Katman: Service | Tool

use std::path::Path;

use crate::domain::guest_boot::IsoAttachment;
use crate::domain::virtual_machine::VmId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaImportState {
    Created,
    Unchanged,
    Replaced,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaError {
    SourceNotFound(String),
    AlreadyExists(String),
    ImportFailed(String),
    DeleteFailed(String),
    RollbackFailed(String),
    CommitFailed(String),
}

pub trait MediaPort {
    fn import_iso(
        &self,
        vm_id: &VmId,
        source_path: &Path,
        attachment: &IsoAttachment,
    ) -> Result<MediaImportState, MediaError>;

    fn commit_iso(
        &self,
        vm_id: &VmId,
        attachment: &IsoAttachment,
        state: MediaImportState,
    ) -> Result<(), MediaError>;

    fn rollback_iso(
        &self,
        vm_id: &VmId,
        attachment: &IsoAttachment,
        state: MediaImportState,
    ) -> Result<(), MediaError>;

    fn delete_iso(&self, vm_id: &VmId, attachment: &IsoAttachment) -> Result<(), MediaError>;
}
