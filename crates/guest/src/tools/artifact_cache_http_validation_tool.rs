// # 📄 Dosya Yolu: /turkuazvm/crates/guest/src/tools/artifact_cache_http_validation_tool.rs
// # 📌 Amac: Mutable HTTP artifact kaynaklarini ETag ve Last-Modified ile conditional olarak revalidate eder
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: URL/redirect policy, conditional HEAD, transient hata siniflandirmasi ve 304/2xx sonucunu typed contracta donusturur
// # Bagimli Oldugu Katman: Tool

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use turkuazvm_artifact_cache::domain::artifact_cache::ArtifactSourceValidators;
use turkuazvm_artifact_cache::ports::artifact_source_validation_port::{
    ArtifactSourceValidationError, ArtifactSourceValidationOutcome, ArtifactSourceValidationPort,
};

use crate::tools::artifact_cache_http_policy_tool::{
    curl_protocol_argument, validate_artifact_url, ArtifactHttpPolicy, ArtifactHttpPolicyError,
};

const HEADER_ETAG: &str = "etag";
const HEADER_LAST_MODIFIED: &str = "last-modified";
const HEADER_IF_NONE_MATCH: &str = "If-None-Match";
const HEADER_IF_MODIFIED_SINCE: &str = "If-Modified-Since";
const HTTP_CODE_MARKER: &str = "TVM_HTTP_CODE:";
const EFFECTIVE_URL_MARKER: &str = "TVM_EFFECTIVE_URL:";
const REDIRECT_URL_MARKER: &str = "TVM_REDIRECT_URL:";
const HTTP_NOT_MODIFIED: u16 = 304;

#[derive(Debug, Clone)]
pub struct ArtifactCacheHttpValidationSettings {
    pub curl_binary: PathBuf,
    pub connect_timeout_seconds: u64,
    pub request_timeout: Duration,
    pub policy: ArtifactHttpPolicy,
}

#[derive(Debug, Clone)]
pub struct CurlArtifactSourceValidationTool {
    settings: ArtifactCacheHttpValidationSettings,
}

impl CurlArtifactSourceValidationTool {
    pub const fn new(settings: ArtifactCacheHttpValidationSettings) -> Self {
        Self { settings }
    }

    fn execute(
        &self,
        source_url: &str,
        validators: &ArtifactSourceValidators,
    ) -> Result<String, ArtifactSourceValidationError> {
        let mut current_url = source_url.to_owned();
        for redirect_index in 0..=self.settings.policy.max_redirects {
            validate_artifact_url(&current_url, self.settings.policy).map_err(map_policy_error)?;
            let output = self.execute_once(&current_url, validators)?;
            let status = parse_http_code(&output)?;
            if (300..400).contains(&status) {
                if redirect_index >= self.settings.policy.max_redirects {
                    return Err(ArtifactSourceValidationError::Policy(String::from(
                        "artifact source redirect limit exceeded",
                    )));
                }
                let redirect_url = parse_redirect_url(&output)?;
                validate_artifact_url(&redirect_url, self.settings.policy).map_err(map_policy_error)?;
                current_url = redirect_url;
                continue;
            }
            let effective_url = parse_effective_url(&output)?;
            validate_artifact_url(effective_url, self.settings.policy).map_err(map_policy_error)?;
            return Ok(output);
        }
        Err(ArtifactSourceValidationError::Policy(String::from(
            "artifact source redirect limit exceeded",
        )))
    }

    fn execute_once(
        &self,
        source_url: &str,
        validators: &ArtifactSourceValidators,
    ) -> Result<String, ArtifactSourceValidationError> {
        let protocols = curl_protocol_argument(self.settings.policy);
        let mut command = Command::new(&self.settings.curl_binary);
        command
            .args(["--silent", "--show-error", "--head", "--proto"])
            .arg(protocols)
            .arg("--proto-redir")
            .arg(protocols)
            .arg("--connect-timeout")
            .arg(self.settings.connect_timeout_seconds.to_string())
            .arg("--max-time")
            .arg(self.settings.request_timeout.as_secs().max(1).to_string());

        if let Some(etag) = validators.etag.as_deref() {
            validate_header_value(etag)?;
            command.arg("-H").arg(format!("{HEADER_IF_NONE_MATCH}: {etag}"));
        }
        if let Some(last_modified) = validators.last_modified.as_deref() {
            validate_header_value(last_modified)?;
            command.arg("-H").arg(format!("{HEADER_IF_MODIFIED_SINCE}: {last_modified}"));
        }
        command
            .arg("--write-out")
            .arg(format!("\n{HTTP_CODE_MARKER}%{{http_code}}\n{EFFECTIVE_URL_MARKER}%{{url_effective}}\n{REDIRECT_URL_MARKER}%{{redirect_url}}\n"))
            .arg(source_url)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = command
            .spawn()
            .map_err(|error| ArtifactSourceValidationError::Transient(error.to_string()))?;
        let started = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(_)) => {
                    let output = child.wait_with_output()
                        .map_err(|error| ArtifactSourceValidationError::Transient(error.to_string()))?;
                    if !output.status.success() {
                        let detail = String::from_utf8_lossy(&output.stderr).trim().to_owned();
                        return Err(classify_curl_failure(output.status.code(), detail));
                    }
                    return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
                }
                Ok(None) if started.elapsed() < self.settings.request_timeout => {
                    thread::sleep(Duration::from_millis(25));
                }
                Ok(None) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(ArtifactSourceValidationError::Transient(String::from(
                        "artifact source validation timed out",
                    )));
                }
                Err(error) => return Err(ArtifactSourceValidationError::Transient(error.to_string())),
            }
        }
    }

}

impl ArtifactSourceValidationPort for CurlArtifactSourceValidationTool {
    fn revalidate(
        &mut self,
        source_url: &str,
        validators: &ArtifactSourceValidators,
    ) -> Result<ArtifactSourceValidationOutcome, ArtifactSourceValidationError> {
        let output = self.execute(source_url, validators)?;
        let status = parse_http_code(&output)?;
        let response_validators = parse_final_validators(&output);
        if status == HTTP_NOT_MODIFIED {
            return Ok(ArtifactSourceValidationOutcome::NotModified {
                validators: response_validators,
            });
        }
        if (200..300).contains(&status) {
            return Ok(ArtifactSourceValidationOutcome::Modified {
                validators: response_validators,
            });
        }
        Err(ArtifactSourceValidationError::InvalidResponse(format!(
            "artifact source validation returned HTTP {status}",
        )))
    }
}

fn map_policy_error(error: ArtifactHttpPolicyError) -> ArtifactSourceValidationError {
    match error {
        ArtifactHttpPolicyError::ResolutionFailed(message) => ArtifactSourceValidationError::Transient(message),
        ArtifactHttpPolicyError::InvalidUrl(message) | ArtifactHttpPolicyError::ForbiddenTarget(message) => {
            ArtifactSourceValidationError::Policy(message)
        }
    }
}

fn classify_curl_failure(code: Option<i32>, detail: String) -> ArtifactSourceValidationError {
    match code {
        Some(6 | 7 | 28) => ArtifactSourceValidationError::Transient(detail),
        _ => ArtifactSourceValidationError::InvalidResponse(detail),
    }
}

fn validate_header_value(value: &str) -> Result<(), ArtifactSourceValidationError> {
    if value.contains('\r') || value.contains('\n') {
        return Err(ArtifactSourceValidationError::InvalidResponse(String::from(
            "artifact validator contains newline characters",
        )));
    }
    Ok(())
}

fn parse_http_code(output: &str) -> Result<u16, ArtifactSourceValidationError> {
    output
        .lines()
        .rev()
        .find_map(|line| line.trim().strip_prefix(HTTP_CODE_MARKER))
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or_else(|| ArtifactSourceValidationError::InvalidResponse(String::from(
            "artifact source validation HTTP status is missing",
        )))
}

fn parse_effective_url(output: &str) -> Result<&str, ArtifactSourceValidationError> {
    output
        .lines()
        .rev()
        .find_map(|line| line.trim().strip_prefix(EFFECTIVE_URL_MARKER))
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ArtifactSourceValidationError::InvalidResponse(String::from(
            "artifact source effective URL is missing",
        )))
}

fn parse_redirect_url(output: &str) -> Result<String, ArtifactSourceValidationError> {
    output
        .lines()
        .rev()
        .find_map(|line| line.trim().strip_prefix(REDIRECT_URL_MARKER))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| ArtifactSourceValidationError::InvalidResponse(String::from(
            "artifact redirect URL is missing",
        )))
}

pub(crate) fn parse_final_validators(output: &str) -> ArtifactSourceValidators {
    let mut current = ArtifactSourceValidators::default();
    let mut final_headers = ArtifactSourceValidators::default();
    let mut inside_headers = false;
    for raw_line in output.lines() {
        let line = raw_line.trim_end_matches('\r');
        if line.starts_with("HTTP/") {
            current = ArtifactSourceValidators::default();
            inside_headers = true;
            continue;
        }
        if inside_headers && line.is_empty() {
            final_headers = current.clone();
            inside_headers = false;
            continue;
        }
        if !inside_headers {
            continue;
        }
        let Some((name, value)) = line.split_once(':') else { continue; };
        let value = value.trim();
        if name.eq_ignore_ascii_case(HEADER_ETAG) && !value.is_empty() {
            current.etag = Some(value.to_owned());
        } else if name.eq_ignore_ascii_case(HEADER_LAST_MODIFIED) && !value.is_empty() {
            current.last_modified = Some(value.to_owned());
        }
    }
    if inside_headers {
        final_headers = current;
    }
    final_headers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_final_redirect_response_validators() {
        let output = "HTTP/1.1 302 Found\r\nETag: old\r\n\r\nHTTP/2 304\r\nETag: new\r\nLast-Modified: Tue, 25 Aug 2026 10:00:00 GMT\r\n\r\nTVM_HTTP_CODE:304\nTVM_EFFECTIVE_URL:https://example.com/file\nTVM_REDIRECT_URL:\n";
        assert_eq!(parse_http_code(output).expect("status"), 304);
        let validators = parse_final_validators(output);
        assert_eq!(validators.etag.as_deref(), Some("new"));
        assert_eq!(validators.last_modified.as_deref(), Some("Tue, 25 Aug 2026 10:00:00 GMT"));
    }

    #[test]
    fn classifies_only_network_failures_as_transient() {
        assert!(matches!(classify_curl_failure(Some(6), String::new()), ArtifactSourceValidationError::Transient(_)));
        assert!(matches!(classify_curl_failure(Some(60), String::new()), ArtifactSourceValidationError::InvalidResponse(_)));
    }
}
