# 📄 Dosya Yolu: /turkuazvm/scripts/test_windows_installer_smoke.ps1
# 📌 Amac: Uretilen TurkuazVM NSIS paketini current-user ve per-machine modlarinda gercek Windows runner uzerinde kurup kaldirarak dogrular
# 📌 Modul - PowerShell Tool/Test
# Version: 0.41.6
# Aciklama: /CurrentUser ve /AllUsers kurulumlarini, TurkuazLabs/TurkuazVM install-root'unu, registry scope'larini, LOCALAPPDATA runtime materialization'ini ve uninstall davranisini fail-closed test eder
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

# Installer target location and application runtime location are deliberately tested
# as separate concepts. Installed binaries live below TurkuazLabs/TurkuazVM, while
# writable runtime state remains under the dedicated LOCALAPPDATA/TurkuazVM root.
$WindowsLocalAppData = [Environment]::GetFolderPath([Environment+SpecialFolder]::LocalApplicationData)
$WindowsProgramFiles = [Environment]::GetFolderPath([Environment+SpecialFolder]::ProgramFiles)
$WindowsProgramFilesX86 = [Environment]::GetFolderPath([Environment+SpecialFolder]::ProgramFilesX86)
$BrandInstallRelativePath = "TurkuazLabs\TurkuazVM"
$OriginalLocalAppData = $env:LOCALAPPDATA
$SmokeLocalAppData = Join-Path ([System.IO.Path]::GetTempPath()) ("TurkuazVM-installer-smoke-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $SmokeLocalAppData -Force | Out-Null
$env:LOCALAPPDATA = $SmokeLocalAppData
$RuntimeRoot = Join-Path $SmokeLocalAppData "TurkuazVM"
$RuntimeConfigPath = Join-Path $RuntimeRoot "config/turkuazvm.yml"
$RuntimeDownloadSourcesPath = Join-Path $RuntimeRoot "config/download-sources.yml"
$RegistryPaths = @{
    CurrentUser = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*"
    AllUsers = "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*"
}

function Invoke-ProcessChecked {
    param(
        [Parameter(Mandatory = $true)][string]$FilePath,
        [string[]]$ArgumentList = @()
    )

    $Process = Start-Process -FilePath $FilePath -ArgumentList $ArgumentList -Wait -PassThru
    if ($Process.ExitCode -ne 0) {
        throw "PROCESS_FAILED: $FilePath exit=$($Process.ExitCode) args=$($ArgumentList -join ' ')"
    }
}

function Normalize-RegistryPath {
    param([AllowEmptyString()][string]$Value)

    if ([string]::IsNullOrWhiteSpace($Value)) {
        return ""
    }
    return $Value.Trim().Trim([char]34)
}

function Normalize-FullPath {
    param([Parameter(Mandatory = $true)][string]$Path)
    return [System.IO.Path]::GetFullPath($Path).TrimEnd([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar)
}

function Get-SafePropertyValue {
    param(
        [Parameter(Mandatory = $true)]$InputObject,
        [Parameter(Mandatory = $true)][string]$Name
    )

    $Property = $InputObject.PSObject.Properties[$Name]
    if ($null -eq $Property) {
        return $null
    }
    return $Property.Value
}

function Get-TurkuazUninstallEntry {
    param(
        [Parameter(Mandatory = $true)]
        [ValidateSet("CurrentUser", "AllUsers")]
        [string]$Scope
    )

    $Entries = Get-ItemProperty -Path $RegistryPaths[$Scope] -ErrorAction SilentlyContinue
    return $Entries |
        Where-Object { [string](Get-SafePropertyValue -InputObject $_ -Name "DisplayName") -eq "TurkuazVM" } |
        Sort-Object { [string](Get-SafePropertyValue -InputObject $_ -Name "DisplayVersion") } -Descending |
        Select-Object -First 1
}

function Resolve-UninstallerPath {
    param([Parameter(Mandatory = $true)]$Entry)

    $InstallLocation = Normalize-RegistryPath -Value ([string](Get-SafePropertyValue -InputObject $Entry -Name "InstallLocation"))
    if (-not [string]::IsNullOrWhiteSpace($InstallLocation)) {
        $Candidate = Join-Path $InstallLocation "uninstall.exe"
        if (Test-Path -LiteralPath $Candidate -PathType Leaf) {
            return (Resolve-Path -LiteralPath $Candidate).Path
        }
    }

    $Command = [string](Get-SafePropertyValue -InputObject $Entry -Name "UninstallString")
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

function Remove-ExistingTurkuazInstallations {
    foreach ($Scope in @("CurrentUser", "AllUsers")) {
        $Entry = Get-TurkuazUninstallEntry -Scope $Scope
        if ($null -eq $Entry) {
            continue
        }
        $Uninstaller = Resolve-UninstallerPath -Entry $Entry
        if (Test-Path -LiteralPath $Uninstaller -PathType Leaf) {
            Invoke-ProcessChecked -FilePath $Uninstaller -ArgumentList @("/S")
        }
    }
}

function Assert-InstalledLayout {
    param([Parameter(Mandatory = $true)][string]$InstallRoot)

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
    return $RequiredPaths[0]
}

function Assert-InstallRootScope {
    param(
        [Parameter(Mandatory = $true)][string]$InstallRoot,
        [Parameter(Mandatory = $true)]
        [ValidateSet("CurrentUser", "AllUsers")]
        [string]$ExpectedScope
    )

    $NormalizedInstallRoot = Normalize-FullPath -Path $InstallRoot

    if ($ExpectedScope -eq "CurrentUser") {
        if ([string]::IsNullOrWhiteSpace($WindowsLocalAppData)) {
            throw "WINDOWS_LOCALAPPDATA_KNOWN_FOLDER_MISSING"
        }
        $ExpectedRoot = Normalize-FullPath -Path (Join-Path $WindowsLocalAppData $BrandInstallRelativePath)
        if (-not $NormalizedInstallRoot.Equals($ExpectedRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
            throw "CURRENT_USER_INSTALL_ROOT_MISMATCH: expected=$ExpectedRoot actual=$NormalizedInstallRoot"
        }
        return
    }

    $AllowedProgramRoots = @($WindowsProgramFiles, $WindowsProgramFilesX86) |
        Where-Object { -not [string]::IsNullOrWhiteSpace($_) } |
        Select-Object -Unique
    if ($AllowedProgramRoots.Count -eq 0) {
        throw "WINDOWS_PROGRAM_FILES_KNOWN_FOLDER_MISSING"
    }

    $ExpectedRoots = @($AllowedProgramRoots | ForEach-Object {
        Normalize-FullPath -Path (Join-Path $_ $BrandInstallRelativePath)
    })
    $MatchesExpectedRoot = $false
    foreach ($ExpectedRoot in $ExpectedRoots) {
        if ($NormalizedInstallRoot.Equals($ExpectedRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
            $MatchesExpectedRoot = $true
            break
        }
    }
    if (-not $MatchesExpectedRoot) {
        throw "ALL_USERS_INSTALL_ROOT_MISMATCH: expected=$($ExpectedRoots -join ';') actual=$NormalizedInstallRoot"
    }
}

function Assert-PackagedRuntimeTemplate {
    param([Parameter(Mandatory = $true)][string]$InstallRoot)

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
}

function Assert-MaterializedRuntime {
    param(
        [Parameter(Mandatory = $true)][string]$InstallRoot,
        [Parameter(Mandatory = $true)][string]$InstalledExe
    )

    $DesktopProcess = $null
    try {
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
    }
    finally {
        Stop-TurkuazRuntimeProcesses -DesktopProcess $DesktopProcess
    }
}

function Invoke-InstallerModeSmoke {
    param(
        [Parameter(Mandatory = $true)][string]$SetupPath,
        [Parameter(Mandatory = $true)]
        [ValidateSet("/CurrentUser", "/AllUsers")]
        [string]$ModeArgument,
        [Parameter(Mandatory = $true)]
        [ValidateSet("CurrentUser", "AllUsers")]
        [string]$ExpectedScope,
        [Parameter(Mandatory = $true)][string]$Label
    )

    if (Test-Path -LiteralPath $RuntimeRoot) {
        Remove-Item -LiteralPath $RuntimeRoot -Recurse -Force
    }

    Write-Host "[$Label] NSIS sessiz kurulum: $ModeArgument"
    Invoke-ProcessChecked -FilePath $SetupPath -ArgumentList @("/S", $ModeArgument, "/NS")

    $Entry = $null
    $Deadline = (Get-Date).AddSeconds(30)
    while ((Get-Date) -lt $Deadline) {
        $Entry = Get-TurkuazUninstallEntry -Scope $ExpectedScope
        if ($null -ne $Entry) {
            break
        }
        Start-Sleep -Milliseconds 500
    }
    if ($null -eq $Entry) {
        throw "TURKUAZVM_UNINSTALL_ENTRY_NOT_FOUND: scope=$ExpectedScope"
    }

    $UnexpectedScope = if ($ExpectedScope -eq "CurrentUser") { "AllUsers" } else { "CurrentUser" }
    if ($null -ne (Get-TurkuazUninstallEntry -Scope $UnexpectedScope)) {
        throw "TURKUAZVM_UNEXPECTED_REGISTRY_SCOPE: expected=$ExpectedScope unexpected=$UnexpectedScope"
    }

    $Uninstaller = Resolve-UninstallerPath -Entry $Entry
    $InstallRoot = Split-Path -Parent $Uninstaller
    if (-not (Test-Path -LiteralPath $InstallRoot -PathType Container)) {
        throw "INSTALL_ROOT_NOT_FOUND: $InstallRoot"
    }
    Assert-InstallRootScope -InstallRoot $InstallRoot -ExpectedScope $ExpectedScope

    $InstalledExe = Assert-InstalledLayout -InstallRoot $InstallRoot
    Assert-PackagedRuntimeTemplate -InstallRoot $InstallRoot
    Assert-MaterializedRuntime -InstallRoot $InstallRoot -InstalledExe $InstalledExe

    Write-Host "[$Label] NSIS sessiz kaldirma"
    Invoke-ProcessChecked -FilePath $Uninstaller -ArgumentList @("/S")
    Wait-PathRemoved -Path $InstalledExe
    if ($null -ne (Get-TurkuazUninstallEntry -Scope $ExpectedScope)) {
        throw "TURKUAZVM_UNINSTALL_ENTRY_STILL_PRESENT: scope=$ExpectedScope"
    }
    if (-not (Test-Path -LiteralPath $RuntimeConfigPath -PathType Leaf)) {
        throw "UNINSTALL_REMOVED_USER_RUNTIME_CONFIG: mode=$ModeArgument"
    }

    Write-Output "WINDOWS_INSTALL_MODE_SMOKE_${ExpectedScope}=PASS"
}

$Setup = Get-ChildItem -LiteralPath $ArtifactRoot -Filter "TurkuazVM-*-x64-Setup.exe" -File |
    Sort-Object LastWriteTimeUtc -Descending |
    Select-Object -First 1
if ($null -eq $Setup) {
    throw "SETUP_NOT_FOUND: $ArtifactRoot"
}

try {
    Remove-ExistingTurkuazInstallations

    Write-Host "[1/2] Current-user kurulum modu dogrulaniyor..."
    Invoke-InstallerModeSmoke -SetupPath $Setup.FullName -ModeArgument "/CurrentUser" -ExpectedScope "CurrentUser" -Label "current-user"

    Write-Host "[2/2] Per-machine kurulum modu dogrulaniyor..."
    Invoke-InstallerModeSmoke -SetupPath $Setup.FullName -ModeArgument "/AllUsers" -ExpectedScope "AllUsers" -Label "all-users"

    Write-Output "WINDOWS_INSTALLER_SMOKE=PASS"
    Write-Output "WINDOWS_RUNTIME_DATA_ROOT_SMOKE=PASS"
    Write-Output "WINDOWS_DUAL_INSTALL_MODE_SMOKE=PASS"
    Write-Output "WINDOWS_TURKUAZLABS_INSTALL_ROOT_SMOKE=PASS"
}
finally {
    Stop-TurkuazRuntimeProcesses -DesktopProcess $null
    try { Remove-ExistingTurkuazInstallations } catch { Write-Warning $_ }
    if (Test-Path -LiteralPath $SmokeLocalAppData) {
        Remove-Item -LiteralPath $SmokeLocalAppData -Recurse -Force -ErrorAction SilentlyContinue
    }
    $env:LOCALAPPDATA = $OriginalLocalAppData
}
