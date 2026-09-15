# 📄 Dosya Yolu: /turkuazvm/tools/virtiofs_tool.py
# 📌 Amac: Shared-folder capability kontrolu, host path confinement ve read-only virtiofsd process lifecycle yonetimini uygular
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Linux hostta capability varsa virtiofsd baslatir; Windows guest gereksinimlerini Tech Preview gate'i ile fail-closed denetler
# Bagimli Oldugu Katman: Tool | Service

from __future__ import annotations

import re
import shutil
import subprocess
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from tools.runtime_config import RuntimeConfig


class VirtioFsError(RuntimeError):
    pass


@dataclass
class VirtioFsSession:
    tag: str
    source: Path
    socket_path: Path
    read_only: bool
    process: subprocess.Popen[str] | None = None


class VirtioFsTool:
    SAFE_ID_RE = re.compile(r"^[A-Za-z0-9_.-]{1,64}$")

    def __init__(self, config: RuntimeConfig) -> None:
        self.config = config
        self.cfg = config.runtime["virtiofs"]
        self.binary = str(config.runtime["binaries"]["virtiofsd"])

    def validate_guest_capability(self, guest_os: str, capabilities: set[str]) -> None:
        if guest_os == "windows":
            required = set(self.cfg["windows_guest_required_capabilities"])
            if not required.issubset(capabilities):
                missing = sorted(required - capabilities)
                raise VirtioFsError(f"{self.config.codes['windows_virtiofs_blocked']} missing Windows capabilities: {missing}")
            if self.config.policy["guest_integration"]["shared_folder"]["backends"]["windows"]["maturity"] != "experimental":
                raise VirtioFsError(f"{self.config.codes['windows_virtiofs_blocked']} Windows maturity policy drift")
            return
        if guest_os == "linux":
            required = set(self.cfg["linux_guest_required_capabilities"])
            if not required.issubset(capabilities):
                missing = sorted(required - capabilities)
                raise VirtioFsError(f"{self.config.codes['capability_missing']} missing Linux capabilities: {missing}")
            return
        raise VirtioFsError(f"{self.config.codes['capability_missing']} unsupported guest OS")

    def confine_share_path(self, requested: Path, allow_roots: list[Path]) -> Path:
        if not allow_roots:
            raise VirtioFsError(f"{self.config.codes['share_path_rejected']} host share allowlist is empty")
        try:
            resolved = requested.resolve(strict=True)
        except OSError as exc:
            raise VirtioFsError(f"{self.config.codes['share_path_rejected']} share path cannot be resolved") from exc
        if not resolved.is_dir():
            raise VirtioFsError(f"{self.config.codes['share_path_rejected']} share path is not a directory")
        for root in allow_roots:
            try:
                root_resolved = root.resolve(strict=True)
            except OSError:
                continue
            try:
                resolved.relative_to(root_resolved)
                return resolved
            except ValueError:
                continue
        raise VirtioFsError(f"{self.config.codes['share_path_rejected']} share path outside allowlist")

    def start_read_only(
        self,
        *,
        source: Path,
        allow_roots: list[Path],
        socket_path: Path,
        tag: str,
    ) -> VirtioFsSession:
        if shutil.which(self.binary) is None:
            raise VirtioFsError(f"{self.config.codes['capability_missing']} virtiofsd missing")
        confined = self.confine_share_path(source, allow_roots)
        socket_path.parent.mkdir(parents=True, exist_ok=True)
        socket_path.unlink(missing_ok=True)
        flags = self.cfg["daemon_flags"]
        argv = [
            self.binary,
            str(flags["socket_path"]),
            str(socket_path),
            str(flags["shared_dir"]),
            str(confined),
            str(flags["read_only"]),
            str(flags["sandbox"]),
            str(self.cfg["sandbox_mode"]),
        ]
        try:
            process = subprocess.Popen(
                argv,
                stdin=subprocess.DEVNULL,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                shell=False,
            )
        except OSError as exc:
            raise VirtioFsError(f"{self.config.codes['virtiofs_start_failed']} virtiofsd start failed: {exc}") from exc

        deadline = time.monotonic() + float(self.cfg["startup_timeout_seconds"])
        while time.monotonic() < deadline:
            if socket_path.exists():
                return VirtioFsSession(tag=tag, source=confined, socket_path=socket_path, read_only=True, process=process)
            if process.poll() is not None:
                stderr = process.stderr.read() if process.stderr else ""
                raise VirtioFsError(f"{self.config.codes['virtiofs_start_failed']} virtiofsd exited: {stderr.strip()}")
            time.sleep(0.05)
        self.stop(VirtioFsSession(tag, confined, socket_path, True, process))
        raise VirtioFsError(f"{self.config.codes['virtiofs_start_failed']} virtiofsd socket timeout")

    def stop(self, session: VirtioFsSession) -> None:
        process = session.process
        if process is None or process.poll() is not None:
            session.socket_path.unlink(missing_ok=True)
            return
        process.terminate()
        try:
            process.wait(timeout=float(self.cfg["terminate_timeout_seconds"]))
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()
        session.socket_path.unlink(missing_ok=True)

    def qemu_device_fragment(self, session: VirtioFsSession, chardev_id: str, device_id: str) -> list[str]:
        for value in (session.tag, chardev_id, device_id):
            if not self.SAFE_ID_RE.fullmatch(value):
                raise VirtioFsError(f"{self.config.codes['share_path_rejected']} unsafe QEMU identifier")
        if "," in str(session.socket_path):
            raise VirtioFsError(f"{self.config.codes['share_path_rejected']} unsafe QEMU socket path")
        return [
            "-chardev",
            f"socket,id={chardev_id},path={session.socket_path}",
            "-device",
            f"vhost-user-fs-pci,id={device_id},chardev={chardev_id},tag={session.tag}",
        ]
