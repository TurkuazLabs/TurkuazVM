# 📄 Dosya Yolu: /turkuazvm/scripts/run_desktop.ps1
# 📌 Amac: TurkuazVM Engine ve Desktop development akisini tek komutla baslatir
# 📌 Modul - PowerShell
# Version: 0.32.0
# Aciklama: Engine ve TurkuazDisplay binary'lerini build eder ve portable QEMU Turkuaz NAT destekli Tauri Desktop crate'ini Cargo uzerinden calistirir
# Bagimli Oldugu Katman: Tool | View

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$ConfigPath = Join-Path $ProjectRoot "config/turkuazvm.yml"

Push-Location $ProjectRoot
try {
    $env:TURKUAZVM_CONFIG = $ConfigPath
    cargo build -p turkuazvm-engine -p turkuazvm-display
    cargo run -p turkuazvm-desktop
}
finally {
    Pop-Location
}
