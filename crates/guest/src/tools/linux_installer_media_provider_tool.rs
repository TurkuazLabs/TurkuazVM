// # 📄 Dosya Yolu: /turkuazvm/crates/guest/src/tools/linux_installer_media_provider_tool.rs
// # 📌 Amac: Resmi Linux dagitim repository/index sayfalarindan en uygun installer ISO ve checksum kaynaklarini dinamik kesfeder
// # 📌 Modul - Rust
// # Version: 0.40.0
// # Aciklama: Fedora, Debian, Ubuntu, Rocky ve Linux Mint icin config-driven index/official-page discovery uygular; mirror ve checksum adaylarini uretir
// # Bagimli Oldugu Katman: Tool | Service

use std::cmp::Ordering;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use turkuazvm_guest_catalog::domain::installer_media_source::{
    InstallerMediaChecksumStrategy, InstallerMediaDiscoveryMode, InstallerMediaProviderPolicy,
    InstallerMediaSourceError, InstallerMediaSourceOrigin, InstallerMediaSourceRequest,
    ResolvedInstallerMediaSource,
};
use turkuazvm_guest_catalog::ports::installer_media_source_provider_port::InstallerMediaSourceProviderPort;

use crate::tools::http_download_tool::HttpDownloadTool;

pub struct LinuxInstallerMediaProviderTool {
    http: Arc<HttpDownloadTool>,
}

impl LinuxInstallerMediaProviderTool {
    pub const fn new(http: Arc<HttpDownloadTool>) -> Self {
        Self { http }
    }

    fn resolve_official_page_mirrors(
        &self,
        request: &InstallerMediaSourceRequest,
        rule: &turkuazvm_guest_catalog::domain::installer_media_source::InstallerMediaProviderRule,
    ) -> Result<ResolvedInstallerMediaSource, InstallerMediaSourceError> {
        let page_url = request.catalog_source.url.trim();
        let page = self.http.fetch_text(page_url).map_err(|error| {
            InstallerMediaSourceError::Provider(format!(
                "{} resmi download sayfasi okunamadi: {error}",
                request.provider
            ))
        })?;
        let links = extract_href_values(&page);
        let release_line = release_line(request);
        let filename_tokens = expand_tokens(&rule.filename_tokens, request, &release_line);
        let filename_excludes = expand_tokens(&rule.filename_excludes, request, &release_line);

        let mut download_urls = Vec::new();
        let mut selected_filename = None;
        for link in &links {
            if !(link.starts_with("https://") || link.starts_with("http://")) {
                continue;
            }
            let Some(filename) = url_filename(link) else { continue; };
            if !filename.to_ascii_lowercase().ends_with(".iso")
                || !filename_tokens.iter().all(|token| filename.contains(token))
                || !filename_excludes.iter().all(|token| !filename.contains(token))
            {
                continue;
            }
            if let Some(existing) = selected_filename.as_deref() {
                if existing != filename {
                    continue;
                }
            } else {
                selected_filename = Some(filename.to_owned());
            }
            push_unique(&mut download_urls, link.clone());
        }

        let filename = selected_filename.ok_or_else(|| {
            InstallerMediaSourceError::Provider(format!(
                "{} resmi download sayfasinda uygun ISO bulunamadi",
                request.provider
            ))
        })?;
        if download_urls.is_empty() {
            return Err(InstallerMediaSourceError::Provider(format!(
                "{} resmi download sayfasinda mirror URL bulunamadi",
                request.provider
            )));
        }

        let checksum_value = expand_token(&rule.checksum_value, request, &release_line);
        let checksum_tokens = expand_tokens(&rule.checksum_tokens, request, &release_line);
        let mut checksum_urls = Vec::new();
        for link in &links {
            if !(link.starts_with("https://") || link.starts_with("http://")) {
                continue;
            }
            let Some(name) = url_filename(link) else { continue; };
            let matches = match rule.checksum_strategy {
                InstallerMediaChecksumStrategy::FixedName => name == checksum_value,
                InstallerMediaChecksumStrategy::FileSuffix => name == format!("{filename}{checksum_value}"),
                InstallerMediaChecksumStrategy::Discover => {
                    checksum_tokens.iter().all(|token| name.contains(token))
                        && name.to_ascii_lowercase().contains("checksum")
                }
            };
            if matches {
                push_unique(&mut checksum_urls, link.clone());
            }
        }
        if checksum_urls.is_empty() {
            return Err(InstallerMediaSourceError::Provider(format!(
                "{} resmi download sayfasinda checksum URL bulunamadi",
                request.provider
            )));
        }

        Ok(ResolvedInstallerMediaSource {
            cache_key: request.cache_key(),
            provider: request.provider.clone(),
            media_kind: request.media_kind,
            architecture: request.architecture.clone(),
            filename,
            download_urls,
            checksum_urls,
            size_bytes: request.catalog_source.size_bytes,
            origin: InstallerMediaSourceOrigin::OnlineDiscovery,
            resolved_at_unix: now_unix(),
        })
    }
}

impl InstallerMediaSourceProviderPort for LinuxInstallerMediaProviderTool {
    fn resolve(
        &self,
        request: &InstallerMediaSourceRequest,
        policy: &InstallerMediaProviderPolicy,
    ) -> Result<ResolvedInstallerMediaSource, InstallerMediaSourceError> {
        let rule = policy.rules.get(request.media_kind.code()).ok_or_else(|| {
            InstallerMediaSourceError::Policy(format!(
                "{}/{} icin source discovery rule tanimli degil",
                request.provider,
                request.media_kind.code()
            ))
        })?;

        if policy.discovery_mode == InstallerMediaDiscoveryMode::OfficialPageMirrors {
            return self.resolve_official_page_mirrors(request, rule);
        }

        let index_urls = candidate_index_urls(request, policy, &rule.index_path_templates);
        if index_urls.is_empty() {
            return Err(InstallerMediaSourceError::Policy(format!(
                "{} icin installer media index adayi yok",
                request.provider
            )));
        }

        let mut errors = Vec::new();
        for index_url in &index_urls {
            let index = match self.http.fetch_text(index_url) {
                Ok(content) => content,
                Err(error) => {
                    errors.push(format!("{index_url}: {error}"));
                    continue;
                }
            };
            let links = extract_links(&index);
            let release_line = release_line(request);
            let filename_tokens = expand_tokens(&rule.filename_tokens, request, &release_line);
            let filename_excludes = expand_tokens(&rule.filename_excludes, request, &release_line);
            let checksum_tokens = expand_tokens(&rule.checksum_tokens, request, &release_line);
            let checksum_value = expand_token(&rule.checksum_value, request, &release_line);
            let Some(filename) = select_iso_filename(&links, &filename_tokens, &filename_excludes, &request.provider) else {
                errors.push(format!("{index_url}: uygun ISO bulunamadi"));
                continue;
            };
            let checksum_filename = checksum_filename(
                &links,
                &filename,
                rule.checksum_strategy,
                &checksum_value,
                &checksum_tokens,
            );

            let download_urls = mirror_urls_for_filename(&index_urls, index_url, &filename);
            let checksum_urls = checksum_filename
                .as_deref()
                .map(|name| mirror_urls_for_filename(&index_urls, index_url, name))
                .unwrap_or_default();
            if download_urls.is_empty() {
                continue;
            }
            return Ok(ResolvedInstallerMediaSource {
                cache_key: request.cache_key(),
                provider: request.provider.clone(),
                media_kind: request.media_kind,
                architecture: request.architecture.clone(),
                filename,
                download_urls,
                checksum_urls,
                size_bytes: None,
                origin: InstallerMediaSourceOrigin::OnlineDiscovery,
                resolved_at_unix: now_unix(),
            });
        }

        Err(InstallerMediaSourceError::Provider(format!(
            "{} resmi installer media discovery basarisiz: {}",
            request.provider,
            errors.join(" | ")
        )))
    }
}

fn candidate_index_urls(
    request: &InstallerMediaSourceRequest,
    policy: &InstallerMediaProviderPolicy,
    path_templates: &[String],
) -> Vec<String> {
    let mut urls = Vec::new();
    let release_line = release_line(request);
    for base in &policy.base_urls {
        for template in path_templates {
            let path = template
                .replace("{release}", request.release_id.trim())
                .replace("{release_line}", &release_line)
                .replace("{architecture}", request.architecture.trim());
            let url = join_url(base, &path);
            push_unique(&mut urls, url);
        }
    }
    urls
}

fn expand_tokens(values: &[String], request: &InstallerMediaSourceRequest, release_line: &str) -> Vec<String> {
    values
        .iter()
        .map(|value| expand_token(value, request, release_line))
        .collect()
}

fn expand_token(value: &str, request: &InstallerMediaSourceRequest, release_line: &str) -> String {
    value
        .replace("{release}", request.release_id.trim())
        .replace("{release_line}", release_line)
        .replace("{architecture}", request.architecture.trim())
}

fn release_line(request: &InstallerMediaSourceRequest) -> String {
    if let Some(filename) = request.catalog_source.filename.as_deref() {
        let groups = raw_digit_groups(filename);
        if request.provider == "ubuntu" && groups.len() >= 2 {
            return format!("{}.{}", groups[0], pad_two(&groups[1]));
        }
    }
    let release = request.release_id.trim();
    let numeric = release.split('-').next().unwrap_or(release);
    if request.provider == "ubuntu" {
        let mut parts = numeric.split('.');
        if let (Some(major), Some(minor)) = (parts.next(), parts.next()) {
            return format!("{major}.{minor}");
        }
    }
    numeric.to_owned()
}

fn pad_two(value: &str) -> String {
    if value.len() == 1 { format!("0{value}") } else { value.to_owned() }
}

fn join_url(base: &str, path: &str) -> String {
    format!("{}/{}", base.trim_end_matches('/'), path.trim_start_matches('/'))
}

fn mirror_urls_for_filename(index_urls: &[String], preferred_index: &str, filename: &str) -> Vec<String> {
    let mut urls = Vec::new();
    push_unique(&mut urls, join_url(preferred_index, filename));
    for index in index_urls {
        push_unique(&mut urls, join_url(index, filename));
    }
    urls
}

fn extract_href_values(content: &str) -> Vec<String> {
    let mut links = Vec::new();
    for marker in ["href=\"", "href='"] {
        let quote = if marker.ends_with('\"') { '\"' } else { '\'' };
        let mut rest = content;
        while let Some(start) = rest.find(marker) {
            let value_start = start + marker.len();
            let tail = &rest[value_start..];
            let Some(end) = tail.find(quote) else { break; };
            let value = html_unescape(&tail[..end]);
            if !value.is_empty() {
                push_unique(&mut links, value);
            }
            rest = &tail[end + quote.len_utf8()..];
        }
    }
    links
}

fn url_filename(url: &str) -> Option<&str> {
    let without_query = url.split(['?', '#']).next()?;
    let filename = without_query.rsplit('/').next()?;
    (!filename.is_empty()).then_some(filename)
}

fn extract_links(content: &str) -> Vec<String> {
    let mut links = Vec::new();
    for marker in ["href=\"", "href='"] {
        let quote = if marker.ends_with('"') { '"' } else { '\'' };
        let mut rest = content;
        while let Some(start) = rest.find(marker) {
            let value_start = start + marker.len();
            let tail = &rest[value_start..];
            let Some(end) = tail.find(quote) else { break; };
            let value = &tail[..end];
            if !value.is_empty() && !value.contains('/') && !value.starts_with('?') {
                push_unique(&mut links, html_unescape(value));
            }
            rest = &tail[end + quote.len_utf8()..];
        }
    }
    links
}

fn html_unescape(value: &str) -> String {
    value.replace("&amp;", "&")
}

fn select_iso_filename(
    links: &[String],
    required_tokens: &[String],
    excluded_tokens: &[String],
    provider: &str,
) -> Option<String> {
    let mut candidates: Vec<&String> = links
        .iter()
        .filter(|name| name.to_ascii_lowercase().ends_with(".iso"))
        .filter(|name| required_tokens.iter().all(|token| name.contains(token)))
        .filter(|name| excluded_tokens.iter().all(|token| !name.contains(token)))
        .filter(|name| provider != "fedora" || stable_fedora_filename(name))
        .collect();
    candidates.sort_by(|left, right| compare_versionish(left, right));
    candidates.last().map(|value| (*value).clone())
}

fn stable_fedora_filename(filename: &str) -> bool {
    let lower = filename.to_ascii_lowercase();
    !lower.contains("nightly")
        && !lower.contains("rawhide")
        && !lower.contains(".n.")
        && !lower.contains("-n-")
}

fn compare_versionish(left: &str, right: &str) -> Ordering {
    let left_key = digit_groups(left);
    let right_key = digit_groups(right);
    left_key.cmp(&right_key).then_with(|| left.cmp(right))
}

fn digit_groups(value: &str) -> Vec<String> {
    raw_digit_groups(value).into_iter().map(|value| normalize_digits(&value)).collect()
}

fn raw_digit_groups(value: &str) -> Vec<String> {
    let mut groups = Vec::new();
    let mut current = String::new();
    for ch in value.chars() {
        if ch.is_ascii_digit() {
            current.push(ch);
        } else if !current.is_empty() {
            groups.push(current.clone());
            current.clear();
        }
    }
    if !current.is_empty() {
        groups.push(current);
    }
    groups
}

fn normalize_digits(value: &str) -> String {
    let trimmed = value.trim_start_matches('0');
    let normalized = if trimmed.is_empty() { "0" } else { trimmed };
    format!("{:0>12}", normalized)
}

fn checksum_filename(
    links: &[String],
    iso_filename: &str,
    strategy: InstallerMediaChecksumStrategy,
    value: &str,
    tokens: &[String],
) -> Option<String> {
    match strategy {
        InstallerMediaChecksumStrategy::FixedName => Some(value.to_owned()),
        InstallerMediaChecksumStrategy::FileSuffix => Some(format!("{iso_filename}{value}")),
        InstallerMediaChecksumStrategy::Discover => {
            let mut candidates: Vec<&String> = links
                .iter()
                .filter(|name| tokens.iter().all(|token| name.contains(token)))
                .filter(|name| name.to_ascii_uppercase().contains("CHECKSUM"))
                .filter(|name| !name.to_ascii_lowercase().ends_with(".asc"))
                .collect();
            candidates.sort_by(|left, right| compare_versionish(left, right));
            candidates.last().map(|value| (*value).clone())
        }
    }
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::{extract_href_values, extract_links, select_iso_filename, url_filename};

    #[test]
    fn fedora_nightly_candidate_is_rejected() {
        let links = vec![
            String::from("Fedora-Server-netinst-x86_64-44-20260424.n.0.iso"),
            String::from("Fedora-Server-netinst-x86_64-44-1.7.iso"),
        ];
        let selected = select_iso_filename(
            &links,
            &[String::from("Fedora-Server-netinst-x86_64-44-")],
            &[],
            "fedora",
        );
        assert_eq!(selected.as_deref(), Some("Fedora-Server-netinst-x86_64-44-1.7.iso"));
    }

    #[test]
    fn directory_links_are_parsed_without_nested_paths() {
        let html = r#"<a href="file.iso">file</a><a href='SHA256SUMS'>sum</a><a href="sub/">sub</a>"#;
        assert_eq!(extract_links(html), vec![String::from("file.iso"), String::from("SHA256SUMS")]);
    }
    #[test]
    fn official_page_direct_mirror_links_are_preserved() {
        let html = r#"<a href="https://pub.linuxmint.io/stable/22.3/linuxmint-22.3-cinnamon-64bit.iso">ISO</a><a href="https://mirrors.kernel.org/linuxmint/stable/22.3/sha256sum.txt">SUM</a>"#;
        let links = extract_href_values(html);
        assert_eq!(links.len(), 2);
        assert_eq!(url_filename(&links[0]), Some("linuxmint-22.3-cinnamon-64bit.iso"));
        assert_eq!(url_filename(&links[1]), Some("sha256sum.txt"));
    }

}
