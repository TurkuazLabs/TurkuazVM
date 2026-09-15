# 📄 Dosya Yolu: /turkuazvm/scripts/verify_recovered_full.ps1
# 📌 Amac: Windows hostta TurkuazVM FULL recovery source ve Cargo gate'ini tek komutla calistirir
# 📌 Modul - PowerShell
# Version: 0.28.0
# Aciklama: Guncel verify adimlarini, Python recovery gate'ini ve Cargo check/test'i sirali calistirir
# Bagimli Oldugu Katman: Tool | View

$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path

& (Join-Path $Root 'scripts\verify.ps1')

$Python = Get-Command python -ErrorAction SilentlyContinue
if (-not $Python) {
    $Python = Get-Command py -ErrorAction SilentlyContinue
}
if (-not $Python) {
    throw 'Python bulunamadi. FULL recovery gate calistirilamadi.'
}

if ($Python.Name -eq 'py.exe' -or $Python.Name -eq 'py') {
    & $Python.Source -3 (Join-Path $Root 'scripts\full_recovery_gate.py') --cargo
} else {
    & $Python.Source (Join-Path $Root 'scripts\full_recovery_gate.py') --cargo
}
if ($LASTEXITCODE -ne 0) {
    throw "FULL recovery gate basarisiz. exit=$LASTEXITCODE"
}

Write-Host 'TURKUAZVM_FULL_RECOVERY_VERIFY=PASS'
