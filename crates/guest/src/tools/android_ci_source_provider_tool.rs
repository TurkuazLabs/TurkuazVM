// # 📄 Dosya Yolu: /turkuazvm/crates/guest/src/tools/android_ci_source_provider_tool.rs
// # 📌 Amac: Resmi Android CI branch/target/build ve same-build Cuttlefish artifactlerini runtime kesfeder
// # 📌 Modul - Rust
// # Version: 0.40.10
// # Aciklama: Android CI status parserinda kullanilan numeric build-id helperini production akisina geri getirir ve unresolved symbol compile hatasini kapatir
// # Bagimli Oldugu Katman: Tool | Service

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use turkuazvm_android_image::domain::distribution_source::{
    AndroidDistributionSource, AndroidDistributionSourceError, AndroidDistributionSourceRequest,
};
use turkuazvm_android_image::ports::android_distribution_source_provider_port::AndroidDistributionSourceProviderPort;

use crate::tools::http_download_tool::{HttpDownloadSettings, HttpDownloadTool, HttpRequestContext};

const PROVIDER_CODE: &str = "android_ci";
const BRANCH_STATUS_FILE: &str = "status.json";
const BUILD_INFO_FILE: &str = "BUILD_INFO";
const DEVICE_IMAGE_PRIMARY: &str = "aosp_cf_x86_64_phone-img";
const DEVICE_IMAGE_FALLBACK: &str = "aosp_cf_x86_64_only_phone-img";
const HOST_ARCHIVE_NAME: &str = "cvd-host_package.tar.gz";
const ARTIFACT_PROBE_RANGE: &str = "0-0";
const ARTIFACT_PROBE_MAX_FILESIZE_BYTES: u64 = 1;
const ACCEPT_BUILD_INFO: &str = "Accept: application/json,text/plain,*/*";
const ACCEPT_ARTIFACT: &str = "Accept: application/octet-stream,*/*";
const TARGET_ID_FIELDS: [&str; 4] = ["name", "target", "ID", "id"];
const BUILD_ID_FIELDS: [&str; 6] = [
    "last_known_good_build",
    "lastKnownGoodBuild",
    "last_good_build",
    "lastGoodBuild",
    "build_id",
    "buildId",
];
const TARGET_DISCOVERY_REQUIRED_TOKENS: [&str; 3] = ["cf_x86_64", "phone", "userdebug"];
const TARGET_DISCOVERY_EXCLUDED_TOKENS: [&str; 5] = ["arm64", "automotive", "tablet", "wear", "tv"];
const TARGET_DIAGNOSTIC_LIMIT: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
struct AndroidCiStatusTarget {
    target: String,
    build_id: String,
}

#[derive(Debug, Clone)]
pub struct AndroidCiSourceProviderSettings {
    pub curl_binary: PathBuf,
    pub connect_timeout_seconds: u64,
    pub retry_count: u32,
    pub retry_delay_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct AndroidCiSourceProviderTool {
    http: HttpDownloadTool,
}

impl AndroidCiSourceProviderTool {
    pub fn new(settings: AndroidCiSourceProviderSettings) -> Self {
        Self {
            http: HttpDownloadTool::new(HttpDownloadSettings {
                curl_binary: settings.curl_binary,
                connect_timeout_seconds: settings.connect_timeout_seconds,
                retry_count: settings.retry_count,
                retry_delay_seconds: settings.retry_delay_seconds,
            }),
        }
    }

    fn request_context(base_url: &str, accept: &str) -> HttpRequestContext {
        HttpRequestContext {
            user_agent: Some(browser_user_agent()),
            referer: Some(format!("{}/", base_url.trim_end_matches('/'))),
            headers: vec![accept.to_owned()],
        }
    }

    fn fetch_text(&self, base_url: &str, url: &str) -> Result<String, String> {
        self.http.fetch_text_with_context(
            url,
            &Self::request_context(base_url, ACCEPT_BUILD_INFO),
        )
    }

    fn artifact_exists(&self, base_url: &str, url: &str) -> bool {
        self.http
            .probe_range(
                url,
                ARTIFACT_PROBE_RANGE,
                ARTIFACT_PROBE_MAX_FILESIZE_BYTES,
                &Self::request_context(base_url, ACCEPT_ARTIFACT),
            )
            .is_ok_and(|code| matches!(code, 200 | 206))
    }

    fn resolve_known_build(
        &self,
        request: &AndroidDistributionSourceRequest,
        base_url: &str,
        branch: &str,
        target: &str,
        build_id: &str,
    ) -> Result<AndroidDistributionSource, String> {
        let artifact_base_url = format!("{base_url}/builds/submitted/{build_id}/{target}/latest");
        let build_info_url = format!("{artifact_base_url}/raw/{BUILD_INFO_FILE}");
        let build_info = self.fetch_text(base_url, &build_info_url)?;
        let actual_sdk = parse_build_info_string(&build_info, "build_version_sdk")
            .and_then(|value| value.parse::<u32>().ok());
        let actual_release = parse_build_info_string(&build_info, "build_version_release");

        if let (Some(expected), Some(actual)) = (request.expected_sdk, actual_sdk) {
            if expected != actual {
                return Err(format!("SDK uyusmazligi beklenen={expected} bulunan={actual}"));
            }
        } else if let (Some(expected_release), Some(actual_release)) =
            (request.release.as_deref(), actual_release.as_deref())
        {
            if !release_values_match(expected_release, actual_release) {
                return Err(format!(
                    "release uyusmazligi beklenen={expected_release} bulunan={actual_release}"
                ));
            }
        }

        let mut device_artifact_name = None;
        for prefix in [DEVICE_IMAGE_PRIMARY, DEVICE_IMAGE_FALLBACK] {
            let name = format!("{prefix}-{build_id}.zip");
            let url = format!("{artifact_base_url}/raw/{name}");
            if self.artifact_exists(base_url, &url) {
                device_artifact_name = Some(name);
                break;
            }
        }
        let device_artifact_name = device_artifact_name
            .ok_or_else(|| String::from("same-build Cuttlefish x86_64 device image artifact bulunamadi"))?;

        let host_url = format!("{artifact_base_url}/raw/{HOST_ARCHIVE_NAME}");
        let host_package_available = self.artifact_exists(base_url, &host_url);
        if !host_package_available && !request.allow_device_bootloader_fallback {
            return Err(String::from("same-build Cuttlefish host package artifact bulunamadi"));
        }

        let resolved_at_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or_default();

        Ok(AndroidDistributionSource {
            cache_key: request.cache_key(),
            provider: String::from(PROVIDER_CODE),
            base_url: base_url.to_owned(),
            branch: branch.to_owned(),
            target: target.to_owned(),
            build_id: build_id.to_owned(),
            artifact_base_url,
            device_artifact_name,
            host_artifact_name: String::from(HOST_ARCHIVE_NAME),
            host_package_available,
            allow_device_bootloader_fallback: request.allow_device_bootloader_fallback,
            android_release: actual_release.or_else(|| request.release.clone()),
            sdk_level: actual_sdk.or(request.expected_sdk),
            resolved_at_unix,
        })
    }
}

impl AndroidDistributionSourceProviderPort for AndroidCiSourceProviderTool {
    fn resolve(
        &self,
        request: &AndroidDistributionSourceRequest,
    ) -> Result<AndroidDistributionSource, AndroidDistributionSourceError> {
        let mut failures = Vec::new();
        for base_url in &request.base_urls {
            for branch in &request.branch_candidates {
                let status_url = format!("{base_url}/builds/branches/{branch}/{BRANCH_STATUS_FILE}");
                let status_json = match self.fetch_text(base_url, &status_url) {
                    Ok(content) => content,
                    Err(error) => {
                        failures.push(format!("{base_url} {branch}: status.json okunamadi: {error}"));
                        continue;
                    }
                };
                let candidates = match parse_status_target_candidates(
                    &status_json,
                    branch,
                    &request.target_candidates,
                ) {
                    Ok(candidates) => candidates,
                    Err(error) => {
                        failures.push(format!("{base_url} {branch}: {error}"));
                        continue;
                    }
                };
                if candidates.is_empty() {
                    let available = parse_status_target_names(&status_json, branch)
                        .map(|targets| diagnostic_target_list(&targets))
                        .unwrap_or_else(|_| String::from("unavailable"));
                    failures.push(format!(
                        "{base_url} {branch}: Cuttlefish x86_64 userdebug target bulunamadi; status_targets={available}"
                    ));
                    continue;
                }
                for candidate in candidates {
                    match self.resolve_known_build(
                        request,
                        base_url,
                        branch,
                        &candidate.target,
                        &candidate.build_id,
                    ) {
                        Ok(source) => return Ok(source),
                        Err(error) => failures.push(format!(
                            "{base_url} {branch}/{} build={}: {error}",
                            candidate.target, candidate.build_id
                        )),
                    }
                }
            }
        }
        Err(AndroidDistributionSourceError::Provider(format!(
            "Android CI source discovery basarisiz: {}",
            failures.join(" | ")
        )))
    }
}

fn browser_user_agent() -> String {
    format!(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 TurkuazVM/{} Safari/537.36",
        env!("CARGO_PKG_VERSION")
    )
}

fn parse_status_target_candidates(
    content: &str,
    branch: &str,
    preferred_targets: &[String],
) -> Result<Vec<AndroidCiStatusTarget>, String> {
    let records = parse_status_targets(content, branch)?;
    let mut ordered = Vec::new();

    for preferred in preferred_targets {
        for record in &records {
            if record.target == *preferred {
                push_unique_target(&mut ordered, record.clone());
            }
        }
    }

    let mut discovered: Vec<(usize, AndroidCiStatusTarget)> = records
        .into_iter()
        .filter_map(|record| target_discovery_rank(&record.target).map(|rank| (rank, record)))
        .collect();
    discovered.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| left.1.target.cmp(&right.1.target))
    });
    for (_, record) in discovered {
        push_unique_target(&mut ordered, record);
    }
    Ok(ordered)
}

fn parse_status_targets(content: &str, branch: &str) -> Result<Vec<AndroidCiStatusTarget>, String> {
    let value: serde_json::Value = serde_json::from_str(content)
        .map_err(|error| format!("Android CI status.json parse failed: {error}"))?;
    let mut records = Vec::new();
    if let Some(targets) = value.get("targets") {
        collect_status_targets(targets, branch, None, 0, &mut records);
    }
    if records.is_empty() {
        collect_status_targets(&value, branch, None, 0, &mut records);
    }
    Ok(records)
}

fn parse_status_target_names(content: &str, branch: &str) -> Result<Vec<String>, String> {
    let records = parse_status_targets(content, branch)?;
    let mut names = Vec::new();
    for record in records {
        if !names.contains(&record.target) {
            names.push(record.target);
        }
    }
    Ok(names)
}

fn collect_status_targets(
    value: &serde_json::Value,
    branch: &str,
    key_hint: Option<&str>,
    depth: usize,
    output: &mut Vec<AndroidCiStatusTarget>,
) {
    if depth > 8 {
        return;
    }
    match value {
        serde_json::Value::Object(object) => {
            let identifier = TARGET_ID_FIELDS
                .iter()
                .find_map(|key| object.get(*key).and_then(serde_json::Value::as_str))
                .or(key_hint);
            if let (Some(identifier), Some(build_id)) = (identifier, numeric_build_id(value)) {
                let target = normalize_target_id(branch, identifier);
                if !target.is_empty() {
                    push_unique_target(output, AndroidCiStatusTarget { target, build_id });
                }
            }
            for (key, child) in object {
                collect_status_targets(child, branch, Some(key.as_str()), depth + 1, output);
            }
        }
        serde_json::Value::Array(entries) => {
            for entry in entries {
                collect_status_targets(entry, branch, key_hint, depth + 1, output);
            }
        }
        _ => {}
    }
}

fn numeric_build_id(value: &serde_json::Value) -> Option<String> {
    let candidate = match value {
        serde_json::Value::Object(object) => BUILD_ID_FIELDS
            .iter()
            .find_map(|key| object.get(*key))
            .and_then(numeric_build_id),
        serde_json::Value::String(value) => Some(value.trim().to_owned()),
        serde_json::Value::Number(value) => Some(value.to_string()),
        _ => None,
    }?;
    (!candidate.is_empty() && candidate.bytes().all(|byte| byte.is_ascii_digit())).then_some(candidate)
}

fn normalize_target_id(branch: &str, raw: &str) -> String {
    let raw = raw.trim();
    let branch_prefix = format!("{branch}.");
    raw.strip_prefix(&branch_prefix).unwrap_or(raw).to_owned()
}

fn target_discovery_rank(target: &str) -> Option<usize> {
    let normalized = target.to_ascii_lowercase();
    if TARGET_DISCOVERY_REQUIRED_TOKENS
        .iter()
        .any(|token| !normalized.contains(token))
    {
        return None;
    }
    if TARGET_DISCOVERY_EXCLUDED_TOKENS
        .iter()
        .any(|token| normalized.contains(token))
    {
        return None;
    }
    if normalized.contains("only_phone-userdebug") {
        Some(0)
    } else if normalized.contains("phone-userdebug") {
        Some(1)
    } else if normalized.contains("only_phone") {
        Some(2)
    } else {
        Some(3)
    }
}

fn diagnostic_target_list(targets: &[String]) -> String {
    let mut selected = targets
        .iter()
        .filter(|target| {
            let normalized = target.to_ascii_lowercase();
            normalized.contains("cf_") || normalized.contains("x86_64") || normalized.contains("userdebug")
        })
        .take(TARGET_DIAGNOSTIC_LIMIT)
        .cloned()
        .collect::<Vec<_>>();
    if selected.is_empty() {
        selected = targets.iter().take(TARGET_DIAGNOSTIC_LIMIT).cloned().collect();
    }
    if selected.is_empty() {
        String::from("empty")
    } else {
        selected.join(",")
    }
}

fn push_unique_target(output: &mut Vec<AndroidCiStatusTarget>, candidate: AndroidCiStatusTarget) {
    if !output
        .iter()
        .any(|existing| existing.target == candidate.target && existing.build_id == candidate.build_id)
    {
        output.push(candidate);
    }
}

fn release_values_match(expected: &str, actual: &str) -> bool {
    if expected == actual {
        return true;
    }
    matches!((expected, actual), ("12L", "12") | ("12L", "12.1"))
}

fn parse_build_info_string(content: &str, key: &str) -> Option<String> {
    let aliases: &[&str] = match key {
        "build_version_sdk" => &["build_version_sdk", "ro.build.version.sdk", "ro.system.build.version.sdk"],
        "build_version_release" => &[
            "build_version_release",
            "ro.build.version.release",
            "ro.system.build.version.release",
        ],
        _ => &[key],
    };
    for alias in aliases {
        for line in content.lines() {
            let line = line.trim();
            if let Some(value) = line.strip_prefix(&format!("{alias}=")) {
                if let Some(value) = non_empty_unquoted(value) {
                    return Some(value);
                }
            }
            if line.starts_with(&format!("\"{alias}\"")) {
                if let Some((_, value)) = line.split_once(':') {
                    if let Some(value) = non_empty_unquoted(value.trim().trim_end_matches(',')) {
                        return Some(value);
                    }
                }
            }
        }
    }
    None
}

fn non_empty_unquoted(value: &str) -> Option<String> {
    let value = value.trim().trim_matches('"').trim_matches('\'').trim();
    (!value.is_empty() && value != "null").then(|| value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::{parse_build_info_string, parse_status_target_candidates, release_values_match};

    #[test]
    fn production_parser_handles_modern_and_legacy_status_shapes() {
        let modern = r#"{"targets":[{"name":"aosp_cf_x86_64_only_phone-userdebug","last_known_good_build":"123456"}]}"#;
        let modern_preferred = vec![String::from("aosp_cf_x86_64_only_phone-userdebug")];
        let modern_candidates =
            parse_status_target_candidates(modern, "aosp-android17-release", &modern_preferred).unwrap();
        assert_eq!(modern_candidates[0].target, "aosp_cf_x86_64_only_phone-userdebug");
        assert_eq!(modern_candidates[0].build_id, "123456");

        let legacy = r#"{"builds":{"aosp_cf_x86_64_phone-userdebug":{"build_id":987654}}}"#;
        let legacy_preferred = vec![String::from("aosp_cf_x86_64_phone-userdebug")];
        let legacy_candidates =
            parse_status_target_candidates(legacy, "aosp-android11-gsi", &legacy_preferred).unwrap();
        assert_eq!(legacy_candidates[0].target, "aosp_cf_x86_64_phone-userdebug");
        assert_eq!(legacy_candidates[0].build_id, "987654");
    }

    #[test]
    fn dynamically_discovers_historical_cuttlefish_target_names() {
        let content = r#"{"targets":[
            {"ID":"aosp-android12-gsi.aosp_cf_x86_64_phone_gki5_10-userdebug","last_known_good_build":"7654321"},
            {"name":"aosp_x86_64-userdebug","last_known_good_build":"7654000"}
        ]}"#;
        let preferred = vec![String::from("aosp_cf_x86_64_only_phone-userdebug")];
        let candidates = parse_status_target_candidates(content, "aosp-android12-gsi", &preferred).unwrap();
        assert_eq!(candidates[0].target, "aosp_cf_x86_64_phone_gki5_10-userdebug");
        assert_eq!(candidates[0].build_id, "7654321");
    }

    #[test]
    fn exact_configured_target_is_preferred_before_dynamic_fallback() {
        let content = r#"{"targets":[
            {"name":"aosp_cf_x86_64_phone_gki5_10-userdebug","last_known_good_build":"7000001"},
            {"name":"aosp_cf_x86_64_only_phone-userdebug","last_known_good_build":"7000002"}
        ]}"#;
        let preferred = vec![String::from("aosp_cf_x86_64_only_phone-userdebug")];
        let candidates = parse_status_target_candidates(content, "aosp-android13-gsi", &preferred).unwrap();
        assert_eq!(candidates[0].target, "aosp_cf_x86_64_only_phone-userdebug");
        assert_eq!(candidates[0].build_id, "7000002");
    }

    #[test]
    fn android_12l_release_alias_is_accepted_when_sdk_metadata_is_missing() {
        assert!(release_values_match("12L", "12"));
        assert!(release_values_match("12L", "12.1"));
        assert!(!release_values_match("12", "12.1"));
    }

    #[test]
    fn parses_legacy_build_info_aliases() {
        assert_eq!(parse_build_info_string("ro.build.version.sdk=30", "build_version_sdk").as_deref(), Some("30"));
        assert_eq!(parse_build_info_string("ro.build.version.release=11", "build_version_release").as_deref(), Some("11"));
    }
}
