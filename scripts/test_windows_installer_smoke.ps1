# 📄 Dosya Yolu: /turkuazvm/scripts/test_windows_installer_smoke.ps1
# 📌 Amac: Uretilen TurkuazVM NSIS paketini gercek Windows runner uzerinde sessiz kurup kaldirarak dogrular
# 📌 Modul - PowerShell Tool/Test
# Version: 0.41.4
# Aciklama: Setup exit code, HKCU uninstall kaydi, tirnakli registry yollari, gercek NSIS Desktop binary adi, runtime dosyalari/config ve sessiz uninstall davranisini fail-closed test eder
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
try {
    Write-Host "[1/4] NSIS sessiz kurulum baslatiliyor: $($Setup.Name)"
    Invoke-ProcessChecked -FilePath $Setup.FullName -ArgumentList @("/S")

    Write-Host "[2/4] HKCU uninstall kaydi ve kurulum konumu dogrulaniyor..."
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

    Write-Host "[3/4] Kurulu runtime config dogrulaniyor..."
    $RuntimeConfig = Get-Content -LiteralPath (Join-Path $InstallRoot "config/turkuazvm.yml") -Raw
    if ($RuntimeConfig -notmatch 'display_executable_path:\s*bin/turkuazvm-display\.exe') {
        throw "INSTALLED_CONFIG_DISPLAY_PATH_INVALID"
    }
    if ($RuntimeConfig -notmatch 'executable_path:\s*bin/turkuazvm-engine\.exe') {
        throw "INSTALLED_CONFIG_ENGINE_PATH_INVALID"
    }
    if ($RuntimeConfig -match 'target/debug/turkuazvm-(engine|display)') {
        throw "INSTALLED_CONFIG_REFERENCES_DEBUG_BINARY"
    }

    Write-Host "[4/4] NSIS sessiz kaldirma dogrulaniyor..."
    Invoke-ProcessChecked -FilePath $Uninstaller -ArgumentList @("/S")
    Wait-PathRemoved -Path $InstalledExe

    if ($null -ne (Get-TurkuazUninstallEntry)) {
        throw "TURKUAZVM_UNINSTALL_ENTRY_STILL_PRESENT"
    }

    Write-Output "WINDOWS_INSTALLER_SMOKE=PASS"
}
finally {
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
}
