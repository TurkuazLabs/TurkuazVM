// # 📄 Dosya Yolu: /turkuazvm/crates/guest/src/tools/local_guest_media_tool.rs
// # 📌 Amac: Installer ISO dosyasini VM machine root altina atomik ve tekrar calistirilabilir sekilde import eder
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Ayni ISO icin no-op, farkli ISO icin backup tabanli replace ve rollback davranisiyla MediaPort contractini uygular
// # Bagimli Oldugu Katman: Tool

use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::Read;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use turkuazvm_core::domain::guest_boot::IsoAttachment;
use turkuazvm_core::domain::virtual_machine::VmId;
use turkuazvm_core::ports::media_port::{MediaError, MediaImportState, MediaPort};

const DIR_MACHINES: &str = "machines";
const TEMP_SUFFIX: &str = ".turkuazvm-part";
const BACKUP_SUFFIX: &str = ".turkuazvm-backup";
const HASH_BUFFER_SIZE: usize = 1024 * 1024;

#[derive(Debug, Clone)]
pub struct LocalGuestMediaSettings {
    pub data_root: PathBuf,
}

pub struct LocalGuestMediaTool {
    settings: LocalGuestMediaSettings,
}

impl LocalGuestMediaTool {
    pub const fn new(settings: LocalGuestMediaSettings) -> Self {
        Self { settings }
    }

    fn target_path(&self, vm_id: &VmId, attachment: &IsoAttachment) -> PathBuf {
        self.settings
            .data_root
            .join(DIR_MACHINES)
            .join(vm_id.as_str())
            .join(attachment.relative_path())
    }

    fn sibling_with_suffix(path: &Path, suffix: &str) -> PathBuf {
        let mut value = path.as_os_str().to_os_string();
        value.push(OsStr::new(suffix));
        PathBuf::from(value)
    }

    fn sha256(path: &Path) -> Result<[u8; 32], MediaError> {
        let mut file = File::open(path).map_err(|error| MediaError::ImportFailed(error.to_string()))?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0_u8; HASH_BUFFER_SIZE];
        loop {
            let count = file
                .read(&mut buffer)
                .map_err(|error| MediaError::ImportFailed(error.to_string()))?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }
        let digest = hasher.finalize();
        let mut output = [0_u8; 32];
        output.copy_from_slice(&digest);
        Ok(output)
    }

    fn recover_stale_backup(target: &Path, backup: &Path) -> Result<(), MediaError> {
        if target.exists() && backup.exists() {
            fs::remove_file(backup).map_err(|error| MediaError::ImportFailed(error.to_string()))?;
        } else if !target.exists() && backup.exists() {
            fs::rename(backup, target).map_err(|error| MediaError::ImportFailed(error.to_string()))?;
        }
        Ok(())
    }

    fn copy_to_temp(source_path: &Path, temp: &Path) -> Result<(), MediaError> {
        if temp.exists() {
            fs::remove_file(temp).map_err(|error| MediaError::ImportFailed(error.to_string()))?;
        }
        fs::copy(source_path, temp).map_err(|error| MediaError::ImportFailed(error.to_string()))?;
        OpenOptions::new()
            .write(true)
            .open(temp)
            .and_then(|file| file.sync_all())
            .map_err(|error| MediaError::ImportFailed(error.to_string()))?;
        Ok(())
    }
}

impl MediaPort for LocalGuestMediaTool {
    fn import_iso(
        &self,
        vm_id: &VmId,
        source_path: &Path,
        attachment: &IsoAttachment,
    ) -> Result<MediaImportState, MediaError> {
        if !source_path.is_file() {
            return Err(MediaError::SourceNotFound(source_path.display().to_string()));
        }

        let target = self.target_path(vm_id, attachment);
        let parent = target
            .parent()
            .ok_or_else(|| MediaError::ImportFailed(String::from("ISO target parent is unavailable")))?;
        fs::create_dir_all(parent).map_err(|error| MediaError::ImportFailed(error.to_string()))?;

        let temp = Self::sibling_with_suffix(&target, TEMP_SUFFIX);
        let backup = Self::sibling_with_suffix(&target, BACKUP_SUFFIX);
        Self::recover_stale_backup(&target, &backup)?;

        if target.is_file() && Self::sha256(source_path)? == Self::sha256(&target)? {
            if temp.exists() {
                let _ = fs::remove_file(&temp);
            }
            return Ok(MediaImportState::Unchanged);
        }

        Self::copy_to_temp(source_path, &temp)?;

        if target.exists() {
            fs::rename(&target, &backup).map_err(|error| MediaError::ImportFailed(error.to_string()))?;
            if let Err(error) = fs::rename(&temp, &target) {
                let _ = fs::rename(&backup, &target);
                let _ = fs::remove_file(&temp);
                return Err(MediaError::ImportFailed(error.to_string()));
            }
            return Ok(MediaImportState::Replaced);
        }

        fs::rename(&temp, &target).map_err(|error| {
            let _ = fs::remove_file(&temp);
            MediaError::ImportFailed(error.to_string())
        })?;
        Ok(MediaImportState::Created)
    }

    fn commit_iso(
        &self,
        vm_id: &VmId,
        attachment: &IsoAttachment,
        state: MediaImportState,
    ) -> Result<(), MediaError> {
        if state != MediaImportState::Replaced {
            return Ok(());
        }
        let target = self.target_path(vm_id, attachment);
        let backup = Self::sibling_with_suffix(&target, BACKUP_SUFFIX);
        if backup.exists() {
            fs::remove_file(&backup).map_err(|error| MediaError::CommitFailed(error.to_string()))?;
        }
        Ok(())
    }

    fn rollback_iso(
        &self,
        vm_id: &VmId,
        attachment: &IsoAttachment,
        state: MediaImportState,
    ) -> Result<(), MediaError> {
        let target = self.target_path(vm_id, attachment);
        match state {
            MediaImportState::Unchanged => Ok(()),
            MediaImportState::Created => {
                if target.exists() {
                    fs::remove_file(&target)
                        .map_err(|error| MediaError::RollbackFailed(error.to_string()))?;
                }
                Ok(())
            }
            MediaImportState::Replaced => {
                let backup = Self::sibling_with_suffix(&target, BACKUP_SUFFIX);
                if target.exists() {
                    fs::remove_file(&target)
                        .map_err(|error| MediaError::RollbackFailed(error.to_string()))?;
                }
                if !backup.exists() {
                    return Err(MediaError::RollbackFailed(String::from(
                        "ISO replacement backup is unavailable",
                    )));
                }
                fs::rename(&backup, &target)
                    .map_err(|error| MediaError::RollbackFailed(error.to_string()))
            }
        }
    }

    fn delete_iso(&self, vm_id: &VmId, attachment: &IsoAttachment) -> Result<(), MediaError> {
        let target = self.target_path(vm_id, attachment);
        let backup = Self::sibling_with_suffix(&target, BACKUP_SUFFIX);
        let temp = Self::sibling_with_suffix(&target, TEMP_SUFFIX);
        for path in [target.as_path(), backup.as_path(), temp.as_path()] {
            if path.exists() {
                fs::remove_file(path).map_err(|error| MediaError::DeleteFailed(error.to_string()))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    use turkuazvm_core::domain::guest_boot::MediaId;

    fn test_root() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be valid")
            .as_nanos();
        std::env::temp_dir().join(format!("turkuazvm-media-{}-{nonce}", std::process::id()))
    }

    fn attachment() -> IsoAttachment {
        IsoAttachment::create(
            MediaId::parse("installer").expect("media id must be valid"),
            "media/installer.iso",
        )
        .expect("attachment must be valid")
    }

    #[test]
    fn iso_is_copied_under_machine_root() {
        let root = test_root();
        fs::create_dir_all(&root).expect("root must be created");
        let source = root.join("source.iso");
        fs::write(&source, b"iso").expect("source must be created");
        let tool = LocalGuestMediaTool::new(LocalGuestMediaSettings {
            data_root: root.clone(),
        });
        let vm_id = VmId::parse("vm-a").expect("vm id must be valid");
        let attachment = attachment();

        let state = tool
            .import_iso(&vm_id, &source, &attachment)
            .expect("import must succeed");
        assert_eq!(state, MediaImportState::Created);
        tool.commit_iso(&vm_id, &attachment, state)
            .expect("commit must succeed");
        assert!(root.join("machines/vm-a/media/installer.iso").is_file());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn same_iso_is_idempotent() {
        let root = test_root();
        fs::create_dir_all(&root).expect("root must be created");
        let source = root.join("source.iso");
        fs::write(&source, b"same-iso").expect("source must be created");
        let tool = LocalGuestMediaTool::new(LocalGuestMediaSettings {
            data_root: root.clone(),
        });
        let vm_id = VmId::parse("vm-a").expect("vm id must be valid");
        let attachment = attachment();

        let first = tool
            .import_iso(&vm_id, &source, &attachment)
            .expect("first import must succeed");
        tool.commit_iso(&vm_id, &attachment, first)
            .expect("first commit must succeed");
        let second = tool
            .import_iso(&vm_id, &source, &attachment)
            .expect("second import must succeed");
        assert_eq!(second, MediaImportState::Unchanged);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn replacement_can_be_committed() {
        let root = test_root();
        fs::create_dir_all(&root).expect("root must be created");
        let old_source = root.join("old.iso");
        let new_source = root.join("new.iso");
        fs::write(&old_source, b"old-iso").expect("old source must be created");
        fs::write(&new_source, b"new-iso").expect("new source must be created");
        let tool = LocalGuestMediaTool::new(LocalGuestMediaSettings {
            data_root: root.clone(),
        });
        let vm_id = VmId::parse("vm-a").expect("vm id must be valid");
        let attachment = attachment();

        let first = tool
            .import_iso(&vm_id, &old_source, &attachment)
            .expect("old import must succeed");
        tool.commit_iso(&vm_id, &attachment, first)
            .expect("old commit must succeed");
        let replacement = tool
            .import_iso(&vm_id, &new_source, &attachment)
            .expect("replacement must succeed");
        assert_eq!(replacement, MediaImportState::Replaced);
        tool.commit_iso(&vm_id, &attachment, replacement)
            .expect("replacement commit must succeed");
        assert_eq!(
            fs::read(root.join("machines/vm-a/media/installer.iso")).expect("target must be readable"),
            b"new-iso"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn replacement_rollback_restores_previous_iso() {
        let root = test_root();
        fs::create_dir_all(&root).expect("root must be created");
        let old_source = root.join("old.iso");
        let new_source = root.join("new.iso");
        fs::write(&old_source, b"old-iso").expect("old source must be created");
        fs::write(&new_source, b"new-iso").expect("new source must be created");
        let tool = LocalGuestMediaTool::new(LocalGuestMediaSettings {
            data_root: root.clone(),
        });
        let vm_id = VmId::parse("vm-a").expect("vm id must be valid");
        let attachment = attachment();

        let first = tool
            .import_iso(&vm_id, &old_source, &attachment)
            .expect("old import must succeed");
        tool.commit_iso(&vm_id, &attachment, first)
            .expect("old commit must succeed");
        let replacement = tool
            .import_iso(&vm_id, &new_source, &attachment)
            .expect("replacement must succeed");
        tool.rollback_iso(&vm_id, &attachment, replacement)
            .expect("rollback must succeed");
        assert_eq!(
            fs::read(root.join("machines/vm-a/media/installer.iso")).expect("target must be readable"),
            b"old-iso"
        );
        let _ = fs::remove_dir_all(root);
    }
}
