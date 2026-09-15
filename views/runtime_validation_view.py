# 📄 Dosya Yolu: /turkuazvm/views/runtime_validation_view.py
# 📌 Amac: L runtime validation sonucunu UI veya CLI icin salt-okunur ozet modele donusturur
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Test is kurali calistirmadan smoke ve production gate sonucunu gorunur hale getirir
# Bagimli Oldugu Katman: View | Language

from __future__ import annotations

from typing import Any


class RuntimeValidationView:
    def render(self, result: dict[str, Any]) -> dict[str, Any]:
        return {
            "session_id": result.get("session_id"),
            "mode": result.get("mode"),
            "smoke_gate": result.get("smoke_gate"),
            "production_gate": result.get("production_gate"),
            "requested_gate": result.get("requested_gate"),
            "summary": result.get("summary"),
        }
