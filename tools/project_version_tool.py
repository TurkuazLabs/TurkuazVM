# 📄 Dosya Yolu: /turkuazvm/tools/project_version_tool.py
# 📌 Amac: TurkuazVM workspace surumunu tek otorite olan Cargo.toml dosyasindan okur
# 📌 Modul - Python
# Version: 0.37.0
# Aciklama: Runtime tarafindan uretilen YAML, evidence ve ayar dosyalarinda stale surum header'i olusmasini engeller
# Bagimli Oldugu Katman: Tool

from __future__ import annotations

from functools import lru_cache
from pathlib import Path
import re

_VERSION_PATTERN = re.compile(r'^version\s*=\s*"([0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?)"\s*$')


class ProjectVersionError(RuntimeError):
    pass


@lru_cache(maxsize=1)
def project_version(root: Path | None = None) -> str:
    workspace_root = (root or Path(__file__).resolve().parents[1]).resolve()
    manifest = workspace_root / "Cargo.toml"
    if not manifest.is_file():
        raise ProjectVersionError(f"WORKSPACE_MANIFEST_NOT_FOUND: {manifest}")

    in_workspace_package = False
    for raw_line in manifest.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if line == "[workspace.package]":
            in_workspace_package = True
            continue
        if in_workspace_package and line.startswith("["):
            break
        if in_workspace_package:
            match = _VERSION_PATTERN.match(line)
            if match:
                return match.group(1)

    raise ProjectVersionError("WORKSPACE_VERSION_NOT_FOUND")
