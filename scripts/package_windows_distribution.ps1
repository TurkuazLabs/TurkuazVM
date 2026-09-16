# 📄 Dosya Yolu: /turkuazvm/scripts/package_windows_distribution.ps1
# 📌 Amac: TurkuazVM Windows NSIS installer ve portable ZIP dagitim paketlerini uretir
# 📌 Modul - PowerShell Tool
# Version: 0.41.4
# Aciklama: Engine/Display release binarylerini stage eder, runtime config yollarini dagitim icin duzeltir, Tauri NSIS ve portable paket olusturur
# Bagimli Oldugu Katman: Tool | CI/CD | View

[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$CargoToml = Join-Path $Root "Cargo.toml"
$DesktopRoot = Join-Path $Root "apps/desktop"
$TauriRoot = Join-Path $DesktopRoot "src-tauri"
$StageRoot = Join-Path $TauriRoot "distribution/windows/stage"
$TargetRelease = Join-Path $Root "target/release"
$OutputRoot = Join-Path $Root "artifacts/distribution/windows"

function Invoke-NativeChecked {
    param(
        [Parameter(Mandatory = $true)][string]$FilePath,
        [Parameter(Mandatory = $true)][string[]]$Arguments,
        [string]$WorkingDirectory = $Root
    )

    Push-Location $WorkingDirectory
    try {
        & $FilePath @Arguments
        if ($LASTEXITCODE -ne 0) {
            throw "Native command failed ($LASTEXITCODE): $FilePath $($Arguments -join ' ')"
        }
    }
    finally {
        Pop-Location
    }
}

function Get-WorkspaceVersion {
    $InWorkspacePackage = $false
    foreach ($Line in Get-Content -LiteralPath $CargoToml) {
        if ($Line -match '^\s*\[workspace\.package\]\s*$') {
            $InWorkspacePackage = $true
            continue
        }
        if ($InWorkspacePackage -and $Line -match '^\s*\[') {
            break
        }
        if ($InWorkspacePackage -and $Line -match '^\s*version\s*=\s*"([^"]+)"\s*$') {
            return $Matches[1]
        }
    }
    throw "WORKSPACE_VERSION_NOT_FOUND"
}

function Reset-Directory {
    param([Parameter(Mandatory = $true)][string]$Path)
    if (Test-Path -LiteralPath $Path) {
        Remove-Item -LiteralPath $Path -Recurse -Force
    }
    New-Item -ItemType Directory -Path $Path -Force | Out-Null
}

$Version = Get-WorkspaceVersion
Write-Host "TurkuazVM Windows distribution build v$Version"

Reset-Directory -Path $StageRoot
Reset-Directory -Path $OutputRoot

Write-Host "[1/6] Engine ve Display release binaryleri derleniyor..."
Invoke-NativeChecked -FilePath "cargo" -Arguments @(
    "build", "--release",
    "-p", "turkuazvm-engine",
    "-p", "turkuazvm-display"
)

$EngineExe = Join-Path $TargetRelease "turkuazvm-engine.exe"
$DisplayExe = Join-Path $TargetRelease "turkuazvm-display.exe"
foreach ($Required in @($EngineExe, $DisplayExe)) {
    if (-not (Test-Path -LiteralPath $Required -PathType Leaf)) {
        throw "REQUIRED_RELEASE_BINARY_NOT_FOUND: $Required"
    }
}

Write-Host "[2/6] Runtime stage hazirlaniyor..."
$StageBin = Join-Path $StageRoot "bin"
$StageConfig = Join-Path $StageRoot "config"
$StageScripts = Join-Path $StageRoot "scripts"
New-Item -ItemType Directory -Path $StageBin, $StageConfig, $StageScripts -Force | Out-Null

Copy-Item -LiteralPath $EngineExe -Destination (Join-Path $StageBin "turkuazvm-engine.exe")
Copy-Item -LiteralPath $DisplayExe -Destination (Join-Path $StageBin "turkuazvm-display.exe")
Copy-Item -Path (Join-Path $Root "config/*") -Destination $StageConfig -Recurse -Force
Copy-Item -LiteralPath (Join-Path $Root "scripts/network_windows_managed.ps1") -Destination $StageScripts -Force

$GuestAndroidScripts = Join-Path $Root "guest/android-image/scripts"
if (Test-Path -LiteralPath $GuestAndroidScripts -PathType Container) {
    $StageGuestScripts = Join-Path $StageRoot "guest/android-image/scripts"
    New-Item -ItemType Directory -Path $StageGuestScripts -Force | Out-Null
    Copy-Item -Path (Join-Path $GuestAndroidScripts "*") -Destination $StageGuestScripts -Recurse -Force
}

$RuntimeConfigPath = Join-Path $StageConfig "turkuazvm.yml"
$RuntimeConfig = Get-Content -LiteralPath $RuntimeConfigPath -Raw
$RuntimeConfig = $RuntimeConfig.Replace(
    "display_executable_path: target/debug/turkuazvm-display",
    "display_executable_path: bin/turkuazvm-display.exe"
)
$RuntimeConfig = $RuntimeConfig.Replace(
    "executable_path: target/debug/turkuazvm-engine",
    "executable_path: bin/turkuazvm-engine.exe"
)
Set-Content -LiteralPath $RuntimeConfigPath -Value $RuntimeConfig -Encoding utf8

if ((Get-Content -LiteralPath $RuntimeConfigPath -Raw) -match 'target/debug/turkuazvm-(engine|display)') {
    throw "DISTRIBUTION_CONFIG_STILL_REFERENCES_DEBUG_BINARY"
}

Write-Host "[3/6] Tauri NSIS installer derleniyor..."
Invoke-NativeChecked -FilePath "cargo" -Arguments @(
    "tauri", "build",
    "--bundles", "nsis",
    "--config", "src-tauri/tauri.windows.conf.json5"
) -WorkingDirectory $DesktopRoot

$DesktopExe = Join-Path $TargetRelease "turkuazvm-desktop.exe"
if (-not (Test-Path -LiteralPath $DesktopExe -PathType Leaf)) {
    throw "DESKTOP_RELEASE_BINARY_NOT_FOUND: $DesktopExe"
}

$NsisRoot = Join-Path $TargetRelease "bundle/nsis"
$SetupSource = Get-ChildItem -LiteralPath $NsisRoot -Filter "*.exe" -File -ErrorAction Stop |
    Where-Object { $_.Name -match 'setup' } |
    Sort-Object LastWriteTimeUtc -Descending |
    Select-Object -First 1
if ($null -eq $SetupSource) {
    throw "NSIS_SETUP_NOT_FOUND: $NsisRoot"
}

$SetupName = "TurkuazVM-$Version-x64-Setup.exe"
$SetupPath = Join-Path $OutputRoot $SetupName
Copy-Item -LiteralPath $SetupSource.FullName -Destination $SetupPath -Force

Write-Host "[4/6] Portable paket olusturuluyor..."
$PortableStage = Join-Path $OutputRoot "portable-stage"
Reset-Directory -Path $PortableStage
Copy-Item -LiteralPath $DesktopExe -Destination (Join-Path $PortableStage "TurkuazVM.exe")
Copy-Item -Path (Join-Path $StageRoot "*") -Destination $PortableStage -Recurse -Force

$PortableReadme = @"
TurkuazVM v$Version Portable

1. Bu klasoru yazma izniniz olan bir konuma cikartin.
2. TurkuazVM.exe dosyasini bu klasor icinden calistirin.
3. QEMU sistemde kurulu/erisilebilir olmalidir; TurkuazVM dependency katmani bunu ayrica yonetir.
4. data ve packages dizinleri ilk kullanimda bu portable kok altinda olusabilir.
"@
Set-Content -LiteralPath (Join-Path $PortableStage "README-PORTABLE.txt") -Value $PortableReadme -Encoding utf8

$PortableName = "TurkuazVM-$Version-x64-Portable.zip"
$PortablePath = Join-Path $OutputRoot $PortableName
Compress-Archive -Path (Join-Path $PortableStage "*") -DestinationPath $PortablePath -CompressionLevel Optimal -Force
Remove-Item -LiteralPath $PortableStage -Recurse -Force

Write-Host "[5/6] SHA-256 manifest olusturuluyor..."
$HashName = "TurkuazVM-$Version-SHA256SUMS.txt"
$HashPath = Join-Path $OutputRoot $HashName
$HashLines = foreach ($Artifact in @($SetupPath, $PortablePath)) {
    $Hash = Get-FileHash -LiteralPath $Artifact -Algorithm SHA256
    "{0}  {1}" -f $Hash.Hash.ToLowerInvariant(), (Split-Path -Leaf $Artifact)
}
Set-Content -LiteralPath $HashPath -Value $HashLines -Encoding ascii

Write-Host "[6/6] Dagitim paketi hazir."
Get-ChildItem -LiteralPath $OutputRoot -File | ForEach-Object {
    Write-Host ("  {0} ({1:N0} bytes)" -f $_.Name, $_.Length)
}

Write-Output "WINDOWS_DISTRIBUTION_VERSION=$Version"
Write-Output "WINDOWS_DISTRIBUTION_SETUP=$SetupPath"
Write-Output "WINDOWS_DISTRIBUTION_PORTABLE=$PortablePath"
Write-Output "WINDOWS_DISTRIBUTION_HASHES=$HashPath"
