# 📄 Dosya Yolu: /turkuazvm/views/host_resilience_view.py
# 📌 Amac: Host resilience Service sonuclarini UI/API icin karar vermeyen sade response yapisina donusturur
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: View sadece status, code, data ve message alanlarini formatlar; runtime is kurali calistirmaz
# Bagimli Oldugu Katman: View | Language

from __future__ import annotations

from typing import Any


class HostResilienceView:
    @staticmethod
    def success(code: str, data: dict[str, Any]) -> dict[str, Any]:
        return {"status": "PASS", "code": code, "data": data}

    @staticmethod
    def blocked(code: str, message: str) -> dict[str, Any]:
        return {"status": "BLOCKED", "code": code, "message": message}
