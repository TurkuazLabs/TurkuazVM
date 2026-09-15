# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0402_native_display_telemetry_contract.py
# 📌 Amac: Native display FPS yaniltmasini kapatan telemetri ayrimini ve CPU scaling optimizasyonunu regression olarak dogrular
# 📌 Modul - Python
# Version: 0.40.2
# Aciklama: RFB UPS, present FPS, gercek render duration, drop/throughput sayaçlari ve cached scale-map kontratini fail-closed kilitler
# Bagimli Oldugu Katman: Tool | View

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> None:
    clock = read("apps/display/src/tools/frame_clock_tool.rs")
    rfb = read("apps/display/src/tools/rfb_client_tool.rs")
    view = read("apps/display/src/views/native_display_view.rs")
    scale = read("apps/display/src/tools/scale_map_tool.rs")
    modules = read("apps/display/src/tools/mod.rs")

    require("1000.0 / fps" not in clock, "DERIVED_FRAMETIME_BUG_RETURNED")
    require("record_present(&mut self, render_duration: Duration)" in clock, "REAL_RENDER_DURATION_MISSING")
    require("rfb_updates_per_second" in clock, "RFB_UPS_TELEMETRY_MISSING")
    require("rfb_megabits_per_second" in clock, "RFB_THROUGHPUT_TELEMETRY_MISSING")
    require("dropped_frames" in clock, "FRAME_DROP_TELEMETRY_MISSING")

    for token in (
        "AtomicU64",
        "framebuffer_updates",
        "rectangles",
        "raw_bytes",
        "frames_enqueued",
        "frames_dropped",
        "telemetry_snapshot",
    ):
        require(token in rfb, f"RFB_TELEMETRY_TOKEN_MISSING:{token}")

    require("pub mod scale_map_tool;" in modules, "SCALE_MAP_MODULE_MISSING")
    require("build_axis_map" in scale, "CACHED_AXIS_MAP_BUILDER_MISSING")
    require("self.scale_map" in view, "VIEW_SCALE_MAP_USAGE_MISSING")
    require("buffer.copy_from_slice(&self.frame.pixels)" in view, "ONE_TO_ONE_COPY_FAST_PATH_MISSING")
    require("self.render_target_size != Some((size.width, size.height))" in view, "SURFACE_RESIZE_CACHE_MISSING")
    require("render_started.elapsed()" in view, "REAL_RENDER_MEASUREMENT_MISSING")
    require("RFB {:.1} UPS" in view, "TITLE_RFB_UPS_LABEL_MISSING")
    require("Present {:.1} FPS" in view, "TITLE_PRESENT_FPS_LABEL_MISSING")
    require("Render {:.2} ms" in view, "TITLE_RENDER_MS_LABEL_MISSING")
    require("Drop {}" in view, "TITLE_DROP_LABEL_MISSING")
    require("FPS / {:.2} ms" not in view, "LEGACY_MISLEADING_TITLE_RETURNED")

    print("V0402_NATIVE_DISPLAY_TELEMETRY_CONTRACT=PASS")


if __name__ == "__main__":
    main()
