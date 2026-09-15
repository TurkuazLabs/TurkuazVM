# 📄 Dosya Yolu: /turkuazvm/scripts/verify.ps1
# 📌 Amac: TurkuazVM workspace icin format, check, clippy ve unit test dogrulamasini calistirir
# 📌 Modul - PowerShell
# Version: 0.28.0
# Aciklama: Developer makinesinde release oncesi Rust kalite kapilarini tek komutta uygular
# Bagimli Oldugu Katman: Tool

$ErrorActionPreference = "Stop"

Write-Host "TurkuazVM Verify v0.28.0"
Write-Host ""

& (Join-Path $PSScriptRoot "verify_structure.ps1")

cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets
cargo test --workspace

Write-Host ""
Write-Host "VERIFY_OK"
