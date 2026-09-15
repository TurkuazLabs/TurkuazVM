// # 📄 Dosya Yolu: /turkuazvm/crates/guest/src/tools/artifact_cache_http_fetch_tool.rs
// # 📌 Amac: Mutable HTTP artifactini payload ve validator metadata'si ayni GET cevabindan gelecek sekilde indirir
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: HTTPS/private-network policy, redirect protocol siniri, retry ve final ETag/Last-Modified capture uygular
// # Bagimli Oldugu Katman: Tool

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use turkuazvm_artifact_cache::domain::artifact_cache::ArtifactSourceValidators;

use crate::tools::artifact_cache_http_policy_tool::{
    curl_protocol_argument, validate_artifact_url, ArtifactHttpPolicy, ValidatedArtifactUrl,
};
use crate::tools::artifact_cache_http_validation_tool::parse_final_validators;

const HTTP_CODE_MARKER: &str = "TVM_HTTP_CODE:";
const EFFECTIVE_URL_MARKER: &str = "TVM_EFFECTIVE_URL:";
const REDIRECT_URL_MARKER: &str = "TVM_REDIRECT_URL:";

#[derive(Debug, Clone)]
pub struct ArtifactCacheHttpFetchSettings {
    pub curl_binary: PathBuf,
    pub connect_timeout_seconds: u64,
    pub request_timeout: Duration,
    pub retry_count: u32,
    pub retry_delay_seconds: u64,
    pub policy: ArtifactHttpPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactCacheHttpFetchResult {
    pub effective_url: String,
    pub validators: ArtifactSourceValidators,
}

#[derive(Debug, Clone)]
pub struct ArtifactCacheHttpFetchTool {
    settings: ArtifactCacheHttpFetchSettings,
}

impl ArtifactCacheHttpFetchTool {
    pub const fn new(settings: ArtifactCacheHttpFetchSettings) -> Self {
        Self { settings }
    }

    pub fn fetch(&self, source_url: &str, destination: &Path) -> Result<ArtifactCacheHttpFetchResult, String> {
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let mut current_url = source_url.to_owned();
        for redirect_index in 0..=self.settings.policy.max_redirects {
            let validated_url = validate_artifact_url(&current_url, self.settings.policy)
                .map_err(|error| format!("artifact URL policy rejected source: {error:?}"))?;
            let header_path = destination.with_extension(format!("headers.{}.{}", std::process::id(), redirect_index));
            let output = match self.fetch_once(&current_url, &validated_url, destination, &header_path) {
                Ok(output) => output,
                Err(error) => {
                    let _ = fs::remove_file(destination);
                    let _ = fs::remove_file(&header_path);
                    return Err(error);
                }
            };
            let stdout = String::from_utf8_lossy(&output.stdout);
            let status = parse_marker_u16(&stdout, HTTP_CODE_MARKER)?;
            if (300..400).contains(&status) {
                let _ = fs::remove_file(destination);
                let _ = fs::remove_file(&header_path);
                if redirect_index >= self.settings.policy.max_redirects {
                    return Err(String::from("artifact redirect limit exceeded"));
                }
                let redirect_url = parse_marker_string(&stdout, REDIRECT_URL_MARKER)?;
                validate_artifact_url(&redirect_url, self.settings.policy)
                    .map_err(|error| format!("artifact redirect target rejected: {error:?}"))?;
                current_url = redirect_url;
                continue;
            }
            if !(200..300).contains(&status) {
                let _ = fs::remove_file(destination);
                let _ = fs::remove_file(&header_path);
                return Err(format!("artifact GET returned HTTP {status}"));
            }
            let effective_url = parse_marker_string(&stdout, EFFECTIVE_URL_MARKER)?;
            validate_artifact_url(&effective_url, self.settings.policy)
                .map_err(|error| format!("artifact effective URL rejected: {error:?}"))?;
            if !destination.is_file() {
                let _ = fs::remove_file(&header_path);
                return Err(String::from("artifact GET produced no payload"));
            }
            let headers = fs::read_to_string(&header_path).map_err(|error| error.to_string())?;
            let _ = fs::remove_file(&header_path);
            return Ok(ArtifactCacheHttpFetchResult {
                effective_url,
                validators: parse_final_validators(&headers),
            });
        }
        Err(String::from("artifact redirect limit exceeded"))
    }

    fn fetch_once(
        &self,
        source_url: &str,
        validated_url: &ValidatedArtifactUrl,
        destination: &Path,
        header_path: &Path,
    ) -> Result<std::process::Output, String> {
        let protocols = curl_protocol_argument(self.settings.policy);
        let mut command = Command::new(&self.settings.curl_binary);
        command
            .args(["--fail", "--silent", "--show-error", "--proto"])
            .arg(protocols)
            .arg("--proto-redir")
            .arg(protocols)
            .arg("--connect-timeout")
            .arg(self.settings.connect_timeout_seconds.to_string())
            .arg("--max-time")
            .arg(self.settings.request_timeout.as_secs().max(1).to_string())
            .arg("--retry")
            .arg(self.settings.retry_count.to_string())
            .arg("--retry-delay")
            .arg(self.settings.retry_delay_seconds.to_string())
            .arg("--dump-header")
            .arg(header_path)
            .arg("--output")
            .arg(destination)
            .arg("--write-out")
            .arg(format!("{HTTP_CODE_MARKER}%{{http_code}}\n{EFFECTIVE_URL_MARKER}%{{url_effective}}\n{REDIRECT_URL_MARKER}%{{redirect_url}}\n"));
        for address in &validated_url.resolved_addresses {
            let address_text = match address {
                std::net::IpAddr::V4(value) => value.to_string(),
                std::net::IpAddr::V6(value) => format!("[{value}]"),
            };
            command
                .arg("--resolve")
                .arg(format!("{}:{}:{}", validated_url.host, validated_url.port, address_text));
        }
        command
            .arg(source_url)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let output = run_with_timeout(command, self.settings.request_timeout)?;
        if !output.status.success() {
            return Err(format!(
                "artifact GET failed (curl {:?}): {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        Ok(output)
    }

}

fn run_with_timeout(mut command: Command, timeout: Duration) -> Result<std::process::Output, String> {
    let mut child = command
        .spawn()
        .map_err(|error| format!("curl launch failed: {error}"))?;
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return child.wait_with_output().map_err(|error| error.to_string()),
            Ok(None) if started.elapsed() < timeout => thread::sleep(Duration::from_millis(25)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(String::from("artifact GET timed out"));
            }
            Err(error) => return Err(error.to_string()),
        }
    }
}

fn parse_marker_u16(output: &str, marker: &str) -> Result<u16, String> {
    parse_marker_string(output, marker)?
        .parse::<u16>()
        .map_err(|_| format!("{marker} value is invalid"))
}

fn parse_marker_string(output: &str, marker: &str) -> Result<String, String> {
    output
        .lines()
        .rev()
        .find_map(|line| line.trim().strip_prefix(marker))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| format!("{marker} is missing"))
}
