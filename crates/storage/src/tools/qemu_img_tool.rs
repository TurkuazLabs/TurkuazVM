// # 📄 Dosya Yolu: /turkuazvm/crates/storage/src/tools/qemu_img_tool.rs
// # 📌 Amac: StoragePort islemlerini qemu-img executable uzerinden uygular
// # 📌 Modul - Rust
// # Version: 0.41.6
// # Aciklama: Yeni QCOW2/RAW diskleri image_root altinda olusturur; eski data_root diskleri icin read/boot/snapshot/clone fallback uyumlulugunu korur
// # Bagimli Oldugu Katman: Tool

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;
use turkuazvm_core::domain::clone::CloneMode;
use turkuazvm_core::domain::disk::{DiskFormat, DiskImage};
use turkuazvm_core::domain::snapshot::SnapshotId;
use turkuazvm_core::domain::virtual_machine::VmId;
use turkuazvm_core::ports::clone_storage_port::{CloneStorageError, CloneStoragePort};
use turkuazvm_core::ports::snapshot_port::{SnapshotPort, SnapshotStorageError};
use turkuazvm_core::ports::storage_port::{StorageError, StorageImageInfo, StoragePort};

const CMD_CREATE: &str = "create";
const CMD_INFO: &str = "info";
const CMD_RESIZE: &str = "resize";
const CMD_SNAPSHOT: &str = "snapshot";
const CMD_CONVERT: &str = "convert";
const ARG_FORMAT: &str = "-f";
const ARG_OUTPUT_JSON: &str = "--output=json";
const ARG_SNAPSHOT_CREATE: &str = "-c";
const ARG_SNAPSHOT_APPLY: &str = "-a";
const ARG_SNAPSHOT_DELETE: &str = "-d";
const ARG_OUTPUT_FORMAT: &str = "-O";
const ARG_BACKING_FILE: &str = "-b";
const ARG_BACKING_FORMAT: &str = "-F";
const DIR_MACHINES: &str = "machines";
const DIR_MEDIA: &str = "media";
const DIR_FIRMWARE: &str = "firmware";
const FORMAT_QCOW2: &str = "qcow2";
const FORMAT_RAW: &str = "raw";

#[derive(Debug, Deserialize)]
struct QemuImgInfoOutput {
    format: String,
    #[serde(rename = "virtual-size")]
    virtual_size: u64,
    #[serde(rename = "actual-size")]
    actual_size: Option<u64>,
}

#[derive(Clone)]
pub struct QemuImgTool {
    binary: PathBuf,
    data_root: PathBuf,
    image_root: PathBuf,
}

impl QemuImgTool {
    pub fn new(binary: PathBuf, data_root: PathBuf, image_root: PathBuf) -> Self {
        Self {
            binary,
            data_root,
            image_root,
        }
    }

    fn data_machine_root(&self, vm_id: &VmId) -> PathBuf {
        self.data_root.join(DIR_MACHINES).join(vm_id.as_str())
    }

    fn image_machine_root(&self, vm_id: &VmId) -> PathBuf {
        self.image_root.join(vm_id.as_str())
    }

    fn resolve_under_machine_root(
        machine_root: &Path,
        image: &DiskImage,
    ) -> Result<PathBuf, StorageError> {
        let resolved = machine_root.join(image.relative_path());
        if !resolved.starts_with(machine_root) {
            return Err(StorageError::UnsafePath(image.relative_path().to_owned()));
        }
        Ok(resolved)
    }

    fn resolve_image_target_path(
        &self,
        vm_id: &VmId,
        image: &DiskImage,
    ) -> Result<PathBuf, StorageError> {
        Self::resolve_under_machine_root(&self.image_machine_root(vm_id), image)
    }

    fn resolve_existing_image_path(
        &self,
        vm_id: &VmId,
        image: &DiskImage,
    ) -> Result<PathBuf, StorageError> {
        let primary = self.resolve_image_target_path(vm_id, image)?;
        if primary.exists() {
            return Ok(primary);
        }

        let legacy = Self::resolve_under_machine_root(&self.data_machine_root(vm_id), image)?;
        if legacy.exists() {
            return Ok(legacy);
        }
        Ok(primary)
    }

    fn ensure_target_parent(path: &Path) -> Result<(), CloneStorageError> {
        let parent = path
            .parent()
            .ok_or_else(|| CloneStorageError::UnsafePath(path.display().to_string()))?;
        fs::create_dir_all(parent)
            .map_err(|error| CloneStorageError::ExecutionFailed(error.to_string()))
    }

    fn copy_directory_recursive(source: &Path, target: &Path) -> Result<(), CloneStorageError> {
        if !source.exists() {
            return Ok(());
        }
        fs::create_dir_all(target)
            .map_err(|error| CloneStorageError::ExecutionFailed(error.to_string()))?;
        for entry in fs::read_dir(source)
            .map_err(|error| CloneStorageError::ExecutionFailed(error.to_string()))?
        {
            let entry = entry
                .map_err(|error| CloneStorageError::ExecutionFailed(error.to_string()))?;
            let source_path = entry.path();
            let target_path = target.join(entry.file_name());
            if source_path.is_dir() {
                Self::copy_directory_recursive(&source_path, &target_path)?;
            } else {
                fs::copy(&source_path, &target_path)
                    .map_err(|error| CloneStorageError::ExecutionFailed(error.to_string()))?;
            }
        }
        Ok(())
    }

    fn format_name(format: DiskFormat) -> &'static str {
        match format {
            DiskFormat::Qcow2 => FORMAT_QCOW2,
            DiskFormat::Raw => FORMAT_RAW,
        }
    }

    fn parse_format(value: &str) -> Result<DiskFormat, StorageError> {
        match value {
            FORMAT_QCOW2 => Ok(DiskFormat::Qcow2),
            FORMAT_RAW => Ok(DiskFormat::Raw),
            other => Err(StorageError::InvalidOutput(format!(
                "Unsupported qemu-img format: {other}"
            ))),
        }
    }

    fn run(&self, arguments: &[String]) -> Result<std::process::Output, StorageError> {
        Command::new(&self.binary)
            .args(arguments)
            .output()
            .map_err(|error| StorageError::ExecutionFailed(error.to_string()))
    }

    fn require_success(output: std::process::Output) -> Result<std::process::Output, StorageError> {
        if output.status.success() {
            return Ok(output);
        }

        Err(StorageError::ExecutionFailed(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ))
    }

    fn inspect_path(&self, path: &Path) -> Result<StorageImageInfo, StorageError> {
        if !path.is_file() {
            return Err(StorageError::NotFound(path.display().to_string()));
        }

        let arguments = vec![
            String::from(CMD_INFO),
            String::from(ARG_OUTPUT_JSON),
            path.display().to_string(),
        ];
        let output = Self::require_success(self.run(&arguments)?)?;
        let parsed: QemuImgInfoOutput = serde_json::from_slice(&output.stdout)
            .map_err(|error| StorageError::InvalidOutput(error.to_string()))?;

        Ok(StorageImageInfo {
            format: Self::parse_format(&parsed.format)?,
            virtual_size_bytes: parsed.virtual_size,
            actual_size_bytes: parsed.actual_size,
        })
    }
}

impl StoragePort for QemuImgTool {
    fn create_image(&self, vm_id: &VmId, image: &DiskImage) -> Result<StorageImageInfo, StorageError> {
        let path = self.resolve_image_target_path(vm_id, image)?;
        if path.exists() {
            return Err(StorageError::AlreadyExists(path.display().to_string()));
        }

        let parent = path
            .parent()
            .ok_or_else(|| StorageError::UnsafePath(path.display().to_string()))?;
        fs::create_dir_all(parent)
            .map_err(|error| StorageError::ExecutionFailed(error.to_string()))?;

        let arguments = vec![
            String::from(CMD_CREATE),
            String::from(ARG_FORMAT),
            String::from(Self::format_name(image.format())),
            path.display().to_string(),
            image.virtual_size_bytes().to_string(),
        ];
        Self::require_success(self.run(&arguments)?)?;
        self.inspect_path(&path)
    }

    fn inspect_image(&self, vm_id: &VmId, image: &DiskImage) -> Result<StorageImageInfo, StorageError> {
        let path = self.resolve_existing_image_path(vm_id, image)?;
        self.inspect_path(&path)
    }

    fn resize_image(
        &self,
        vm_id: &VmId,
        image: &DiskImage,
        new_virtual_size_bytes: u64,
    ) -> Result<StorageImageInfo, StorageError> {
        let path = self.resolve_existing_image_path(vm_id, image)?;
        if !path.is_file() {
            return Err(StorageError::NotFound(path.display().to_string()));
        }

        let arguments = vec![
            String::from(CMD_RESIZE),
            String::from(ARG_FORMAT),
            String::from(Self::format_name(image.format())),
            path.display().to_string(),
            new_virtual_size_bytes.to_string(),
        ];
        Self::require_success(self.run(&arguments)?)?;
        self.inspect_path(&path)
    }

    fn delete_image(&self, vm_id: &VmId, image: &DiskImage) -> Result<(), StorageError> {
        let path = self.resolve_existing_image_path(vm_id, image)?;
        if !path.exists() {
            return Ok(());
        }
        fs::remove_file(&path).map_err(|error| StorageError::ExecutionFailed(error.to_string()))
    }
}


impl SnapshotPort for QemuImgTool {
    fn create_snapshot(
        &self,
        vm_id: &VmId,
        image: &DiskImage,
        snapshot_id: &SnapshotId,
    ) -> Result<(), SnapshotStorageError> {
        if image.format() != DiskFormat::Qcow2 {
            return Err(SnapshotStorageError::UnsupportedFormat(
                image.relative_path().to_owned(),
            ));
        }
        let path = self
            .resolve_existing_image_path(vm_id, image)
            .map_err(map_storage_to_snapshot)?;
        if !path.is_file() {
            return Err(SnapshotStorageError::NotFound(path.display().to_string()));
        }
        let arguments = vec![
            String::from(CMD_SNAPSHOT),
            String::from(ARG_SNAPSHOT_CREATE),
            snapshot_id.as_str().to_owned(),
            path.display().to_string(),
        ];
        Self::require_success(self.run(&arguments).map_err(map_storage_to_snapshot)?)
            .map_err(map_storage_to_snapshot)?;
        Ok(())
    }

    fn restore_snapshot(
        &self,
        vm_id: &VmId,
        image: &DiskImage,
        snapshot_id: &SnapshotId,
    ) -> Result<(), SnapshotStorageError> {
        let path = self
            .resolve_existing_image_path(vm_id, image)
            .map_err(map_storage_to_snapshot)?;
        let arguments = vec![
            String::from(CMD_SNAPSHOT),
            String::from(ARG_SNAPSHOT_APPLY),
            snapshot_id.as_str().to_owned(),
            path.display().to_string(),
        ];
        Self::require_success(self.run(&arguments).map_err(map_storage_to_snapshot)?)
            .map_err(map_storage_to_snapshot)?;
        Ok(())
    }

    fn delete_snapshot(
        &self,
        vm_id: &VmId,
        image: &DiskImage,
        snapshot_id: &SnapshotId,
    ) -> Result<(), SnapshotStorageError> {
        let path = self
            .resolve_existing_image_path(vm_id, image)
            .map_err(map_storage_to_snapshot)?;
        let arguments = vec![
            String::from(CMD_SNAPSHOT),
            String::from(ARG_SNAPSHOT_DELETE),
            snapshot_id.as_str().to_owned(),
            path.display().to_string(),
        ];
        Self::require_success(self.run(&arguments).map_err(map_storage_to_snapshot)?)
            .map_err(map_storage_to_snapshot)?;
        Ok(())
    }
}

impl CloneStoragePort for QemuImgTool {
    fn prepare_clone_target(&self, target_vm_id: &VmId) -> Result<(), CloneStorageError> {
        let target_root = self.image_machine_root(target_vm_id);
        if target_root.exists() {
            return Err(CloneStorageError::AlreadyExists(
                target_root.display().to_string(),
            ));
        }
        fs::create_dir_all(&target_root)
            .map_err(|error| CloneStorageError::ExecutionFailed(error.to_string()))
    }

    fn clone_image(
        &self,
        source_vm_id: &VmId,
        source: &DiskImage,
        target_vm_id: &VmId,
        target: &DiskImage,
        mode: CloneMode,
    ) -> Result<StorageImageInfo, CloneStorageError> {
        let source_path = self
            .resolve_existing_image_path(source_vm_id, source)
            .map_err(map_storage_to_clone)?;
        let target_path = self
            .resolve_image_target_path(target_vm_id, target)
            .map_err(map_storage_to_clone)?;
        if !source_path.is_file() {
            return Err(CloneStorageError::NotFound(source_path.display().to_string()));
        }
        if target_path.exists() {
            return Err(CloneStorageError::AlreadyExists(target_path.display().to_string()));
        }
        Self::ensure_target_parent(&target_path)?;

        let arguments = match mode {
            CloneMode::Full => vec![
                String::from(CMD_CONVERT),
                String::from(ARG_FORMAT),
                String::from(Self::format_name(source.format())),
                String::from(ARG_OUTPUT_FORMAT),
                String::from(Self::format_name(target.format())),
                source_path.display().to_string(),
                target_path.display().to_string(),
            ],
            CloneMode::Linked => {
                if source.format() != DiskFormat::Qcow2 || target.format() != DiskFormat::Qcow2 {
                    return Err(CloneStorageError::UnsupportedFormat(String::from(
                        "Linked clone requires qcow2 source and target",
                    )));
                }
                let target_parent = target_path
                    .parent()
                    .ok_or_else(|| CloneStorageError::UnsafePath(target_path.display().to_string()))?;
                let backing_name = format!(".{}-clone-base.qcow2", target.id().as_str());
                let backing_path = target_parent.join(&backing_name);
                let base_arguments = vec![
                    String::from(CMD_CONVERT),
                    String::from(ARG_FORMAT),
                    String::from(FORMAT_QCOW2),
                    String::from(ARG_OUTPUT_FORMAT),
                    String::from(FORMAT_QCOW2),
                    source_path.display().to_string(),
                    backing_path.display().to_string(),
                ];
                Self::require_success(self.run(&base_arguments).map_err(map_storage_to_clone)?)
                    .map_err(map_storage_to_clone)?;
                let backing_absolute = fs::canonicalize(&backing_path)
                    .map_err(|error| CloneStorageError::ExecutionFailed(error.to_string()))?;
                vec![
                    String::from(CMD_CREATE),
                    String::from(ARG_FORMAT),
                    String::from(FORMAT_QCOW2),
                    String::from(ARG_BACKING_FORMAT),
                    String::from(FORMAT_QCOW2),
                    String::from(ARG_BACKING_FILE),
                    backing_absolute.display().to_string(),
                    target_path.display().to_string(),
                ]
            }
        };
        Self::require_success(self.run(&arguments).map_err(map_storage_to_clone)?)
            .map_err(map_storage_to_clone)?;
        self.inspect_path(&target_path).map_err(map_storage_to_clone)
    }

    fn clone_machine_assets(
        &self,
        source_vm_id: &VmId,
        target_vm_id: &VmId,
    ) -> Result<(), CloneStorageError> {
        let source_root = self.data_machine_root(source_vm_id);
        let target_root = self.data_machine_root(target_vm_id);
        for directory in [DIR_MEDIA, DIR_FIRMWARE] {
            Self::copy_directory_recursive(
                &source_root.join(directory),
                &target_root.join(directory),
            )?;
        }
        Ok(())
    }

    fn cleanup_clone(&self, target_vm_id: &VmId) -> Result<(), CloneStorageError> {
        let image_target_root = self.image_machine_root(target_vm_id);
        if image_target_root.exists() {
            fs::remove_dir_all(&image_target_root)
                .map_err(|error| CloneStorageError::ExecutionFailed(error.to_string()))?;
        }

        let data_target_root = self.data_machine_root(target_vm_id);
        if data_target_root.exists() && data_target_root != image_target_root {
            fs::remove_dir_all(&data_target_root)
                .map_err(|error| CloneStorageError::ExecutionFailed(error.to_string()))?;
        }
        Ok(())
    }
}

fn map_storage_to_snapshot(error: StorageError) -> SnapshotStorageError {
    match error {
        StorageError::NotFound(value) => SnapshotStorageError::NotFound(value),
        StorageError::AlreadyExists(value) => SnapshotStorageError::AlreadyExists(value),
        StorageError::ExecutionFailed(value) => SnapshotStorageError::ExecutionFailed(value),
        StorageError::InvalidOutput(value) => SnapshotStorageError::ExecutionFailed(value),
        StorageError::UnsafePath(value) => SnapshotStorageError::UnsafePath(value),
    }
}

fn map_storage_to_clone(error: StorageError) -> CloneStorageError {
    match error {
        StorageError::NotFound(value) => CloneStorageError::NotFound(value),
        StorageError::AlreadyExists(value) => CloneStorageError::AlreadyExists(value),
        StorageError::ExecutionFailed(value) => CloneStorageError::ExecutionFailed(value),
        StorageError::InvalidOutput(value) => CloneStorageError::ExecutionFailed(value),
        StorageError::UnsafePath(value) => CloneStorageError::UnsafePath(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use turkuazvm_core::domain::disk::{DiskId, DiskImage};

    #[test]
    fn format_names_are_stable() {
        assert_eq!(QemuImgTool::format_name(DiskFormat::Qcow2), "qcow2");
        assert_eq!(QemuImgTool::format_name(DiskFormat::Raw), "raw");
    }

    #[test]
    fn relative_image_path_stays_under_machine_root() {
        let tool = QemuImgTool::new(
            PathBuf::from("qemu-img"),
            PathBuf::from("data"),
            PathBuf::from("data/machines"),
        );
        let vm_id = VmId::parse("vm-a").expect("vm id must be valid");
        let disk_id = DiskId::parse("system").expect("disk id must be valid");
        let image = DiskImage::create(
            disk_id,
            DiskFormat::Qcow2,
            1024,
            "disks/system.qcow2",
        )
        .expect("disk must be valid");
        let path = tool
            .resolve_image_target_path(&vm_id, &image)
            .expect("path must resolve");
        assert_eq!(path, PathBuf::from("data/machines/vm-a/disks/system.qcow2"));
    }

    #[test]
    fn existing_image_prefers_new_root_and_falls_back_to_legacy_root() {
        let root = std::env::temp_dir().join(format!(
            "turkuazvm-image-root-{}",
            std::process::id()
        ));
        let data_root = root.join("runtime-data");
        let image_root = root.join("VMs");
        let vm_id = VmId::parse("legacy-vm").expect("vm id must be valid");
        let disk_id = DiskId::parse("system").expect("disk id must be valid");
        let image = DiskImage::create(
            disk_id,
            DiskFormat::Qcow2,
            1024,
            "disks/system.qcow2",
        )
        .expect("disk must be valid");
        let tool = QemuImgTool::new(
            PathBuf::from("qemu-img"),
            data_root.clone(),
            image_root.clone(),
        );

        let legacy = data_root
            .join("machines")
            .join("legacy-vm")
            .join("disks/system.qcow2");
        fs::create_dir_all(legacy.parent().expect("legacy parent"))
            .expect("legacy directory must be created");
        fs::write(&legacy, b"legacy").expect("legacy file must be created");
        assert_eq!(
            tool.resolve_existing_image_path(&vm_id, &image)
                .expect("legacy image must resolve"),
            legacy
        );

        let primary = image_root.join("legacy-vm").join("disks/system.qcow2");
        fs::create_dir_all(primary.parent().expect("primary parent"))
            .expect("primary directory must be created");
        fs::write(&primary, b"primary").expect("primary file must be created");
        assert_eq!(
            tool.resolve_existing_image_path(&vm_id, &image)
                .expect("primary image must resolve"),
            primary
        );

        let _ = fs::remove_dir_all(root);
    }
}

