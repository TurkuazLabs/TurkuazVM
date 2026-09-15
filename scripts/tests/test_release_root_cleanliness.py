# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_release_root_cleanliness.py
# 📌 Amac: FULL paket kokunde yalniz aktif surum release artefactlarinin kaldigini fail-closed dogrular
# 📌 Modul - Python
# Version: 0.30.0
# Aciklama: Eski manifest, release status, patch, validation, cumulative recovery ve tarihsel docs/versions artefactlarinin FULL pakete sizmasini engeller
# Bagimli Oldugu Katman: Tool | View

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def workspace_version() -> str:
    text = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    match = re.search(r"(?ms)^\[workspace\.package\].*?^version\s*=\s*\"([^\"]+)\"", text)
    if not match:
        raise AssertionError("workspace version missing")
    return match.group(1)


def main() -> int:
    version = workspace_version()
    expected_status = f"RELEASE_STATUS_v{version}.yml"
    expected_manifest = f"MANIFEST_SHA256_v{version}.txt"

    status_files = sorted(path.name for path in ROOT.glob("RELEASE_STATUS_v*.yml"))
    assert status_files == [expected_status], status_files

    manifest_files = sorted(path.name for path in ROOT.glob("MANIFEST_SHA256_v*.txt"))
    if manifest_files:
        assert manifest_files == [expected_manifest], manifest_files

    forbidden_patterns = (
        "PATCH_INTEGRATION*.md",
        "VALIDATION_*.md",
        "RELEASE_GATE_STATUS_*.yml",
        "MANIFEST_SHA256_CUMULATIVE*.txt",
        "MANIFEST_SHA256_I.txt",
        "MANIFEST_SHA256_L.txt",
        "RECOVERY_WORK_STATUS.yml",
        "FULL_MANIFEST_SHA256.txt",
        "MIGRATION_*_SUPERSESSION_*.yml",
    )
    leaked: list[str] = []
    for pattern in forbidden_patterns:
        leaked.extend(path.name for path in ROOT.glob(pattern))
    assert not leaked, sorted(set(leaked))

    recovery = ROOT / "docs" / "recovery" / "FULL_RECOVERY_STATUS.yml"
    policy = ROOT / "docs" / "releases" / "RELEASE_ARTIFACT_POLICY.md"
    assert recovery.is_file()
    assert policy.is_file()

    versions_root = ROOT / "docs" / "versions"
    version_entries = sorted(path.name for path in versions_root.iterdir()) if versions_root.is_dir() else []
    expected_version_doc = f"v{version}.md"
    assert version_entries == [expected_version_doc], version_entries

    print(
        f"RELEASE_ROOT_CLEANLINESS_OK version={version} "
        f"root_release_files={1 + len(manifest_files)} version_docs={len(version_entries)}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
