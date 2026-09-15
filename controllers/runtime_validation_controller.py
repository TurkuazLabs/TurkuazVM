# 📄 Dosya Yolu: /turkuazvm/controllers/runtime_validation_controller.py
# 📌 Amac: L runtime validation requestini parse eder ve yalnizca Service katmanina aktarir
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Controller icinde test, QEMU, filesystem veya evidence is kurali bulunmaz
# Bagimli Oldugu Katman: Controller | Service

from __future__ import annotations

from typing import Any

from services.runtime_validation_service import RuntimeValidationService


class RuntimeValidationController:
    def __init__(self, service: RuntimeValidationService) -> None:
        self.service = service

    def run(self, request: dict[str, Any]) -> dict[str, Any]:
        session_id = str(request.get("session_id", "")).strip()
        mode = str(request.get("mode", "")).strip()
        return self.service.run(session_id=session_id, mode=mode)
