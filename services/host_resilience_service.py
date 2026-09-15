# 📄 Dosya Yolu: /turkuazvm/services/host_resilience_service.py
# 📌 Amac: K runtime image rescue, hardware rebind, network recovery, drag-drop ve shared-folder is kurallarini orkestre eder
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Controller'dan gelen talepleri policy ile uygular; Repo ve Tool katmanlari disinda IO veya platform logic barindirmayan ana Service katmanidir
# Bagimli Oldugu Katman: Service | Repo | Tool | View | Language

from __future__ import annotations

import shutil
import time
from pathlib import Path
from typing import Any

from repositories.recovery_repository import RecoveryRepository, RecoveryRepositoryError
from tools.guest_network_tool import GuestNetworkError, GuestNetworkTool
from tools.host_capability_tool import HostCapabilityTool
from tools.network_recovery_tool import NetworkRecoveryError, NetworkRecoveryTool, NicRuntimeState
from tools.qemu_img_tool import QemuImgError, QemuImgTool
from tools.runtime_config import RuntimeConfig
from tools.secret_rewrap_tool import SecretRewrapError, SecretRewrapTool
from tools.tvgb_transfer_tool import TvgbTransferError, TvgbTransferTool
from tools.virtiofs_tool import VirtioFsError, VirtioFsSession, VirtioFsTool


class HostResilienceServiceError(RuntimeError):
    pass


class HostResilienceService:
    def __init__(
        self,
        config: RuntimeConfig,
        repository: RecoveryRepository | None = None,
        qemu_img: QemuImgTool | None = None,
        host_capability: HostCapabilityTool | None = None,
        network: NetworkRecoveryTool | None = None,
        guest_network: GuestNetworkTool | None = None,
        virtiofs: VirtioFsTool | None = None,
        tvgb: TvgbTransferTool | None = None,
        secret_rewrap: SecretRewrapTool | None = None,
    ) -> None:
        self.config = config
        self.repository = repository or RecoveryRepository(config)
        self.qemu_img = qemu_img or QemuImgTool(config)
        self.host_capability = host_capability or HostCapabilityTool(config)
        self.network = network or NetworkRecoveryTool(config)
        self.guest_network = guest_network or GuestNetworkTool(config)
        self.virtiofs = virtiofs or VirtioFsTool(config)
        self.tvgb = tvgb or TvgbTransferTool(config)
        self.secret_rewrap = secret_rewrap or SecretRewrapTool(config)

    def prepare_image_rescue(
        self,
        *,
        vm_id: str,
        source_image: Path,
        sidecars: dict[str, Path],
        source_host_inventory: dict[str, Any] | None = None,
        guest_profile: dict[str, Any] | None = None,
        host_secret_ref: str | None = None,
    ) -> dict[str, Any]:
        required_roles = set(self.config.runtime["rebind"]["source_inventory_roles"])
        profile = self._validate_guest_profile(guest_profile)
        missing = sorted(role for role in required_roles if role not in sidecars or not sidecars[role].is_file())
        if missing:
            raise HostResilienceServiceError(f"{self.config.codes['sidecar_missing']} missing sidecar roles: {missing}")

        session = self.repository.create_session(vm_id)
        try:
            check = self.qemu_img.check_read_only(source_image)
            overlay = self.qemu_img.create_rescue_overlay(source_image, self.repository.rescue_disk_path(session))
            cloned_sidecars: dict[str, str] = {}
            for role in sorted(required_roles):
                cloned = self.repository.sidecar_path(session, role, sidecars[role])
                cloned_sidecars[role] = str(cloned)

            destination_host = self.host_capability.probe().as_dict()
            rebind = self._build_rebind_plan(source_host_inventory, destination_host, profile)
            rewrap: dict[str, Any] | None = None
            moved_host = bool(source_host_inventory and source_host_inventory.get("fingerprint") != destination_host.get("fingerprint"))
            if host_secret_ref and source_host_inventory is None:
                raise HostResilienceServiceError(f"{self.config.codes['secret_rewrap_required']} source host fingerprint required for host-bound secret")
            if host_secret_ref and moved_host:
                rewrap = self.secret_rewrap.rewrap(
                    {
                        "vm_id": vm_id,
                        "source_ref": host_secret_ref,
                        "source_host_fingerprint": (source_host_inventory or {}).get("fingerprint"),
                        "destination_host_fingerprint": destination_host["fingerprint"],
                    }
                )

            state = {
                "schema": 1,
                "version": "0.14.1",
                "vm_id": vm_id,
                "status": "READY_FOR_BOOT_TEST",
                "source": overlay["source"],
                "source_sha256": overlay["source_sha256"],
                "image_check": check,
                "rescue_overlay": overlay["overlay"],
                "sidecars": cloned_sidecars,
                "destination_host": destination_host,
                "guest_profile": profile,
                "rebind_plan": rebind,
                "secret_rewrap": rewrap,
                "journal": ["INVENTORY", "READ_ONLY_CHECK", "SAFE_OVERLAY", "SIDECAR_CLONE", "REBIND_PLAN"],
            }
            self.repository.write_state(session, state)
            return {"session": str(session), **state}
        except HostResilienceServiceError:
            shutil.rmtree(session, ignore_errors=True)
            raise
        except (QemuImgError, RecoveryRepositoryError, SecretRewrapError) as exc:
            shutil.rmtree(session, ignore_errors=True)
            raise HostResilienceServiceError(str(exc)) from exc

    def recover_network_request(self, request: dict[str, Any]) -> dict[str, Any]:
        nic = dict(request["nic"])
        state = NicRuntimeState(
            netdev_id=str(nic["netdev_id"]),
            device_id=str(nic["device_id"]),
            driver=str(nic["driver"]),
            mac=str(nic["mac"]),
            backend_type=str(nic["backend_type"]),
            backend_arguments=dict(nic.get("backend_arguments", {})),
            device_arguments=dict(nic.get("device_arguments", {})),
        )
        return self.recover_network(
            endpoint=str(request["qmp_endpoint"]),
            state=state,
            platform_name=str(request["platform"]),
            host_capabilities=dict(request.get("host_capabilities", {})),
        )

    def recover_network(
        self,
        *,
        endpoint: str,
        state: NicRuntimeState,
        platform_name: str,
        host_capabilities: dict[str, bool],
    ) -> dict[str, Any]:
        max_attempts = int(self.config.runtime["network"]["max_runtime_attempts"])
        attempts: list[dict[str, Any]] = []

        def verified() -> bool:
            try:
                self.guest_network.verify()
                return True
            except GuestNetworkError:
                return False

        try:
            self.guest_network.soft_renew()
            if verified():
                attempts.append({"mode": "guest_soft_renew", "status": "PASS"})
                return {"status": "PASS", "attempts": attempts}
            attempts.append({"mode": "guest_soft_renew", "status": "FAILED", "error": "verify failed"})
        except GuestNetworkError as exc:
            attempts.append({"mode": "guest_soft_renew", "status": "FAILED", "error": str(exc)})

        if len(attempts) < max_attempts:
            try:
                result = self.network.reconnect_link(endpoint, state)
                if verified():
                    attempts.append({"mode": "same_backend_reconnect", "status": "PASS", "backend": state.backend_type})
                    return {"status": "PASS", "attempts": attempts, "result": result}
                attempts.append({"mode": "same_backend_reconnect", "status": "FAILED", "error": "verify failed"})
            except NetworkRecoveryError as exc:
                attempts.append({"mode": "same_backend_reconnect", "status": "FAILED", "error": str(exc)})

        if len(attempts) < max_attempts:
            try:
                result = self.network.recreate_backend(endpoint, state)
                if verified():
                    attempts.append({"mode": "netdev_recreate_same_mac", "status": "PASS", "backend": state.backend_type})
                    return {"status": "PASS", "attempts": attempts, "result": result}
                attempts.append({"mode": "netdev_recreate_same_mac", "status": "FAILED", "error": "verify failed"})
            except NetworkRecoveryError as exc:
                attempts.append({"mode": "netdev_recreate_same_mac", "status": "FAILED", "error": str(exc)})

        if len(attempts) < max_attempts:
            try:
                rescue_backend = self.network.rescue_backend_for_platform(endpoint, platform_name, host_capabilities)
                result = self.network.recreate_backend(endpoint, state, backend_type=rescue_backend, backend_arguments={})
                if verified():
                    attempts.append({"mode": "rescue_backend", "status": "PASS", "backend": rescue_backend})
                    return {"status": "PASS", "attempts": attempts, "result": result}
                attempts.append({"mode": "rescue_backend", "status": "FAILED", "error": "verify failed"})
            except NetworkRecoveryError as exc:
                attempts.append({"mode": "rescue_backend", "status": "FAILED", "error": str(exc)})

        raise HostResilienceServiceError(f"{self.config.codes['network_attempts_exhausted']} recovery failed: {attempts}")

    def send_drag_drop(self, source: Path, guest_destination: str) -> dict[str, Any]:
        try:
            return self.tvgb.send_file(source, guest_destination)
        except TvgbTransferError as exc:
            raise HostResilienceServiceError(str(exc)) from exc

    def start_shared_folder(
        self,
        *,
        source: Path,
        allow_roots: list[Path],
        guest_os: str,
        guest_capabilities: set[str],
        socket_path: Path,
        tag: str,
    ) -> VirtioFsSession:
        try:
            self.virtiofs.validate_guest_capability(guest_os, guest_capabilities)
            return self.virtiofs.start_read_only(source=source, allow_roots=allow_roots, socket_path=socket_path, tag=tag)
        except VirtioFsError as exc:
            raise HostResilienceServiceError(str(exc)) from exc

    def stop_shared_folder(self, session: VirtioFsSession) -> None:
        self.virtiofs.stop(session)

    def _validate_guest_profile(self, guest_profile: dict[str, Any] | None) -> dict[str, Any]:
        if not isinstance(guest_profile, dict):
            raise HostResilienceServiceError(f"{self.config.codes['hardware_incompatible']} guest profile is required")
        required = set(self.config.runtime["rebind"]["required_guest_profile_fields"])
        missing = sorted(field for field in required if guest_profile.get(field) in (None, ""))
        if missing:
            raise HostResilienceServiceError(f"{self.config.codes['hardware_incompatible']} guest profile fields missing: {missing}")
        return dict(guest_profile)

    def _build_rebind_plan(
        self,
        source_host: dict[str, Any] | None,
        destination_host: dict[str, Any],
        guest_profile: dict[str, Any],
    ) -> dict[str, Any]:
        fields = list(self.config.runtime["rebind"]["hardware_change_fields"])
        changes: dict[str, dict[str, Any]] = {}
        if source_host:
            for field in fields:
                before = source_host.get(field)
                after = destination_host.get(field)
                if before != after:
                    changes[field] = {"from": before, "to": after}
        guest_arch = str(guest_profile["architecture"]).lower()
        destination_machine = str(destination_host.get("machine", "")).lower()
        compatible = guest_arch in destination_machine or destination_machine in guest_arch
        common_x86 = {"x86_64", "amd64"}
        if guest_arch in common_x86 and destination_machine in common_x86:
            compatible = True
        if not compatible:
            raise HostResilienceServiceError(f"{self.config.codes['hardware_incompatible']} guest architecture cannot be rebound automatically")
        return {
            "hardware_changes": changes,
            "firmware_mode": guest_profile["firmware_mode"],
            "disk_bus": guest_profile["disk_bus"],
            "machine_profile": guest_profile["machine_profile"],
            "guest_identity": guest_profile["guest_identity"],
            "fingerprint_boot_lock": False,
            "preserve_guest_identity": bool(self.config.policy["image_recovery"]["preserve_guest_identity"]),
            "preserve_firmware_mode": bool(self.config.policy["image_recovery"]["preserve_firmware_mode"]),
            "preserve_disk_bus": bool(self.config.policy["image_recovery"]["preserve_disk_bus_when_known"]),
            "secret_rewrap_required": bool(self.config.runtime["rebind"]["destination_secret_rewrap_required"]),
            "generated_unix_time": int(time.time()),
        }
