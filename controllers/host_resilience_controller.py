# 📄 Dosya Yolu: /turkuazvm/controllers/host_resilience_controller.py
# 📌 Amac: Host resilience UI/API requestlerini parse edip yalnizca HostResilienceService metodlarina delege eder
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Controller icinde recovery, filesystem, QMP veya capability is kurali bulundurmaz
# Bagimli Oldugu Katman: Controller | Service | View

from __future__ import annotations

from pathlib import Path
from typing import Any

from services.host_resilience_service import HostResilienceService


class HostResilienceController:
    def __init__(self, service: HostResilienceService) -> None:
        self.service = service

    def prepare_image_rescue(self, request: dict[str, Any]) -> dict[str, Any]:
        sidecars = {str(role): Path(str(path)) for role, path in dict(request["sidecars"]).items()}
        return self.service.prepare_image_rescue(
            vm_id=str(request["vm_id"]),
            source_image=Path(str(request["source_image"])),
            sidecars=sidecars,
            source_host_inventory=request.get("source_host_inventory"),
            guest_profile=request.get("guest_profile"),
            host_secret_ref=request.get("host_secret_ref"),
        )

    def recover_network(self, request: dict[str, Any]) -> dict[str, Any]:
        return self.service.recover_network_request(request)

    def drag_drop(self, request: dict[str, Any]) -> dict[str, Any]:
        return self.service.send_drag_drop(Path(str(request["source"])), str(request["guest_destination"]))

    def start_shared_folder(self, request: dict[str, Any]) -> dict[str, Any]:
        session = self.service.start_shared_folder(
            source=Path(str(request["source"])),
            allow_roots=[Path(str(item)) for item in request["allow_roots"]],
            guest_os=str(request["guest_os"]),
            guest_capabilities=set(request["guest_capabilities"]),
            socket_path=Path(str(request["socket_path"])),
            tag=str(request["tag"]),
        )
        return {
            "tag": session.tag,
            "source": str(session.source),
            "socket_path": str(session.socket_path),
            "read_only": session.read_only,
        }
