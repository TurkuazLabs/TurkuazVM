# 📄 Dosya Yolu: /turkuazvm/scripts/generate_release_manifest.py
# 📌 Amac: Aktif TurkuazVM FULL release icin tekrarlanabilir SHA-256 manifesti uretir
# 📌 Modul - Python
# Version: 0.32.2
# Aciklama: Workspace surumunu Cargo.toml'dan okuyup manifestin kendisi haric tum dosyalari sirali olarak hashler
# Bagimli Oldugu Katman: Tool

from __future__ import annotations

import hashlib
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def workspace_version() -> str:
    text = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    match = re.search(r'(?ms)^\[workspace\.package\].*?^version\s*=\s*"([^"]+)"', text)
    if match is None:
        raise RuntimeError("workspace version missing")
    return match.group(1)


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def main() -> int:
    version = workspace_version()
    manifest = ROOT / f"MANIFEST_SHA256_v{version}.txt"
    paths = sorted(
        path for path in ROOT.rglob("*")
        if path.is_file() and path != manifest and "__pycache__" not in path.parts
    )
    header = (
        f"# 📄 Dosya Yolu: /turkuazvm/{manifest.name}\n"
        f"# 📌 Amac: TurkuazVM v{version} FULL kaynak paketindeki dosyalarin SHA-256 butunluk kaydini tutar\n"
        "# 📌 Modul - Text\n"
        f"# Version: {version}\n"
        "# Aciklama: Manifest dosyasinin kendisi haric tum release dosyalarinin sirali SHA-256 degerlerini listeler\n"
        "# Bagimli Oldugu Katman: Tool\n\n"
    )
    entries = "\n".join(
        f"{sha256_file(path)}  {path.relative_to(ROOT).as_posix()}"
        for path in paths
    )
    manifest.write_text(f"{header}{entries}\n", encoding="utf-8")
    print(f"MANIFEST_CREATED path={manifest.name} files={len(paths)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
