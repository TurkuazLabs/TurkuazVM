# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_v0404_launcher_softbuffer_contract.py
# 📌 Amac: Launcher structure gate'in optimize native display rendererini stale kaynak tokenlariyla engellemesini onler
# 📌 Modul - Python
# Version: 0.40.4
# Aciklama: Softbuffer verifier'in yerel degisken adina baglanmak yerine semantik render patternlerini kullandigini regression olarak kilitler
# Bagimli Oldugu Katman: Tool | View

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> None:
    verifier = read("scripts/verify_structure.ps1")
    view = read("apps/display/src/views/native_display_view.rs")
    launcher = read("scripts/start_turkuazvm.ps1")

    require('buffer[host_row + x]' not in verifier, "STALE_HOST_X_CONTRACT_RETURNED")
    require('0x00FF_FFFF' not in verifier, "STALE_PIXEL_MASK_CONTRACT_RETURNED")
    require('$NativeDisplayContracts = @(' in verifier, "SEMANTIC_DISPLAY_CONTRACT_SET_MISSING")

    for token in (
        'mutable_surface_buffer',
        'one_to_one_copy_fast_path',
        'cached_scale_map',
        'scaled_pixel_write',
        'softbuffer_present',
        '[regex]::IsMatch($NativeDisplayContent, $Contract.Pattern)',
    ):
        require(token in verifier, f"SEMANTIC_DISPLAY_CONTRACT_MISSING:{token}")

    require('buffer.copy_from_slice(&self.frame.pixels)' in view, "DISPLAY_COPY_FAST_PATH_MISSING")
    require('buffer[host_row + host_x] = self.frame.pixels[guest_row + source_x]' in view, "DISPLAY_SCALED_WRITE_MISSING")
    require('self.scale_map' in view, "DISPLAY_SCALE_MAP_MISSING")
    require('buffer.present()' in view, "DISPLAY_PRESENT_MISSING")
    require('$Verifier = Join-Path $PSScriptRoot "verify_structure.ps1"' in launcher, "LAUNCHER_STRUCTURE_GATE_MISSING")

    print("V0404_LAUNCHER_SOFTBUFFER_CONTRACT=PASS")


if __name__ == "__main__":
    main()
