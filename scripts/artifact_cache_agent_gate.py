# 📄 Dosya Yolu: /turkuazvm/scripts/artifact_cache_agent_gate.py
# 📌 Amac: Artifact Cache ve signed Android Guest Agent v0.21 entegrasyonunu statik olarak dogrular
# 📌 Modul - Python
# Version: 0.40.0
# Aciklama: Minimum config schema, transactional cache, secure HTTP, source lock, APK cert pin, ACL ve rollback invariantlarini fail-closed denetler
# Bagimli Oldugu Katman: Tool | View

from __future__ import annotations

import re
import tomllib
from pathlib import Path

import yaml

MINIMUM_SCHEMA = 21
EXPECTED_API = 24
SEMVER_PATTERN = re.compile(r"^[0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?$")


def root_dir() -> Path:
    return Path(__file__).resolve().parent.parent


def read(root: Path, relative: str) -> str:
    return (root / relative).read_text(encoding="utf-8")


def require(errors: list[str], condition: bool, message: str) -> None:
    if not condition:
        errors.append(message)


def validate(root: Path) -> list[str]:
    errors: list[str] = []

    for relative in (
        "config/turkuazvm.yml",
        "config/turkuazvm.remote-engine.example.yml",
        "config/turkuazvm.desktop-remote.example.yml",
    ):
        document = yaml.safe_load(read(root, relative))
        require(errors, int(document.get("schema_version", 0)) >= MINIMUM_SCHEMA, f"{relative}: schema21+ required")
        cache = document.get("artifact_cache") or {}
        mutable = cache.get("mutable") or {}
        require(errors, mutable.get("allow_stale_on_transient_error") is True, f"{relative}: transient stale policy missing")
        require(errors, mutable.get("require_https") is True, f"{relative}: HTTPS policy missing")
        require(errors, mutable.get("allow_private_networks") is False, f"{relative}: private network default must be false")
        require(errors, int(mutable.get("max_redirects", 0)) > 0, f"{relative}: max_redirects invalid")
        update = (((document.get("android") or {}).get("guest_agent") or {}).get("update") or {})
        for key in ("trusted_public_key_hex", "trusted_apk_cert_sha256", "apksigner_path", "rollback_root"):
            require(errors, key in update, f"{relative}: Guest Agent update field missing {key}")

    with (root / "Cargo.toml").open("rb") as handle:
        cargo = tomllib.load(handle)
    workspace_version = cargo.get("workspace", {}).get("package", {}).get("version")
    require(errors, isinstance(workspace_version, str) and SEMVER_PATTERN.fullmatch(workspace_version) is not None, "workspace version must be valid SemVer")

    api = read(root, "crates/engine-api/src/lib.rs")
    require(errors, f"ENGINE_API_VERSION: u16 = {EXPECTED_API}" in api, "Engine API v24 missing")
    for token in ("FetchMutableArtifactCache", "ArtifactCacheEntry(ArtifactCacheEntryDto)", "RemoteModified", "remote_modified"):
        require(errors, token in api, f"Engine API missing {token}")

    repo_port = read(root, "crates/artifact-cache/src/ports/artifact_cache_repository_port.rs")
    for token in ("UnsupportedFutureSchema(u16)", "acquire_source_lock", "release_source_lock"):
        require(errors, token in repo_port, f"cache repository port missing {token}")

    repo = read(root, "crates/repositories/src/repositories/yaml_artifact_cache_repository.rs")
    for token in (
        "use fs2::FileExt",
        "held_source_locks",
        "UnsupportedFutureSchema",
        "acquire_source_lock",
        "digest_lock",
        "artifact source_key is already bound to a different URL",
        "existing.as_ref().map(|record| record.pinned)",
        "let backup = path.with_extension(format!(\"bak.{nonce}\"))",
    ):
        require(errors, token in repo, f"cache repository missing {token}")
    require(errors, "UnsupportedFutureSchema(version)" in repo and "quarantine_index(&path)" in repo, "future-schema/quarantine handling missing")

    cache_service = read(root, "crates/artifact-cache/src/services/artifact_cache_service.rs")
    for token in ("allow_stale_on_transient_error", "ArtifactSourceValidationError::Transient", "RemoteModified", "find_record"):
        require(errors, token in cache_service, f"cache service missing {token}")
    require(errors, "remove(&record)" not in cache_service[cache_service.find("pub fn revalidate_source"):cache_service.find("pub fn revalidate_all")], "revalidate must retain LKG on remote modification")

    distro = read(root, "crates/guest/src/tools/android_ci_distribution_tool.rs")
    for token in ("acquire_source_lock", "release_source_lock", "download_cached", "Artifact Cache HIT", "Artifact Cache MISS"):
        require(errors, token in distro, f"Android downloader missing {token}")

    policy = read(root, "crates/guest/src/tools/artifact_cache_http_policy_tool.rs")
    for token in ("require_https", "allow_private_networks", "max_redirects", "query", "userinfo"):
        require(errors, token in policy, f"HTTP policy missing {token}")

    validator = read(root, "crates/guest/src/tools/artifact_cache_http_validation_tool.rs")
    fetcher = read(root, "crates/guest/src/tools/artifact_cache_http_fetch_tool.rs")
    for text, name in ((validator, "validator"), (fetcher, "fetcher")):
        for token in ("--proto", "--proto-redir", "redirect_url", "validate_artifact_url"):
            require(errors, token in text, f"HTTP {name} missing {token}")
    for token in ("If-None-Match", "If-Modified-Since"):
        require(errors, token in validator, f"validator missing {token}")
    require(errors, "parse_final_validators" in fetcher, "GET fetch must capture validators from same response")

    app_service = read(root, "apps/engine/src/services/artifact_cache_application_service.rs")
    for token in ("pub fn fetch_mutable", "fetch_mutable_locked", "artifact source_key is already bound to a different URL", "self.fetcher.fetch"):
        require(errors, token in app_service, f"mutable fetch service missing {token}")

    desktop = read(root, "apps/desktop/ui/app.js")
    require(errors, "report.invalidated" not in desktop, "Desktop still uses invalidated semantics")
    for token in ("report.remote_modified", "fetch_mutable_artifact_cache"):
        require(errors, token in desktop, f"Desktop cache UI missing {token}")

    agent = read(root, "crates/guest/src/tools/android_guest_agent_tool.rs")
    for token in (
        "TVM-GUEST-AGENT-V2",
        "trusted_apk_cert_sha256",
        "apk_signing_cert_sha256",
        "verify_strict",
        "ROLLBACK_SHA256_FILE",
        "set_private_directory_permissions(&self.settings.update.rollback_root)",
        "TVGB secret final ACL failed",
        'args.push("-d")',
        "set_windows_private_acl",
    ):
        require(errors, token in agent, f"Guest Agent hardening missing {token}")

    manifest = read(root, "guest/android-agent/AndroidManifest.xml")
    require(errors, 'android:versionCode="2100"' in manifest and 'android:versionName="0.21.0"' in manifest, "explicit built-in Agent version missing")
    example = yaml.safe_load(read(root, "packages/guest-agent/manifest.example.yml"))
    require(errors, example.get("schema_version") == 2 and "apk_signing_cert_sha256" in example, "signed manifest schema2/cert digest missing")
    helper = read(root, "scripts/release_guest_agent_update.py")
    for token in ("platform-pk8", "platform-x509-pem", "apksigner", "trusted_apk_cert_sha256", "TVM-GUEST-AGENT-V2"):
        require(errors, token in helper, f"release Agent helper missing {token}")

    return errors


def main() -> int:
    errors = validate(root_dir())
    if errors:
        print("ARTIFACT_CACHE_AGENT_GATE=BLOCKED")
        for error in errors:
            print(f"- {error}")
        return 2
    print("ARTIFACT_CACHE_AGENT_GATE=PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
