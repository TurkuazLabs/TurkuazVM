# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_artifact_cache_http_revalidation.py
# 📌 Amac: Artifact Cache mutable HTTP ETag/Last-Modified conditional revalidation contractini gercek curl ile dogrular
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Local HEAD endpointinde 200 validator kesfi ve If-None-Match/If-Modified-Since ile 304 sonucunu test eder
# Bagimli Oldugu Katman: Tool | View

from __future__ import annotations

import http.server
import shutil
import socketserver
import subprocess
import threading
from contextlib import contextmanager

ETAG = '"tvm-cache-v20"'
LAST_MODIFIED = "Wed, 26 Aug 2026 00:00:00 GMT"
HTTP_CODE_MARKER = "TVM_HTTP_CODE:"


class ValidationHandler(http.server.BaseHTTPRequestHandler):
    def do_HEAD(self) -> None:  # noqa: N802
        etag_match = self.headers.get("If-None-Match") == ETAG
        modified_match = self.headers.get("If-Modified-Since") == LAST_MODIFIED
        if etag_match or modified_match:
            self.send_response(304)
        else:
            self.send_response(200)
        self.send_header("ETag", ETAG)
        self.send_header("Last-Modified", LAST_MODIFIED)
        self.end_headers()

    def log_message(self, _format: str, *_args: object) -> None:
        return


@contextmanager
def validation_server():
    with socketserver.TCPServer(("127.0.0.1", 0), ValidationHandler) as server:
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            host, port = server.server_address
            yield f"http://{host}:{port}/artifact.bin"
        finally:
            server.shutdown()
            thread.join(timeout=2)


def curl_head(curl: str, url: str, headers: list[str]) -> str:
    command = [
        curl,
        "-L",
        "--silent",
        "--show-error",
        "--head",
        "--connect-timeout",
        "5",
        "--max-time",
        "5",
    ]
    for header in headers:
        command.extend(["-H", header])
    command.extend(["--write-out", f"\n{HTTP_CODE_MARKER}%{{http_code}}\n", url])
    completed = subprocess.run(command, check=True, capture_output=True, text=True)
    return completed.stdout


def parse_status(output: str) -> int:
    for line in reversed(output.splitlines()):
        if line.strip().startswith(HTTP_CODE_MARKER):
            return int(line.strip().removeprefix(HTTP_CODE_MARKER))
    raise AssertionError("HTTP status marker missing")


def main() -> int:
    curl = shutil.which("curl") or shutil.which("curl.exe")
    if curl is None:
        raise RuntimeError("curl is required for Artifact Cache HTTP validation")
    with validation_server() as url:
        initial = curl_head(curl, url, [])
        if parse_status(initial) != 200:
            raise AssertionError("initial validator discovery must return HTTP 200")
        if f"ETag: {ETAG}".lower() not in initial.lower():
            raise AssertionError("ETag header missing from initial HEAD")
        if f"Last-Modified: {LAST_MODIFIED}".lower() not in initial.lower():
            raise AssertionError("Last-Modified header missing from initial HEAD")
        conditional = curl_head(
            curl,
            url,
            [f"If-None-Match: {ETAG}", f"If-Modified-Since: {LAST_MODIFIED}"],
        )
        if parse_status(conditional) != 304:
            raise AssertionError("conditional validator request must return HTTP 304")
    print("ARTIFACT_CACHE_HTTP_REVALIDATION=PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
