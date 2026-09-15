# 📄 Dosya Yolu: C:\TurkuazVM-GitHub-Publish.ps1
# 📌 Amac: TurkuazVM kaynak agacini temiz bir GitHub worktree uzerinden TurkuazLabs/TurkuazVM main branchine aktarir
# 📌 Modul - PowerShell
# Version: 1.0.0
# Aciklama: Lokal kaynaklari clone edilen repoya kopyalar, uretilen/runtime dosyalarini haric tutar, commit eder ve push eder
# Bagimli Oldugu Katman: Tool

[CmdletBinding()]
param(
    [string]$SourceRoot = "",
    [string]$RepositoryUrl = "https://github.com/TurkuazLabs/TurkuazVM.git",
    [string]$Branch = "main"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Write-Step {
    param([Parameter(Mandatory = $true)][string]$Message)

    Write-Host ""
    Write-Host "==> $Message" -ForegroundColor Cyan
}

function Invoke-Git {
    param([Parameter(Mandatory = $true)][string[]]$Arguments)

    & git @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "Git komutu basarisiz: git $($Arguments -join ' ')"
    }
}

if ([string]::IsNullOrWhiteSpace($SourceRoot)) {
    $SourceRoot = Read-Host "TurkuazVM kaynak klasorunun tam yolunu girin"
}

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    throw "Git bulunamadi. Once Git for Windows kurulmali ve git PATH icinde olmali."
}

if (-not (Test-Path -LiteralPath $SourceRoot -PathType Container)) {
    throw "Kaynak klasoru bulunamadi: $SourceRoot"
}

$SourceRoot = (Resolve-Path -LiteralPath $SourceRoot).Path

$RequiredFiles = @(
    "Cargo.toml",
    ".gitignore",
    "README.md",
    "TurkuazVM-Start.cmd"
)

foreach ($RequiredFile in $RequiredFiles) {
    $RequiredPath = Join-Path $SourceRoot $RequiredFile
    if (-not (Test-Path -LiteralPath $RequiredPath -PathType Leaf)) {
        throw "Kaynak klasoru TurkuazVM root gibi gorunmuyor. Eksik dosya: $RequiredFile"
    }
}

$WorkRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("TurkuazVM-GitHub-Publish-" + [guid]::NewGuid().ToString("N"))
$InsideWorktree = $false

try {
    Write-Step "GitHub repository clone ediliyor"
    Invoke-Git -Arguments @(
        "clone",
        "--branch", $Branch,
        "--single-branch",
        $RepositoryUrl,
        $WorkRoot
    )

    Write-Step "Kaynak agaci temiz worktree icine kopyalaniyor"

    $RobocopyArguments = @(
        $SourceRoot,
        $WorkRoot,
        "/E",
        "/COPY:DAT",
        "/DCOPY:DAT",
        "/R:2",
        "/W:1",
        "/NFL",
        "/NDL",
        "/NJH",
        "/NJS",
        "/NP",
        "/XD",
        ".git",
        "target",
        "artifacts",
        "runtime",
        "data",
        "secrets",
        ".turkuazvm",
        ".idea",
        ".vscode",
        "__pycache__",
        "/XF",
        "*.log",
        "*.pyc",
        "*.apk"
    )

    & robocopy @RobocopyArguments | Out-Null
    $RobocopyExitCode = $LASTEXITCODE

    if ($RobocopyExitCode -ge 8) {
        throw "Robocopy basarisiz. Exit code: $RobocopyExitCode"
    }

    Push-Location $WorkRoot
    $InsideWorktree = $true

    # Release paketindeki dosya byte'larini koru; lokal Git ayari global autocrlf davranisini ezsin.
    Invoke-Git -Arguments @("config", "core.autocrlf", "false")

    Write-Step "Git degisiklikleri kontrol ediliyor"
    $Status = (& git status --porcelain) -join "`n"
    if ($LASTEXITCODE -ne 0) {
        throw "Git status komutu basarisiz."
    }

    if ([string]::IsNullOrWhiteSpace($Status)) {
        Write-Host "Repository zaten kaynak agaci ile ayni. Push gerekmiyor." -ForegroundColor Green
        return
    }

    $UserName = (& git config user.name 2>$null)
    $UserEmail = (& git config user.email 2>$null)

    if ([string]::IsNullOrWhiteSpace(($UserName -join ""))) {
        Invoke-Git -Arguments @("config", "user.name", "b1glord")
    }

    if ([string]::IsNullOrWhiteSpace(($UserEmail -join ""))) {
        Invoke-Git -Arguments @("config", "user.email", "b1glord@users.noreply.github.com")
    }

    Write-Step "Dosyalar commit icin hazirlaniyor"
    Invoke-Git -Arguments @("add", "-A")

    Write-Step "Ilk TurkuazVM kaynak commit'i olusturuluyor"
    Invoke-Git -Arguments @(
        "commit",
        "-m",
        "chore: bootstrap TurkuazVM v0.41.2 source"
    )

    Write-Step "main branch GitHub'a gonderiliyor"
    Invoke-Git -Arguments @(
        "push",
        "origin",
        "HEAD:$Branch"
    )

    Write-Host ""
    Write-Host "BASARILI - TurkuazVM v0.41.2 kaynak agaci GitHub'a tasindi." -ForegroundColor Green
    Write-Host "Repository: https://github.com/TurkuazLabs/TurkuazVM"
    Write-Host "Branch    : $Branch"
    Write-Host ""
    Write-Host "Not: Push tamamlaninca GitHub Actions otomatik baslayabilir."
}
finally {
    if ($InsideWorktree) {
        Pop-Location
    }

    if (Test-Path -LiteralPath $WorkRoot) {
        Remove-Item -LiteralPath $WorkRoot -Recurse -Force -ErrorAction SilentlyContinue
    }
}
