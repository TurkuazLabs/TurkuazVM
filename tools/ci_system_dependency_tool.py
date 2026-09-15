# 📄 Dosya Yolu: /turkuazvm/tools/ci_system_dependency_tool.py
# 📌 Amac: CI sistem bagimliliklarini config profiline gore kurar ve sonraki GitHub Actions adimlari icin PATH export eder
# 📌 Modul - Python
# Version: 1.0.0
# Aciklama: Paket yoneticisi komutlarini merkezi configden uretir, host platformunu dogrular ve gerekli binaryleri fail-closed kontrol eder
# Bagimli Oldugu Katman: Tool

from __future__ import annotations

import argparse
import os
import platform
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Mapping, Sequence

import yaml

ROOT = Path(__file__).resolve().parents[1]
CONFIG_PATH = ROOT / "config" / "ci-system-dependencies.yml"
CONFIG_SCHEMA = 1
CONFIG_VERSION = "1.0.0"
INSTALLER_APT = "apt"
INSTALLER_CHOCOLATEY = "chocolatey"
PLATFORM_WINDOWS = "windows"
GITHUB_PATH_ENV = "GITHUB_PATH"
PATH_ENV = "PATH"
WINDOWS_ENV_PATTERN = re.compile(r"%([^%]+)%")


class CiSystemDependencyError(RuntimeError):
    pass


@dataclass(frozen=True)
class DependencyPlan:
    profile: str
    host_platform: str
    commands: tuple[tuple[str, ...], ...]
    path_exports: tuple[Path, ...]
    required_binaries: tuple[str, ...]


class CiSystemDependencyTool:
    def __init__(
        self,
        config_path: Path = CONFIG_PATH,
        *,
        environ: Mapping[str, str] | None = None,
    ) -> None:
        self.config_path = config_path
        self.environ = dict(os.environ if environ is None else environ)
        self.config = self._load_config(config_path)

    def plan(self, profile_name: str, host_platform: str | None = None) -> DependencyPlan:
        profiles = self.config["profiles"]
        profile_cfg = profiles.get(profile_name)
        if not isinstance(profile_cfg, dict):
            raise CiSystemDependencyError(f"CI_DEP_PROFILE_MISSING profile not found: {profile_name}")

        actual_platform = (host_platform or platform.system()).strip().lower()
        expected_platform = str(profile_cfg.get("host_platform", "")).strip().lower()
        if actual_platform != expected_platform:
            raise CiSystemDependencyError(
                f"CI_DEP_PLATFORM_MISMATCH profile={profile_name} expected={expected_platform} actual={actual_platform}"
            )

        installer = str(profile_cfg.get("installer", "")).strip().lower()
        packages = self._string_list(profile_cfg.get("packages"), "packages")
        required_binaries = tuple(self._string_list(profile_cfg.get("required_binaries"), "required_binaries"))
        path_exports = tuple(
            Path(self._expand_environment(value))
            for value in self._string_list(profile_cfg.get("path_exports"), "path_exports", allow_empty=True)
        )
        commands = self._installer_commands(installer, packages)
        return DependencyPlan(
            profile=profile_name,
            host_platform=actual_platform,
            commands=commands,
            path_exports=path_exports,
            required_binaries=required_binaries,
        )

    def apply(self, profile_name: str) -> DependencyPlan:
        plan = self.plan(profile_name)
        for command in plan.commands:
            self._run(command)
        self._export_paths(plan.path_exports)
        self._verify_binaries(plan.required_binaries, plan.path_exports)
        return plan

    def _installer_commands(self, installer: str, packages: Sequence[str]) -> tuple[tuple[str, ...], ...]:
        installers = self.config["installers"]
        installer_cfg = installers.get(installer)
        if not isinstance(installer_cfg, dict):
            raise CiSystemDependencyError(f"CI_DEP_INSTALLER_INVALID installer not configured: {installer}")

        if installer == INSTALLER_APT:
            update = tuple(self._string_list(installer_cfg.get("update_command"), "update_command"))
            install = tuple(self._string_list(installer_cfg.get("install_command"), "install_command")) + tuple(packages)
            return (update, install)

        if installer == INSTALLER_CHOCOLATEY:
            install = tuple(self._string_list(installer_cfg.get("install_command"), "install_command"))
            arguments = tuple(
                self._string_list(installer_cfg.get("install_arguments"), "install_arguments", allow_empty=True)
            )
            return (install + tuple(packages) + arguments,)

        raise CiSystemDependencyError(f"CI_DEP_INSTALLER_INVALID unsupported installer: {installer}")

    def _export_paths(self, paths: Sequence[Path]) -> None:
        if not paths:
            return
        github_path = self.environ.get(GITHUB_PATH_ENV)
        if not github_path:
            raise CiSystemDependencyError("CI_DEP_GITHUB_PATH_MISSING GITHUB_PATH is required for path exports")

        resolved_paths: list[str] = []
        for path in paths:
            if not path.is_dir():
                raise CiSystemDependencyError(f"CI_DEP_PATH_MISSING path not found: {path}")
            resolved_paths.append(str(path.resolve()))

        output = Path(github_path)
        output.parent.mkdir(parents=True, exist_ok=True)
        with output.open("a", encoding="utf-8", newline="\n") as stream:
            for value in resolved_paths:
                stream.write(value + "\n")

    def _verify_binaries(self, binaries: Sequence[str], exported_paths: Sequence[Path]) -> None:
        search_path = self.environ.get(PATH_ENV, "")
        if exported_paths:
            search_path = os.pathsep.join([*(str(path) for path in exported_paths), search_path])
        missing = [binary for binary in binaries if shutil.which(binary, path=search_path) is None]
        if missing:
            raise CiSystemDependencyError(f"CI_DEP_BINARY_MISSING binaries not found: {', '.join(missing)}")

    def _run(self, command: Sequence[str]) -> None:
        completed = subprocess.run(tuple(command), check=False, shell=False)
        if completed.returncode != 0:
            raise CiSystemDependencyError(
                f"CI_DEP_COMMAND_FAILED exit={completed.returncode} command={' '.join(command)}"
            )

    def _expand_environment(self, value: str) -> str:
        def replace_windows(match: re.Match[str]) -> str:
            name = match.group(1)
            return self.environ.get(name, match.group(0))

        expanded = WINDOWS_ENV_PATTERN.sub(replace_windows, value)
        for name, env_value in self.environ.items():
            expanded = expanded.replace(f"${{{name}}}", env_value).replace(f"${name}", env_value)
        return expanded

    @staticmethod
    def _string_list(value: Any, field: str, *, allow_empty: bool = False) -> list[str]:
        if value is None and allow_empty:
            return []
        if not isinstance(value, list):
            raise CiSystemDependencyError(f"CI_DEP_CONFIG_INVALID {field} must be a list")
        result = [str(item).strip() for item in value]
        if any(not item for item in result):
            raise CiSystemDependencyError(f"CI_DEP_CONFIG_INVALID {field} contains empty value")
        if not result and not allow_empty:
            raise CiSystemDependencyError(f"CI_DEP_CONFIG_INVALID {field} cannot be empty")
        return result

    @staticmethod
    def _load_config(path: Path) -> dict[str, Any]:
        if not path.is_file():
            raise CiSystemDependencyError(f"CI_DEP_CONFIG_MISSING config not found: {path}")
        payload = yaml.safe_load(path.read_text(encoding="utf-8"))
        if not isinstance(payload, dict):
            raise CiSystemDependencyError("CI_DEP_CONFIG_INVALID root must be mapping")
        if payload.get("schema") != CONFIG_SCHEMA or payload.get("version") != CONFIG_VERSION:
            raise CiSystemDependencyError("CI_DEP_CONFIG_INVALID unsupported schema/version")
        config = payload.get("ci_system_dependencies")
        if not isinstance(config, dict):
            raise CiSystemDependencyError("CI_DEP_CONFIG_INVALID ci_system_dependencies missing")
        if not isinstance(config.get("installers"), dict) or not isinstance(config.get("profiles"), dict):
            raise CiSystemDependencyError("CI_DEP_CONFIG_INVALID installers/profiles missing")
        return config


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", required=True)
    parser.add_argument("--validate-only", action="store_true")
    args = parser.parse_args(argv)

    try:
        tool = CiSystemDependencyTool()
        plan = tool.plan(args.profile) if args.validate_only else tool.apply(args.profile)
    except CiSystemDependencyError as exc:
        print(str(exc), file=sys.stderr)
        return 1

    print(
        f"CI SYSTEM DEPENDENCY: PASS profile={plan.profile} platform={plan.host_platform} commands={len(plan.commands)}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
