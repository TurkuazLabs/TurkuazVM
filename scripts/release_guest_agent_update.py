# 📄 Dosya Yolu: /turkuazvm/scripts/release_guest_agent_update.py
# 📌 Amac: Android Guest Agent update APK'sini platform certificate ile imzalar ve Ed25519 manifest schema 2 uretir
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: APK package/version/cert/SHA-256 zincirini dogrular; private keyleri release paketine kopyalamaz
# Bagimli Oldugu Katman: Tool

from __future__ import annotations

import argparse
import hashlib
import re
import shutil
import subprocess
import tempfile
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from tools.project_version_tool import project_version

PACKAGE_NAME = "com.turkuazvm.inputagent"
MIN_UPDATE_VERSION_CODE = 2101
APK_NAME = "TurkuazInputAgent.apk"
SPKI_ED25519_PREFIX = bytes.fromhex("302a300506032b6570032100")


def run(args: list[str], *, capture: bool = False) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        args,
        check=True,
        text=True,
        capture_output=capture,
        stdin=subprocess.DEVNULL,
    )


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def parse_badging(text: str) -> tuple[str, int]:
    match = re.search(r"package: name='([^']+)' versionCode='([0-9]+)'", text)
    if not match:
        raise RuntimeError("aapt package/versionCode output is missing")
    return match.group(1), int(match.group(2))


def parse_cert_digest(text: str) -> str:
    match = re.search(r"Signer #1 certificate SHA-256 digest:\s*([0-9a-fA-F]{64})", text)
    if not match:
        raise RuntimeError("apksigner certificate SHA-256 digest is missing")
    return match.group(1).lower()


def signed_payload(version_code: int, apk_sha256: str, cert_sha256: str) -> bytes:
    return (
        "TVM-GUEST-AGENT-V2\n"
        f"package_name={PACKAGE_NAME}\n"
        f"version_code={version_code}\n"
        f"apk_file={APK_NAME}\n"
        f"sha256={apk_sha256}\n"
        f"apk_signing_cert_sha256={cert_sha256}\n"
    ).encode("utf-8")


def ed25519_public_key_hex(openssl: str, private_key: Path, temp: Path) -> str:
    der = temp / "ed25519-public.der"
    run([openssl, "pkey", "-in", str(private_key), "-pubout", "-outform", "DER", "-out", str(der)])
    value = der.read_bytes()
    if len(value) != len(SPKI_ED25519_PREFIX) + 32 or not value.startswith(SPKI_ED25519_PREFIX):
        raise RuntimeError("release manifest key is not Ed25519")
    return value[len(SPKI_ED25519_PREFIX):].hex()


def write_manifest(path: Path, version_code: int, apk_sha256: str, cert_sha256: str, signature_hex: str) -> None:
    path.write_text(
        "# 📄 Dosya Yolu: /turkuazvm/packages/guest-agent/manifest.yml\n"
        "# 📌 Amac: Signed Android Guest Agent update metadata'sini tasir\n"
        "# 📌 Modul - YAML\n"
        f"# Version: {project_version()}\n"
        "# Aciklama: APK SHA-256, platform signing certificate SHA-256 ve Ed25519 detached signature kaydidir\n"
        "# Bagimli Oldugu Katman: Service | Tool\n\n"
        "schema_version: 2\n"
        f'package_name: "{PACKAGE_NAME}"\n'
        f"version_code: {version_code}\n"
        f'apk_file: "{APK_NAME}"\n'
        f'sha256: "{apk_sha256}"\n'
        f'apk_signing_cert_sha256: "{cert_sha256}"\n'
        f'signature_hex: "{signature_hex}"\n',
        encoding="utf-8",
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input-apk", required=True, type=Path)
    parser.add_argument("--output-dir", required=True, type=Path)
    parser.add_argument("--platform-pk8", required=True, type=Path)
    parser.add_argument("--platform-x509-pem", required=True, type=Path)
    parser.add_argument("--ed25519-private-key", required=True, type=Path)
    parser.add_argument("--apksigner", default="apksigner")
    parser.add_argument("--aapt", default="aapt")
    parser.add_argument("--openssl", default="openssl")
    args = parser.parse_args()

    for required in (args.input_apk, args.platform_pk8, args.platform_x509_pem, args.ed25519_private_key):
        if not required.is_file():
            raise FileNotFoundError(required)
    args.output_dir.mkdir(parents=True, exist_ok=True)
    output_apk = args.output_dir / APK_NAME
    shutil.copy2(args.input_apk, output_apk)

    run([
        args.apksigner,
        "sign",
        "--key", str(args.platform_pk8),
        "--cert", str(args.platform_x509_pem),
        str(output_apk),
    ])
    verify = run([args.apksigner, "verify", "--print-certs", str(output_apk)], capture=True)
    cert_sha256 = parse_cert_digest(verify.stdout + "\n" + verify.stderr)
    badging = run([args.aapt, "dump", "badging", str(output_apk)], capture=True)
    package_name, version_code = parse_badging(badging.stdout)
    if package_name != PACKAGE_NAME:
        raise RuntimeError(f"unexpected package name: {package_name}")
    if version_code < MIN_UPDATE_VERSION_CODE:
        raise RuntimeError(
            f"update versionCode must be >= {MIN_UPDATE_VERSION_CODE}; APK contains {version_code}"
        )
    apk_sha256 = sha256_file(output_apk)

    with tempfile.TemporaryDirectory(prefix="tvm-agent-release-") as temp_dir:
        temp = Path(temp_dir)
        message = temp / "manifest-message.bin"
        signature = temp / "manifest-signature.bin"
        message.write_bytes(signed_payload(version_code, apk_sha256, cert_sha256))
        run([
            args.openssl,
            "pkeyutl",
            "-sign",
            "-rawin",
            "-inkey", str(args.ed25519_private_key),
            "-in", str(message),
            "-out", str(signature),
        ])
        signature_bytes = signature.read_bytes()
        if len(signature_bytes) != 64:
            raise RuntimeError("Ed25519 signature length is invalid")
        public_key_hex = ed25519_public_key_hex(args.openssl, args.ed25519_private_key, temp)

    manifest = args.output_dir / "manifest.yml"
    write_manifest(manifest, version_code, apk_sha256, cert_sha256, signature_bytes.hex())
    print(f"apk={output_apk}")
    print(f"manifest={manifest}")
    print(f"trusted_public_key_hex={public_key_hex}")
    print(f"trusted_apk_cert_sha256={cert_sha256}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
