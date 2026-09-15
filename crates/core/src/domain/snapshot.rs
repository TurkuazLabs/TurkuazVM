// # 📄 Dosya Yolu: /turkuazvm/crates/core/src/domain/snapshot.rs
// # 📌 Amac: VM snapshot kimligi ve metadata domain modellerini tanimlar
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: Storage implementasyonundan bagimsiz snapshot invariantlarini ve disk kapsamini modeller
// # Bagimli Oldugu Katman: Service | Repo | Tool

use crate::domain::disk::DiskId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SnapshotId(String);

impl SnapshotId {
    pub fn parse(value: impl Into<String>) -> Result<Self, SnapshotDomainError> {
        let value = value.into();
        let valid = !value.is_empty()
            && value.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
            });
        if !valid {
            return Err(SnapshotDomainError::InvalidId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotRecord {
    id: SnapshotId,
    name: String,
    created_at_unix_ms: u64,
    disk_ids: Vec<DiskId>,
}

impl SnapshotRecord {
    pub fn create(
        id: SnapshotId,
        name: impl Into<String>,
        created_at_unix_ms: u64,
        disk_ids: Vec<DiskId>,
    ) -> Result<Self, SnapshotDomainError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(SnapshotDomainError::InvalidName);
        }
        if disk_ids.is_empty() {
            return Err(SnapshotDomainError::EmptyDiskSet);
        }
        for (index, disk_id) in disk_ids.iter().enumerate() {
            if disk_ids[..index].contains(disk_id) {
                return Err(SnapshotDomainError::DuplicateDisk(disk_id.clone()));
            }
        }
        Ok(Self {
            id,
            name,
            created_at_unix_ms,
            disk_ids,
        })
    }

    pub fn id(&self) -> &SnapshotId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn created_at_unix_ms(&self) -> u64 {
        self.created_at_unix_ms
    }

    pub fn disk_ids(&self) -> &[DiskId] {
        &self.disk_ids
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotDomainError {
    InvalidId,
    InvalidName,
    EmptyDiskSet,
    DuplicateDisk(DiskId),
    SnapshotAlreadyExists(SnapshotId),
    SnapshotNotFound(SnapshotId),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_disk_is_rejected() {
        let id = SnapshotId::parse("before-update").expect("snapshot id must be valid");
        let disk = DiskId::parse("system").expect("disk id must be valid");
        let result = SnapshotRecord::create(id, "Before Update", 1, vec![disk.clone(), disk]);
        assert!(matches!(result, Err(SnapshotDomainError::DuplicateDisk(_))));
    }
}
