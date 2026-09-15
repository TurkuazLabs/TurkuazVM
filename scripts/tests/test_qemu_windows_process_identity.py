# 📄 Dosya Yolu: /turkuazvm/scripts/tests/test_qemu_windows_process_identity.py
# 📌 Amac: Windows QEMU process identity PowerShell PID aktarim kontratini regression olarak dogrular
# 📌 Modul - Python
# Version: 0.28.0
# Aciklama: PID'nin -Command sonrasinda ayri token olarak verilmesini ve $args tabanli eski parser bug'ini engeller
# Bagimli Oldugu Katman: Tool

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "crates" / "qemu" / "src" / "tools" / "qemu_runtime_tool.rs"


def main() -> None:
    text = SOURCE.read_text(encoding="utf-8")
    assert "$args[0]" not in text
    assert "PROCESS_QUERY_POWERSHELL_PREFIX" in text
    assert "PROCESS_QUERY_POWERSHELL_SUFFIX" in text
    assert "{process_id}" in text
    assert '.args(["-NoProfile", "-NonInteractive", "-Command", command.as_str()])' in text
    start = text.index('#[cfg(target_os = \"windows\")]\nfn process_identity')
    end = text.index('#[cfg(not(any(target_os = \"linux\", target_os = \"windows\")))]', start)
    process_identity_block = text[start:end]
    assert 'process_id_text.as_str()' not in process_identity_block
    print("QEMU_WINDOWS_PROCESS_IDENTITY_GATE=PASS")


if __name__ == "__main__":
    main()
