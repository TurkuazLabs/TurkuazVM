// # 📄 Dosya Yolu: /turkuazvm/crates/guest-catalog/src/services/guest_catalog_service.rs
// # 📌 Amac: Guest Catalog template sorgu ve Linux installer medya secim is kurallarini uygular
// # 📌 Modul - Rust
// # Version: 0.39.4
// # Aciklama: Repository kayitlarini yukler; Linux Desktop/Server profillerinde politika siralamasina gore en uygun medyayi varsayilan yapar
// # Bagimli Oldugu Katman: Repo

use crate::domain::guest_template::{GuestFamily, GuestTemplate, InstallerMediaSource};
use crate::domain::linux_media_policy::LinuxMediaPolicy;
use crate::ports::guest_catalog_repository_port::{
    GuestCatalogRepositoryError, GuestCatalogRepositoryPort,
};

pub struct GuestCatalogService<R: GuestCatalogRepositoryPort> {
    repository: R,
}

impl<R: GuestCatalogRepositoryPort> GuestCatalogService<R> {
    pub const fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn list(&self) -> Result<Vec<GuestTemplate>, GuestCatalogRepositoryError> {
        let policies = self.repository.linux_media_policies()?;
        self.repository
            .list()?
            .into_iter()
            .map(|template| apply_linux_media_policy(template, &policies))
            .collect()
    }

    pub fn get(&self, id: &str) -> Result<GuestTemplate, GuestCatalogRepositoryError> {
        let policies = self.repository.linux_media_policies()?;
        apply_linux_media_policy(self.repository.get(id)?, &policies)
    }
}

fn apply_linux_media_policy(
    mut template: GuestTemplate,
    policies: &[LinuxMediaPolicy],
) -> Result<GuestTemplate, GuestCatalogRepositoryError> {
    if template.family != GuestFamily::Linux {
        return Ok(template);
    }
    let Some(policy) = policies
        .iter()
        .find(|policy| policy.matches(&template.product_id, &template.profile_id))
    else {
        return Ok(template);
    };

    let mut sources = Vec::new();
    if let Some(primary) = template.installer_media.take() {
        sources.push(primary);
    }
    sources.append(&mut template.installer_media_options);
    if sources.is_empty() {
        return Ok(template);
    }

    let selected_index = sources
        .iter()
        .enumerate()
        .filter_map(|(index, media)| policy.rank(media.media_kind).map(|rank| (rank, index)))
        .min_by_key(|(rank, index)| (*rank, *index))
        .map(|(_, index)| index)
        .unwrap_or(0);

    let mut selected = sources.remove(selected_index);
    selected.recommended = true;
    for media in &mut sources {
        media.recommended = false;
    }
    validate_policy_selection(policy, &selected)?;
    template.installer_media = Some(selected);
    template.installer_media_options = sources;
    Ok(template)
}

fn validate_policy_selection(
    policy: &LinuxMediaPolicy,
    selected: &InstallerMediaSource,
) -> Result<(), GuestCatalogRepositoryError> {
    if policy.rank(selected.media_kind).is_none() {
        return Err(GuestCatalogRepositoryError::Storage(format!(
            "Linux media policy could not select a supported media kind for {}/{}",
            policy.product_id, policy.profile_id
        )));
    }
    Ok(())
}
