// # 📄 Dosya Yolu: /turkuazvm/crates/guest/src/tools/artifact_cache_http_policy_tool.rs
// # 📌 Amac: Artifact Cache HTTP kaynak URL guvenlik politikasini merkezi uygular
// # 📌 Modul - Rust
// # Version: 0.28.0
// # Aciklama: HTTPS-only, credential/query redaction, private IP/SSRF ve redirect protocol sinirlarini dogrular
// # Bagimli Oldugu Katman: Tool

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, ToSocketAddrs};

#[derive(Debug, Clone, Copy)]
pub struct ArtifactHttpPolicy {
    pub require_https: bool,
    pub allow_private_networks: bool,
    pub max_redirects: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactHttpPolicyError {
    InvalidUrl(String),
    ForbiddenTarget(String),
    ResolutionFailed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedArtifactUrl {
    pub scheme: String,
    pub host: String,
    pub port: u16,
    pub resolved_addresses: Vec<IpAddr>,
}

pub fn validate_artifact_url(
    source_url: &str,
    policy: ArtifactHttpPolicy,
) -> Result<ValidatedArtifactUrl, ArtifactHttpPolicyError> {
    if source_url.contains('?') || source_url.contains('#') {
        return Err(ArtifactHttpPolicyError::InvalidUrl(String::from(
            "artifact cache URL must not contain query or fragment data",
        )));
    }
    let (scheme, rest) = source_url.split_once("://").ok_or_else(|| {
        ArtifactHttpPolicyError::InvalidUrl(String::from("artifact cache URL scheme is missing"))
    })?;
    let scheme = scheme.to_ascii_lowercase();
    if policy.require_https && scheme != "https" {
        return Err(ArtifactHttpPolicyError::ForbiddenTarget(String::from(
            "artifact cache policy requires HTTPS",
        )));
    }
    if scheme != "https" && scheme != "http" {
        return Err(ArtifactHttpPolicyError::ForbiddenTarget(String::from(
            "artifact cache policy allows only HTTP(S)",
        )));
    }
    let authority = rest.split('/').next().unwrap_or_default();
    if authority.is_empty() || authority.contains('@') {
        return Err(ArtifactHttpPolicyError::InvalidUrl(String::from(
            "artifact cache URL authority is invalid or contains userinfo",
        )));
    }
    let default_port = if scheme == "https" { 443 } else { 80 };
    let (host, port) = parse_host_port(authority, default_port)?;
    if host.eq_ignore_ascii_case("localhost") && !policy.allow_private_networks {
        return Err(ArtifactHttpPolicyError::ForbiddenTarget(String::from(
            "localhost artifact source is blocked",
        )));
    }
    let addresses = (host.as_str(), port)
        .to_socket_addrs()
        .map_err(|error| ArtifactHttpPolicyError::ResolutionFailed(error.to_string()))?
        .collect::<Vec<_>>();
    if addresses.is_empty() {
        return Err(ArtifactHttpPolicyError::ResolutionFailed(String::from(
            "artifact source resolved to no addresses",
        )));
    }
    let mut resolved_addresses = addresses.into_iter().map(|address| address.ip()).collect::<Vec<_>>();
    resolved_addresses.sort_unstable();
    resolved_addresses.dedup();
    if !policy.allow_private_networks && resolved_addresses.iter().any(|address| !is_public_ip(*address)) {
        return Err(ArtifactHttpPolicyError::ForbiddenTarget(String::from(
            "artifact source resolves to a non-public address",
        )));
    }
    Ok(ValidatedArtifactUrl { scheme, host, port, resolved_addresses })
}

pub fn curl_protocol_argument(policy: ArtifactHttpPolicy) -> &'static str {
    if policy.require_https { "=https" } else { "=http,https" }
}

fn parse_host_port(authority: &str, default_port: u16) -> Result<(String, u16), ArtifactHttpPolicyError> {
    if let Some(rest) = authority.strip_prefix('[') {
        let close = rest.find(']').ok_or_else(|| ArtifactHttpPolicyError::InvalidUrl(String::from("IPv6 authority is malformed")))?;
        let host = rest[..close].to_owned();
        let suffix = &rest[close + 1..];
        let port = if suffix.is_empty() {
            default_port
        } else {
            suffix.strip_prefix(':')
                .and_then(|value| value.parse::<u16>().ok())
                .filter(|value| *value != 0)
                .ok_or_else(|| ArtifactHttpPolicyError::InvalidUrl(String::from("artifact source port is invalid")))?
        };
        return Ok((host, port));
    }
    if authority.matches(':').count() > 1 {
        return Err(ArtifactHttpPolicyError::InvalidUrl(String::from("IPv6 host must use brackets")));
    }
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) if !host.is_empty() => {
            let port = port.parse::<u16>().ok().filter(|value| *value != 0)
                .ok_or_else(|| ArtifactHttpPolicyError::InvalidUrl(String::from("artifact source port is invalid")))?;
            (host.to_owned(), port)
        }
        _ => (authority.to_owned(), default_port),
    };
    if host.is_empty() || host.chars().any(char::is_whitespace) {
        return Err(ArtifactHttpPolicyError::InvalidUrl(String::from("artifact source host is invalid")));
    }
    Ok((host, port))
}

fn is_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => is_public_ipv4(ip),
        IpAddr::V6(ip) => is_public_ipv6(ip),
    }
}

fn is_public_ipv4(ip: Ipv4Addr) -> bool {
    let octets = ip.octets();
    !(ip.is_private()
        || ip.is_loopback()
        || ip.is_link_local()
        || ip.is_unspecified()
        || ip.is_multicast()
        || octets == [255, 255, 255, 255]
        || (octets[0] == 100 && (64..=127).contains(&octets[1]))
        || (octets[0] == 192 && octets[1] == 0 && octets[2] == 0)
        || (octets[0] == 192 && octets[1] == 0 && octets[2] == 2)
        || (octets[0] == 198 && octets[1] == 51 && octets[2] == 100)
        || (octets[0] == 203 && octets[1] == 0 && octets[2] == 113)
        || octets[0] >= 240)
}

fn is_public_ipv6(ip: Ipv6Addr) -> bool {
    if let Some(mapped) = ip.to_ipv4_mapped() {
        return is_public_ipv4(mapped);
    }
    !(ip.is_loopback()
        || ip.is_unspecified()
        || ip.is_multicast()
        || (ip.segments()[0] & 0xfe00) == 0xfc00
        || (ip.segments()[0] & 0xffc0) == 0xfe80)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_query_and_private_loopback() {
        let policy = ArtifactHttpPolicy { require_https: true, allow_private_networks: false, max_redirects: 5 };
        assert!(validate_artifact_url("https://example.com/file?token=x", policy).is_err());
        assert!(validate_artifact_url("https://127.0.0.1/file", policy).is_err());
    }

    #[test]
    fn rejects_ipv4_mapped_loopback() {
        assert!(!is_public_ipv6("::ffff:127.0.0.1".parse::<Ipv6Addr>().expect("mapped ipv6")));
    }

    #[test]
    fn allows_http_loopback_only_when_test_policy_allows_it() {
        let policy = ArtifactHttpPolicy { require_https: false, allow_private_networks: true, max_redirects: 1 };
        assert!(validate_artifact_url("http://127.0.0.1:8080/file", policy).is_ok());
    }
}
