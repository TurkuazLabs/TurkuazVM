# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_engine_supervision_contract.py
# 📌 Amac: Desktop local Engine supervision, loglama ve yeniden baslatma kontratini statik olarak dogrular
# 📌 Modul - Python
# Version: 0.39.5
# Aciklama: Engine process handle, Ping readiness, stdout/stderr loglama, exit diagnostics ve startup timeout ayarlarini regression seviyesinde korur
# Bagimli Oldugu Katman: Tool

from pathlib import Path

import yaml


PROJECT_ROOT = Path(__file__).resolve().parents[2]
PROCESS_TOOL = PROJECT_ROOT / "apps/desktop/src-tauri/src/tools/engine_process_tool.rs"
DESKTOP_SERVICE = PROJECT_ROOT / "apps/desktop/src-tauri/src/services/desktop_service.rs"
MAIN_CONFIG = PROJECT_ROOT / "config/turkuazvm.yml"


def require_tokens(content: str, tokens: tuple[str, ...], label: str) -> None:
    missing = [token for token in tokens if token not in content]
    if missing:
        raise SystemExit(f"{label}_MISSING: {', '.join(missing)}")


def main() -> None:
    process_tool = PROCESS_TOOL.read_text(encoding="utf-8")
    desktop_service = DESKTOP_SERVICE.read_text(encoding="utf-8")
    config = yaml.safe_load(MAIN_CONFIG.read_text(encoding="utf-8"))

    require_tokens(
        process_tool,
        (
            "EngineProcessHandle",
            "data/logs/engine",
            "diagnostic_tail",
            ".stdout(Stdio::from(stdout_file))",
            ".stderr(Stdio::from(log_file))",
            "try_wait",
        ),
        "ENGINE_PROCESS_SUPERVISION",
    )
    require_tokens(
        desktop_service,
        (
            "engine_processes",
            "spawn_local_engine",
            "wait_for_engine_ready",
            "send_after_readiness",
            "action: EngineAction::Ping",
            "take_engine_exit_detail",
            "engine_not_ready_message",
        ),
        "DESKTOP_ENGINE_SUPERVISION",
    )

    local_host = next(
        host
        for host in config["desktop"]["hosts"]
        if host["id"] == config["desktop"]["active_host"]
    )
    if not local_host["auto_start"]:
        raise SystemExit("LOCAL_ENGINE_AUTO_START_DISABLED")
    if int(local_host["startup_timeout_ms"]) < 10000:
        raise SystemExit("LOCAL_ENGINE_STARTUP_TIMEOUT_TOO_LOW")

    print("ENGINE_SUPERVISION_CONTRACT_OK")


if __name__ == "__main__":
    main()
