# 📄 Dosya Yolu: /turkuazvm/scripts/test_windows_installer_smoke.ps1
# 📌 Amac: Uretilen TurkuazVM NSIS paketini gercek Windows runner uzerinde sessiz kurup, AppData runtime'ini baslatip ve kaldirarak dogrular
# 📌 Modul - PowerShell Tool/Test
# Version: 0.41.5
# Aciklama: Setup/uninstall, HKCU kaydi, kurulu layout, LOCALAPPDATA runtime config materialization'i ve uninstall sonrasi kullanici verisi korunmasini fail-closed test eder
# Bagimli Oldugu Katman: Tool | CI/CD | View

[CmdletBinding()]
param(
    [string]$ArtifactRoot = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
if ([string]::IsNullOrWhiteSpace($ArtifactRoot)) {
    $ArtifactRoot = Join-Path $Root "artifacts/distribution/windows"
}
$ArtifactRoot = (Resolve-Path -LiteralPath $ArtifactRoot).Path
$OriginalLocalAppData = $env:LOCALAPPDATA
$SmokeLocalAppData = Join-Path ([System.IO.Path]::GetTempPath()) ("TurkuazVM-installer-smoke-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $SmokeLocalAppData -Force | Out-Null
$env:LOCALAPPDATA = $SmokeLocalAppData
$RuntimeRoot = Join-Path $SmokeLocalAppData "TurkuazVM"
$RuntimeConfigPath = Join-Path $RuntimeRoot "config/turkuazvm.yml"
$RuntimeDownloadSourcesPath = Join-Path $RuntimeRoot "config/download-sources.yml"

function Invoke-ProcessChecked {
    param(
        [Parameter(Mandatory = $true)][string]$FilePath,
        [string[]]$ArgumentList = @()
    )

    $Process = Start-Process -FilePath $FilePath -ArgumentList $ArgumentList -Wait -PassThru
    if ($Process.ExitCode -ne 0) {
        throw "PROCESS_FAILED: $FilePath exit=$($Process.ExitCode)"
    }
}

function Normalize-RegistryPath {
    param([AllowEmptyString()][string]$Value)

    if ([string]::IsNullOrWhiteSpace($Value)) {
        return ""
    }
    return $Value.Trim().Trim([char]34)
}

function Get-TurkuazUninstallEntry {
    $Entries = Get-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*" -ErrorAction SilentlyContinue
    return $Entries |
        Where-Object { $_.DisplayName -eq "TurkuazVM" } |
        Sort-Object { [string]$_.DisplayVersion } -Descending |
        Select-Object -First 1
}

function Resolve-UninstallerPath {
    param([Parameter(Mandatory = $true)]$Entry)

    $InstallLocation = Normalize-RegistryPath -Value ([string]$Entry.InstallLocation)
    if (-not [string]::IsNullOrWhiteSpace($InstallLocation)) {
        $Candidate = Join-Path $InstallLocation "uninstall.exe"
        if (Test-Path -LiteralPath $Candidate -PathType Leaf) {
            return (Resolve-Path -LiteralPath $Candidate).Path
        }
    }

    $Command = [string]$Entry.UninstallString
    if ($Command -match '^\s*"([^"]+\.exe)"') {
        return $Matches[1]
    }
    if ($Command -match '^\s*([^\s]+\.exe)') {
        return $Matches[1]
    }
    throw "UNINSTALLER_PATH_NOT_RESOLVED"
}

function Wait-PathRemoved {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [int]$TimeoutSeconds = 30
    )

    $Deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $Deadline) {
        if (-not (Test-Path -LiteralPath $Path)) {
            return
        }
        Start-Sleep -Milliseconds 500
    }
    throw "PATH_NOT_REMOVED_AFTER_UNINSTALL: $Path"
}

function Wait-FileCreated {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)]$Process,
        [int]$TimeoutSeconds = 30
    )

    $Deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $Deadline) {
        if (Test-Path -LiteralPath $Path -PathType Leaf) {
            return
        }
        if ($Process.HasExited) {
            throw "DESKTOP_EXITED_BEFORE_RUNTIME_CONFIG: exit=$($Process.ExitCode)"
        }
        Start-Sleep -Milliseconds 500
    }
    throw "RUNTIME_CONFIG_NOT_MATERIALIZED: $Path"
}

function Stop-TurkuazRuntimeProcesses {
    param($DesktopProcess)

    if ($null -ne $DesktopProcess -and -not $DesktopProcess.HasExited) {
        Stop-Process -Id $DesktopProcess.Id -Force -ErrorAction SilentlyContinue
        try { $DesktopProcess.WaitForExit(10000) | Out-Null } catch { }
    }
    Get-Process -Name "turkuazvm-engine" -ErrorAction SilentlyContinue |
        Stop-Process -Force -ErrorAction SilentlyContinue
    Get-Process -Name "turkuazvm-display" -ErrorAction SilentlyContinue |
        Stop-Process -Force -ErrorAction SilentlyContinue
}

$Setup = Get-ChildItem -LiteralPath $ArtifactRoot -Filter "TurkuazVM-*-x64-Setup.exe" -File |
    Sort-Object LastWriteTimeUtc -Descending |
    Select-Object -First 1
if ($null -eq $Setup) {
    throw "SETUP_NOT_FOUND: $ArtifactRoot"
}

$ExistingEntry = Get-TurkuazUninstallEntry
if ($null -ne $ExistingEntry) {
    $ExistingUninstaller = Resolve-UninstallerPath -Entry $ExistingEntry
    Invoke-ProcessChecked -FilePath $ExistingUninstaller -ArgumentList @("/S")
}

$InstalledExe = $null
$Uninstaller = $null
$DesktopProcess = $null
try {
    Write-Host "[1/6] NSIS sessiz kurulum baslatiliyor: $($Setup.Name)"
    Invoke-ProcessChecked -FilePath $Setup.FullName -ArgumentList @("/S")

    Write-Host "[2/6] HKCU uninstall kaydi ve kurulum konumu dogrulaniyor..."
    $Entry = $null
    $Deadline = (Get-Date).AddSeconds(30)
    while ((Get-Date) -lt $Deadline) {
        $Entry = Get-TurkuazUninstallEntry
        if ($null -ne $Entry) {
            break
        }
        Start-Sleep -Milliseconds 500
    }
    if ($null -eq $Entry) {
        throw "TURKUAZVM_UNINSTALL_ENTRY_NOT_FOUND"
    }

    $Uninstaller = Resolve-UninstallerPath -Entry $Entry
    $InstallRoot = Split-Path -Parent $Uninstaller
    if (-not (Test-Path -LiteralPath $InstallRoot -PathType Container)) {
        throw "INSTALL_ROOT_NOT_FOUND: $InstallRoot"
    }

    $RequiredPaths = @(
        (Join-Path $InstallRoot "turkuazvm-desktop.exe"),
        (Join-Path $InstallRoot "bin/turkuazvm-engine.exe"),
        (Join-Path $InstallRoot "bin/turkuazvm-display.exe"),
        (Join-Path $InstallRoot "config/turkuazvm.yml")
    )
    foreach ($RequiredPath in $RequiredPaths) {
        if (-not (Test-Path -LiteralPath $RequiredPath -PathType Leaf)) {
            throw "INSTALLED_FILE_MISSING: $RequiredPath"
        }
    }
    $InstalledExe = $RequiredPaths[0]

    Write-Host "[3/6] Kurulu read-only runtime template dogrulaniyor..."
    $PackagedRuntimeConfig = Get-Content -LiteralPath (Join-Path $InstallRoot "config/turkuazvm.yml") -Raw
    if ($PackagedRuntimeConfig -notmatch 'display_executable_path:\s*bin/turkuazvm-display\.exe') {
        throw "INSTALLED_CONFIG_DISPLAY_PATH_INVALID"
    }
    if ($PackagedRuntimeConfig -notmatch 'executable_path:\s*bin/turkuazvm-engine\.exe') {
        throw "INSTALLED_CONFIG_ENGINE_PATH_INVALID"
    }
    if ($PackagedRuntimeConfig -match 'target/debug/turkuazvm-(engine|display)') {
        throw "INSTALLED_CONFIG_REFERENCES_DEBUG_BINARY"
    }

    Write-Host "[4/6] Desktop baslatiliyor ve LOCALAPPDATA runtime config materialization'i dogrulaniyor..."
    $DesktopProcess = Start-Process -FilePath $InstalledExe -PassThru
    Wait-FileCreated -Path $RuntimeConfigPath -Process $DesktopProcess
    if (-not (Test-Path -LiteralPath $RuntimeDownloadSourcesPath -PathType Leaf)) {
        throw "RUNTIME_DOWNLOAD_SOURCES_NOT_MATERIALIZED: $RuntimeDownloadSourcesPath"
    }
    foreach ($Directory in @((Join-Path $RuntimeRoot "data"), (Join-Path $RuntimeRoot "packages"))) {
        if (-not (Test-Path -LiteralPath $Directory -PathType Container)) {
            throw "RUNTIME_DIRECTORY_NOT_MATERIALIZED: $Directory"
        }
    }

    $RuntimeConfig = Get-Content -LiteralPath $RuntimeConfigPath -Raw
    $InstallRootYaml = $InstallRoot.Replace('\', '/')
    foreach ($Expected in @(
        "display_executable_path: `"$InstallRootYaml/bin/turkuazvm-display.exe`"",
        "executable_path: `"$InstallRootYaml/bin/turkuazvm-engine.exe`"",
        "managed_helper_path: `"$InstallRootYaml/scripts/network_windows_managed.ps1`"",
        "build_script: `"$InstallRootYaml/guest/android-image/scripts/build_turkuaz_android_image.sh`""
    )) {
        if (-not $RuntimeConfig.Contains($Expected)) {
            throw "RUNTIME_CONFIG_ABSOLUTE_ASSET_PATH_MISSING: $Expected"
        }
    }
    if ($RuntimeConfig -notmatch 'data_root:\s*\./data') {
        throw "RUNTIME_CONFIG_DATA_ROOT_NOT_LOCALAPPDATA_RELATIVE"
    }
    if ($RuntimeConfig -match 'target/debug/turkuazvm-(engine|display)') {
        throw "RUNTIME_CONFIG_REFERENCES_DEBUG_BINARY"
    }
    Start-Sleep -Seconds 2
    if ($DesktopProcess.HasExited) {
        throw "DESKTOP_EXITED_AFTER_RUNTIME_MATERIALIZATION: exit=$($DesktopProcess.ExitCode)"
    }
    Stop-TurkuazRuntimeProcesses -DesktopProcess $DesktopProcess
    $DesktopProcess = $null

    Write-Host "[5/6] NSIS sessiz kaldirma dogrulaniyor..."
    Invoke-ProcessChecked -FilePath $Uninstaller -ArgumentList @("/S")
    Wait-PathRemoved -Path $InstalledExe
    if ($null -ne (Get-TurkuazUninstallEntry)) {
        throw "TURKUAZVM_UNINSTALL_ENTRY_STILL_PRESENT"
    }

    Write-Host "[6/6] Uninstall kullanici runtime verisini koruyor..."
    if (-not (Test-Path -LiteralPath $RuntimeConfigPath -PathType Leaf)) {
        throw "UNINSTALL_REMOVED_USER_RUNTIME_CONFIG"
    }

    Write-Output "WINDOWS_INSTALLER_SMOKE=PASS"
    Write-Output "WINDOWS_RUNTIME_DATA_ROOT_SMOKE=PASS"
}
finally {
    Stop-TurkuazRuntimeProcesses -DesktopProcess $DesktopProcess
    if ($null -ne $InstalledExe -and (Test-Path -LiteralPath $InstalledExe)) {
        if ($null -ne $Uninstaller -and (Test-Path -LiteralPath $Uninstaller -PathType Leaf)) {
            try {
                Invoke-ProcessChecked -FilePath $Uninstaller -ArgumentList @("/S")
            }
            catch {
                Write-Warning "Installer smoke cleanup failed: $($_.Exception.Message)"
            }
        }
    }
    if (Test-Path -LiteralPath $SmokeLocalAppData) {
        Remove-Item -LiteralPath $SmokeLocalAppData -Recurse -Force -ErrorAction SilentlyContinue
    }
    $env:LOCALAPPDATA = $OriginalLocalAppData
}
