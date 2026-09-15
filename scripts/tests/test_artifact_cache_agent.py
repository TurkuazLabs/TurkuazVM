# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_artifact_cache_agent.py
# 📌 Amac: Artifact Cache ve signed Guest Agent gate regression testini calistirir
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: Gate importu ile tum cache, schema, signature ve rollback invariantlarini test eder
# Bagimli Oldugu Katman: Tool | View

from __future__ import annotations

import importlib.util
from pathlib import Path


def load_gate():
    root = Path(__file__).resolve().parents[2]
    path = root / "scripts" / "artifact_cache_agent_gate.py"
    spec = importlib.util.spec_from_file_location("artifact_cache_agent_gate", path)
    if spec is None or spec.loader is None:
        raise RuntimeError("gate module could not be loaded")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module, root


def main() -> int:
    gate, root = load_gate()
    errors = gate.validate(root)
    if errors:
        print("ARTIFACT_CACHE_AGENT_REGRESSION=BLOCKED")
        for error in errors:
            print(f"- {error}")
        return 2
    print("ARTIFACT_CACHE_AGENT_REGRESSION=PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
