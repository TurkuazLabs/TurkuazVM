// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/domain/disk.rs
// # 📌 Amac: Sanal disk kimligi, format, bus ve attachment domain modellerini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Storage adapterlerinden bagimsiz, tasinabilir VM disk invariantlarini modeller
// # Bagimli Oldugu Katman: Service | Repo | Tool

use std::path::{Component, Path};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiskId(String);

impl DiskId {
    pub fn parse(value: impl Into<String>) -> Result<Self, DiskDomainError> {
        let value = value.into();
        let valid = !value.is_empty()
            && value.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
            });

        if !valid {
            return Err(DiskDomainError::InvalidId);
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiskFormat {
    Qcow2,
    Raw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiskBus {
    Virtio,
    Ide,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiskDomainError {
    InvalidId,
    InvalidVirtualSize,
    InvalidRelativePath,
    DiskAlreadyAttached(DiskId),
    DiskNotAttached(DiskId),
    ResizeMustGrow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskImage {
    id: DiskId,
    format: DiskFormat,
    virtual_size_bytes: u64,
    relative_path: String,
}

impl DiskImage {
    pub fn create(
        id: DiskId,
        format: DiskFormat,
        virtual_size_bytes: u64,
        relative_path: impl Into<String>,
    ) -> Result<Self, DiskDomainError> {
        if virtual_size_bytes == 0 {
            return Err(DiskDomainError::InvalidVirtualSize);
        }

        let relative_path = relative_path.into();
        if !Self::is_safe_relative_path(&relative_path) {
            return Err(DiskDomainError::InvalidRelativePath);
        }

        Ok(Self {
            id,
            format,
            virtual_size_bytes,
            relative_path,
        })
    }

    pub fn id(&self) -> &DiskId {
        &self.id
    }

    pub const fn format(&self) -> DiskFormat {
        self.format
    }

    pub const fn virtual_size_bytes(&self) -> u64 {
        self.virtual_size_bytes
    }

    pub fn relative_path(&self) -> &str {
        &self.relative_path
    }

    pub fn grow_to(&mut self, new_virtual_size_bytes: u64) -> Result<(), DiskDomainError> {
        if new_virtual_size_bytes <= self.virtual_size_bytes {
            return Err(DiskDomainError::ResizeMustGrow);
        }

        self.virtual_size_bytes = new_virtual_size_bytes;
        Ok(())
    }

    fn is_safe_relative_path(value: &str) -> bool {
        if value.trim().is_empty() {
            return false;
        }

        if value.starts_with('/') || value.starts_with('\\') || value.contains('\\') || value.contains(':') {
            return false;
        }

        let path = Path::new(value);
        if path.is_absolute() {
            return false;
        }

        path.components().all(|component| {
            matches!(component, Component::Normal(_) | Component::CurDir)
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskAttachment {
    image: DiskImage,
    bus: DiskBus,
    boot_index: Option<u8>,
}

impl DiskAttachment {
    pub const fn new(image: DiskImage, bus: DiskBus, boot_index: Option<u8>) -> Self {
        Self {
            image,
            bus,
            boot_index,
        }
    }

    pub const fn image(&self) -> &DiskImage {
        &self.image
    }

    pub fn image_mut(&mut self) -> &mut DiskImage {
        &mut self.image
    }

    pub const fn bus(&self) -> DiskBus {
        self.bus
    }

    pub const fn boot_index(&self) -> Option<u8> {
        self.boot_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_path_is_rejected() {
        let id = DiskId::parse("system").expect("disk id must be valid");
        let result = DiskImage::create(id, DiskFormat::Qcow2, 1024, "/tmp/system.qcow2");
        assert_eq!(result, Err(DiskDomainError::InvalidRelativePath));
    }

    #[test]
    fn parent_path_escape_is_rejected() {
        let id = DiskId::parse("system").expect("disk id must be valid");
        let result = DiskImage::create(id, DiskFormat::Qcow2, 1024, "../system.qcow2");
        assert_eq!(result, Err(DiskDomainError::InvalidRelativePath));
    }

    #[test]
    fn disk_can_only_grow() {
        let id = DiskId::parse("system").expect("disk id must be valid");
        let mut disk = DiskImage::create(id, DiskFormat::Qcow2, 1024, "disks/system.qcow2")
            .expect("disk must be valid");
        assert_eq!(disk.grow_to(512), Err(DiskDomainError::ResizeMustGrow));
        disk.grow_to(2048).expect("growth must succeed");
        assert_eq!(disk.virtual_size_bytes(), 2048);
    }
}
