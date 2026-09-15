# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0410_engine_start_crash_diagnostics_contract.py
# 📌 Amac: Engine StartVm panic ve local process exitlerinin generic TCP 10054 hatasi olarak kaybolmasini engeller
# 📌 Modul - Python
# Version: 0.40.10
# Aciklama: Engine request panic containment ve Desktop process-tail diagnostik kontratini fail-closed dogrular
# Bagimli Oldugu Katman: Controller | Service | Tool | View

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> None:
    engine_main = read("apps/engine/src/main.rs")
    desktop_service = read("apps/desktop/src-tauri/src/services/desktop_service.rs")

    require("catch_unwind" in engine_main, "ENGINE_REQUEST_PANIC_GUARD_MISSING")
    require("AssertUnwindSafe" in engine_main, "ENGINE_REQUEST_UNWIND_GUARD_MISSING")
    require("engine_request_panic" in engine_main, "ENGINE_REQUEST_PANIC_RESPONSE_MISSING")
    require("panic_detail" in engine_main, "ENGINE_REQUEST_PANIC_DETAIL_MISSING")
    require(
        "take_engine_exit_detail(&profile.id)" in desktop_service,
        "DESKTOP_ENGINE_EXIT_DIAGNOSTIC_MISSING",
    )
    require(
        "Engine API baglantisi request sonrasinda kapandi ve local Engine processi sonlandi" in desktop_service,
        "DESKTOP_ENGINE_EXIT_MESSAGE_MISSING",
    )

    print("V0410_ENGINE_START_CRASH_DIAGNOSTICS_CONTRACT=PASS")


if __name__ == "__main__":
    main()
