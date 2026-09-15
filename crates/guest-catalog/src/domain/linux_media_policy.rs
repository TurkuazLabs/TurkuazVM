// # 📄 Dosya Yolu: /turkuazvm/crates/guest-catalog/src/domain/linux_media_policy.rs
// # 📌 Amac: Linux kurulum profilleri icin tercih edilen installer medya turu sirasini typed domain modeli olarak tanimlar
// # 📌 Modul - Rust
// # Version: 0.39.4
// # Aciklama: Dagitim ve profil bazinda Network Install, Boot, Minimal, Server Standard, Live ve DVD onceliklerini tanimlar
// # Bagimli Oldugu Katman: Service | Repo

use crate::domain::guest_template::InstallerMediaKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinuxMediaPolicy {
    pub product_id: String,
    pub profile_id: String,
    pub preferred_media_kinds: Vec<InstallerMediaKind>,
}

impl LinuxMediaPolicy {
    pub fn new(
        product_id: String,
        profile_id: String,
        preferred_media_kinds: Vec<InstallerMediaKind>,
    ) -> Result<Self, LinuxMediaPolicyError> {
        if !valid_id(&product_id) || !valid_id(&profile_id) || preferred_media_kinds.is_empty() {
            return Err(LinuxMediaPolicyError::InvalidPolicy);
        }
        if preferred_media_kinds.contains(&InstallerMediaKind::Unknown) {
            return Err(LinuxMediaPolicyError::InvalidPolicy);
        }
        let mut unique = std::collections::HashSet::new();
        if preferred_media_kinds.iter().any(|kind| !unique.insert(*kind)) {
            return Err(LinuxMediaPolicyError::InvalidPolicy);
        }
        Ok(Self {
            product_id,
            profile_id,
            preferred_media_kinds,
        })
    }

    pub fn matches(&self, product_id: &str, profile_id: &str) -> bool {
        self.product_id == product_id && self.profile_id == profile_id
    }

    pub fn rank(&self, kind: InstallerMediaKind) -> Option<usize> {
        self.preferred_media_kinds.iter().position(|candidate| *candidate == kind)
    }
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 120
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinuxMediaPolicyError {
    InvalidPolicy,
}
