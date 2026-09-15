# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_artifact_cache_v21_contract.py
# 📌 Amac: v0.21 mutable Artifact Cache HTTP ve hardening contractini regression seviyesinde dogrular
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Same-GET validator capture, conditional 304, remote change 200 ve source-lock/cert/ACL invariantlarini test eder
# Bagimli Oldugu Katman: Tool | Service | Repo | View

from __future__ import annotations

import http.server
import shutil
import socketserver
import subprocess
import tempfile
import threading
from contextlib import contextmanager
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
LAST_MODIFIED = "Wed, 26 Aug 2026 19:00:00 GMT"


class State:
    etag = '"tvm-v21-a"'
    payload = b"payload-a"


class Handler(http.server.BaseHTTPRequestHandler):
    def _conditional(self) -> bool:
        return self.headers.get("If-None-Match") == State.etag

    def do_HEAD(self) -> None:  # noqa: N802
        self.send_response(304 if self._conditional() else 200)
        self.send_header("ETag", State.etag)
        self.send_header("Last-Modified", LAST_MODIFIED)
        self.end_headers()

    def do_GET(self) -> None:  # noqa: N802
        self.send_response(200)
        self.send_header("Content-Length", str(len(State.payload)))
        self.send_header("ETag", State.etag)
        self.send_header("Last-Modified", LAST_MODIFIED)
        self.end_headers()
        self.wfile.write(State.payload)

    def log_message(self, _format: str, *_args: object) -> None:
        return


@contextmanager
def server():
    with socketserver.TCPServer(("127.0.0.1", 0), Handler) as httpd:
        thread = threading.Thread(target=httpd.serve_forever, daemon=True)
        thread.start()
        try:
            host, port = httpd.server_address
            yield f"http://{host}:{port}/artifact.bin"
        finally:
            httpd.shutdown()
            thread.join(timeout=2)


def run_curl(args: list[str]) -> subprocess.CompletedProcess[str]:
    curl = shutil.which("curl") or shutil.which("curl.exe")
    if curl is None:
        raise RuntimeError("curl is required")
    return subprocess.run([curl, *args], check=True, capture_output=True, text=True)


def static_invariants() -> None:
    repo = (ROOT / "crates/repositories/src/repositories/yaml_artifact_cache_repository.rs").read_text()
    agent = (ROOT / "crates/guest/src/tools/android_guest_agent_tool.rs").read_text()
    fetcher = (ROOT / "crates/guest/src/tools/artifact_cache_http_fetch_tool.rs").read_text()
    validator = (ROOT / "crates/guest/src/tools/artifact_cache_http_validation_tool.rs").read_text()
    desktop = (ROOT / "apps/desktop/ui/app.js").read_text()
    checks = {
        "future schema fail-closed": "UnsupportedFutureSchema" in repo,
        "source file lock": "held_source_locks" in repo and "lock_exclusive" in repo,
        "PIN preservation": "existing.as_ref().map(|record| record.pinned)" in repo,
        "transaction backup": "bak.{nonce}" in repo,
        "manual fetch redirect validation": "validate_artifact_url(&redirect_url" in fetcher,
        "manual validator redirect validation": "validate_artifact_url(&redirect_url" in validator,
        "APK cert pin": "trusted_apk_cert_sha256" in agent and "apk_signing_cert_sha256" in agent,
        "rollback hash": "ROLLBACK_SHA256_FILE" in agent,
        "rollback root ACL": "set_private_directory_permissions(&self.settings.update.rollback_root)" in agent,
        "secret final ACL": "TVGB secret final ACL failed" in agent,
        "desktop LKG semantics": "report.remote_modified" in desktop and "report.invalidated" not in desktop,
    }
    failed = [name for name, ok in checks.items() if not ok]
    if failed:
        raise AssertionError("missing v0.21 invariants: " + ", ".join(failed))


def http_contract() -> None:
    State.etag = '"tvm-v21-a"'
    State.payload = b"payload-a"
    with server() as url, tempfile.TemporaryDirectory(prefix="tvm-v21-http-") as tmp:
        body = Path(tmp) / "artifact.bin"
        headers = Path(tmp) / "headers.txt"
        run_curl(["--silent", "--show-error", "--dump-header", str(headers), "--output", str(body), url])
        text = headers.read_text(errors="replace")
        if body.read_bytes() != State.payload:
            raise AssertionError("GET payload mismatch")
        if f"ETag: {State.etag}".lower() not in text.lower() or f"Last-Modified: {LAST_MODIFIED}".lower() not in text.lower():
            raise AssertionError("same GET response validators missing")

        unchanged = run_curl(["--silent", "--show-error", "--head", "-H", f"If-None-Match: {State.etag}", "--write-out", "%{http_code}", url])
        if not unchanged.stdout.rstrip().endswith("304"):
            raise AssertionError("conditional HEAD must return 304")

        old_etag = State.etag
        State.etag = '"tvm-v21-b"'
        State.payload = b"payload-b"
        changed = run_curl(["--silent", "--show-error", "--head", "-H", f"If-None-Match: {old_etag}", "--write-out", "%{http_code}", url])
        if not changed.stdout.rstrip().endswith("200"):
            raise AssertionError("remote validator change must return 200")


def main() -> int:
    static_invariants()
    http_contract()
    print("ARTIFACT_CACHE_V21_CONTRACT=PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
