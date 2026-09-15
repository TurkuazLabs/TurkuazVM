# 📄 Dosya Yolu: /turkuazvm/tools/command_runner.py
# 📌 Amac: Runtime dis proseslerini shell kullanmadan, timeout ve output limiti ile guvenli sekilde calistirir
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: qemu-img, virtiofsd, guest bridge ve secret rewrap adaptorleri icin ortak subprocess sinirini saglar
# Bagimli Oldugu Katman: Tool

from __future__ import annotations

import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

from tools.runtime_config import RuntimeConfig


class CommandError(RuntimeError):
    pass


@dataclass(frozen=True)
class CommandResult:
    argv: tuple[str, ...]
    returncode: int
    stdout: str
    stderr: str


class CommandRunner:
    def __init__(self, config: RuntimeConfig) -> None:
        self.config = config
        self.timeout = int(config.runtime["command"]["default_timeout_seconds"])
        self.max_output = int(config.runtime["command"]["max_output_bytes"])

    def run(
        self,
        argv: Iterable[str],
        *,
        timeout: int | None = None,
        cwd: Path | None = None,
        input_text: str | None = None,
        allowed_returncodes: tuple[int, ...] = (0,),
    ) -> CommandResult:
        args = tuple(str(item) for item in argv)
        if not args or any("\x00" in item for item in args):
            raise CommandError(f"{self.config.codes['unsafe_command_blocked']} invalid argv")
        try:
            completed = subprocess.run(
                args,
                cwd=str(cwd) if cwd else None,
                input=input_text,
                capture_output=True,
                text=True,
                shell=False,
                timeout=timeout or self.timeout,
                check=False,
            )
        except FileNotFoundError as exc:
            raise CommandError(f"{self.config.codes['dependency_missing']} executable missing: {args[0]}") from exc
        except subprocess.TimeoutExpired as exc:
            raise CommandError(f"{self.config.codes['dependency_missing']} command timeout: {args[0]}") from exc

        stdout = completed.stdout[-self.max_output :]
        stderr = completed.stderr[-self.max_output :]
        result = CommandResult(args, completed.returncode, stdout, stderr)
        if completed.returncode not in allowed_returncodes:
            message = stderr.strip() or stdout.strip() or "command failed"
            raise CommandError(f"{self.config.codes['dependency_missing']} rc={completed.returncode}: {message}")
        return result
