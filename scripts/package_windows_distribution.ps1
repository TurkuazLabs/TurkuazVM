# 📄 Dosya Yolu: /turkuazvm/scripts/package_windows_distribution.ps1
# 📌 Amac: TurkuazVM Windows NSIS installer ve portable ZIP dagitim paketlerini uretir
# 📌 Modul - PowerShell Tool
# Version: 0.41.6
# Aciklama: Tauri 2.11.4 NSIS template'ini blob SHA ile dogrulayip TurkuazLabs/TurkuazVM install-root patchini uygular; Engine/Display stage, NSIS ve portable paketleri uretir
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
$GeneratedTauriConfig = Join-Path $TauriRoot "tauri.distribution.generated.conf.json5"
$GeneratedNsisTemplate = Join-Path $TauriRoot "windows/installer.generated.nsi"
$TargetRelease = Join-Path $Root "target/release"
$OutputRoot = Join-Path $Root "artifacts/distribution/windows"

$TauriCliVersion = "2.11.4"
$TauriNsisTemplateUrl = "https://raw.githubusercontent.com/tauri-apps/tauri/tauri-cli-v$TauriCliVersion/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi"
$TauriNsisTemplateBlobSha = "d372e3c391770cf231db974422a1e4f8adaac3a6"
$BrandInstallSubdirectory = "TurkuazLabs\TurkuazVM"

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

function Get-GitBlobSha1 {
    param([Parameter(Mandatory = $true)][string]$Path)

    $Bytes = [System.IO.File]::ReadAllBytes($Path)
    $Header = [System.Text.Encoding]::ASCII.GetBytes("blob $($Bytes.Length)`0")
    $Payload = New-Object byte[] ($Header.Length + $Bytes.Length)
    [System.Buffer]::BlockCopy($Header, 0, $Payload, 0, $Header.Length)
    [System.Buffer]::BlockCopy($Bytes, 0, $Payload, $Header.Length, $Bytes.Length)

    $Sha1 = [System.Security.Cryptography.SHA1]::Create()
    try {
        return -join ($Sha1.ComputeHash($Payload) | ForEach-Object { $_.ToString("x2") })
    }
    finally {
        $Sha1.Dispose()
    }
}

function Replace-RequiredLiteral {
    param(
        [Parameter(Mandatory = $true)][string]$Content,
        [Parameter(Mandatory = $true)][string]$OldValue,
        [Parameter(Mandatory = $true)][string]$NewValue,
        [Parameter(Mandatory = $true)][int]$ExpectedCount
    )

    $Count = 0
    $Offset = 0
    while ($true) {
        $Index = $Content.IndexOf($OldValue, $Offset, [System.StringComparison]::Ordinal)
        if ($Index -lt 0) {
            break
        }
        $Count++
        $Offset = $Index + $OldValue.Length
    }

    if ($Count -ne $ExpectedCount) {
        throw "NSIS_TEMPLATE_PATCH_COUNT_MISMATCH: expected=$ExpectedCount actual=$Count value=$OldValue"
    }
    return $Content.Replace($OldValue, $NewValue)
}

function New-BrandedNsisTemplate {
    New-Item -ItemType Directory -Path (Split-Path -Parent $GeneratedNsisTemplate) -Force | Out-Null
    Invoke-WebRequest -Uri $TauriNsisTemplateUrl -OutFile $GeneratedNsisTemplate -TimeoutSec 60

    $ActualBlobSha = Get-GitBlobSha1 -Path $GeneratedNsisTemplate
    if ($ActualBlobSha -ne $TauriNsisTemplateBlobSha) {
        throw "TAURI_NSIS_TEMPLATE_BLOB_SHA_MISMATCH: expected=$TauriNsisTemplateBlobSha actual=$ActualBlobSha"
    }

    $Template = [System.IO.File]::ReadAllText($GeneratedNsisTemplate)
    $Template = Replace-RequiredLiteral -Content $Template -OldValue '!define PLACEHOLDER_INSTALL_DIR "placeholder\${PRODUCTNAME}"' -NewValue '!define PLACEHOLDER_INSTALL_DIR "placeholder\TurkuazLabs\${PRODUCTNAME}"' -ExpectedCount 1
    $Template = Replace-RequiredLiteral -Content $Template -OldValue '  !define MULTIUSER_INSTALLMODE_INSTDIR "${PRODUCTNAME}"' -NewValue '  !define MULTIUSER_INSTALLMODE_INSTDIR "TurkuazLabs\${PRODUCTNAME}"' -ExpectedCount 1
    $Template = Replace-RequiredLiteral -Content $Template -OldValue '          StrCpy $INSTDIR "$PROGRAMFILES64\${PRODUCTNAME}"' -NewValue '          StrCpy $INSTDIR "$PROGRAMFILES64\TurkuazLabs\${PRODUCTNAME}"' -ExpectedCount 2
    $Template = Replace-RequiredLiteral -Content $Template -OldValue '          StrCpy $INSTDIR "$PROGRAMFILES\${PRODUCTNAME}"' -NewValue '          StrCpy $INSTDIR "$PROGRAMFILES\TurkuazLabs\${PRODUCTNAME}"' -ExpectedCount 1
    $Template = Replace-RequiredLiteral -Content $Template -OldValue '        StrCpy $INSTDIR "$PROGRAMFILES\${PRODUCTNAME}"' -NewValue '        StrCpy $INSTDIR "$PROGRAMFILES\TurkuazLabs\${PRODUCTNAME}"' -ExpectedCount 1
    $Template = Replace-RequiredLiteral -Content $Template -OldValue '      StrCpy $INSTDIR "$LOCALAPPDATA\${PRODUCTNAME}"' -NewValue '      StrCpy $INSTDIR "$LOCALAPPDATA\TurkuazLabs\${PRODUCTNAME}"' -ExpectedCount 1

    $Utf8NoBom = New-Object System.Text.UTF8Encoding($false)
    [System.IO.File]::WriteAllText($GeneratedNsisTemplate, $Template, $Utf8NoBom)

    foreach ($Required in @(
        '!define MULTIUSER_INSTALLMODE_INSTDIR "TurkuazLabs\${PRODUCTNAME}"',
        'StrCpy $INSTDIR "$PROGRAMFILES64\TurkuazLabs\${PRODUCTNAME}"',
        'StrCpy $INSTDIR "$PROGRAMFILES\TurkuazLabs\${PRODUCTNAME}"',
        'StrCpy $INSTDIR "$LOCALAPPDATA\TurkuazLabs\${PRODUCTNAME}"'
    )) {
        if (-not $Template.Contains($Required)) {
            throw "BRANDED_NSIS_TEMPLATE_MISSING: $Required"
        }
    }
}

$Version = Get-WorkspaceVersion
Write-Host "TurkuazVM Windows distribution build v$Version"
Write-Host "Install root contract: $BrandInstallSubdirectory"

Reset-Directory -Path $StageRoot
Reset-Directory -Path $OutputRoot

Write-Host "[1/7] Engine ve Display release binaryleri derleniyor..."
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

Write-Host "[2/7] Runtime stage hazirlaniyor..."
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

Write-Host "[3/7] Tauri NSIS template'i dogrulaniyor ve TurkuazLabs install-root patchi uygulanıyor..."
New-BrandedNsisTemplate

$GeneratedTauriConfigContent = @'
{
  "bundle": {
    "resources": {
      "distribution/windows/stage/": ""
    },
    "windows": {
      "nsis": {
        "template": "windows/installer.generated.nsi"
      }
    }
  }
}
'@
Set-Content -LiteralPath $GeneratedTauriConfig -Value $GeneratedTauriConfigContent -Encoding utf8

Write-Host "[4/7] Tauri NSIS installer derleniyor..."
try {
    Invoke-NativeChecked -FilePath "tauri" -Arguments @(
        "build",
        "--bundles", "nsis",
        "--config", "src-tauri/tauri.distribution.generated.conf.json5"
    ) -WorkingDirectory $DesktopRoot
}
finally {
    foreach ($GeneratedPath in @($GeneratedTauriConfig, $GeneratedNsisTemplate)) {
        if (Test-Path -LiteralPath $GeneratedPath -PathType Leaf) {
            Remove-Item -LiteralPath $GeneratedPath -Force
        }
    }
}

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

Write-Host "[5/7] Portable paket olusturuluyor..."
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

Write-Host "[6/7] SHA-256 manifest olusturuluyor..."
$HashName = "TurkuazVM-$Version-SHA256SUMS.txt"
$HashPath = Join-Path $OutputRoot $HashName
$HashLines = foreach ($Artifact in @($SetupPath, $PortablePath)) {
    $Hash = Get-FileHash -LiteralPath $Artifact -Algorithm SHA256
    "{0}  {1}" -f $Hash.Hash.ToLowerInvariant(), (Split-Path -Leaf $Artifact)
}
Set-Content -LiteralPath $HashPath -Value $HashLines -Encoding ascii

Write-Host "[7/7] Dagitim paketi hazir."
Get-ChildItem -LiteralPath $OutputRoot -File | ForEach-Object {
    Write-Host ("  {0} ({1:N0} bytes)" -f $_.Name, $_.Length)
}

Write-Output "WINDOWS_DISTRIBUTION_VERSION=$Version"
Write-Output "WINDOWS_DISTRIBUTION_INSTALL_SUBDIRECTORY=$BrandInstallSubdirectory"
Write-Output "WINDOWS_DISTRIBUTION_SETUP=$SetupPath"
Write-Output "WINDOWS_DISTRIBUTION_PORTABLE=$PortablePath"
Write-Output "WINDOWS_DISTRIBUTION_HASHES=$HashPath"
