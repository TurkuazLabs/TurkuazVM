# 📄 Dosya Yolu: /turkuazvm/scripts/start_turkuazvm.ps1
# 📌 Amac: TurkuazVM Windows test ortaminda dependency preflight, build ve Desktop baslatma akisini yonetir
# 📌 Modul - PowerShell
# Version: 0.41.2
# Aciklama: Tek tik launcher preflight/build akisini yonetir; v0.41.2 Connection Center release metadata ve mevcut SDK/runtime guvenlik gate zincirini uygular
# Bagimli Oldugu Katman: Tool | View

param(
    [switch]$NoPause
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$ConfigPath = Join-Path $ProjectRoot "config/turkuazvm.yml"
$LogRoot = Join-Path $ProjectRoot "data/logs/launcher"
$Script:TranscriptStarted = $false
$DependencyBootstrapPath = Join-Path $PSScriptRoot "bootstrap_windows_dependencies.ps1"
$WorkspaceManifestPath = Join-Path $ProjectRoot "Cargo.toml"
$RustToolchainPath = Join-Path $ProjectRoot "rust-toolchain.toml"
$NativeProcessToolPath = Join-Path $ProjectRoot "tools/windows_native_process_tool.ps1"

function Get-WorkspacePackageVersion {
    if (-not (Test-Path -LiteralPath $WorkspaceManifestPath -PathType Leaf)) {
        throw "WORKSPACE_MANIFEST_NOT_FOUND"
    }

    $InWorkspacePackage = $false
    foreach ($Line in Get-Content -LiteralPath $WorkspaceManifestPath) {
        if ($Line -match '^\s*\[workspace\.package\]\s*$') {
            $InWorkspacePackage = $true
            continue
        }

        if ($InWorkspacePackage -and $Line -match '^\s*\[') {
            break
        }

        if ($InWorkspacePackage -and $Line -match '^\s*version\s*=\s*"([0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?)"\s*$') {
            return $Matches[1]
        }
    }

    throw "WORKSPACE_VERSION_NOT_FOUND"
}

function Get-RustToolchainPolicy {
    if (-not (Test-Path -LiteralPath $RustToolchainPath -PathType Leaf)) {
        throw "RUST_TOOLCHAIN_POLICY_NOT_FOUND"
    }

    $Channel = $null
    $Components = New-Object System.Collections.Generic.List[string]
    foreach ($Line in Get-Content -LiteralPath $RustToolchainPath) {
        if ($Line -match '^\s*channel\s*=\s*"([^"]+)"\s*$') {
            $Channel = $Matches[1]
            continue
        }

        if ($Line -match '^\s*components\s*=\s*\[(.*)\]\s*$') {
            foreach ($Match in [regex]::Matches($Matches[1], '"([^"]+)"')) {
                $Components.Add($Match.Groups[1].Value)
            }
        }
    }

    if ([string]::IsNullOrWhiteSpace($Channel)) {
        throw "RUST_TOOLCHAIN_CHANNEL_NOT_FOUND"
    }

    return [pscustomobject]@{
        Channel = $Channel
        Components = @($Components)
    }
}

$WorkspacePackageVersion = Get-WorkspacePackageVersion
$RustToolchainPolicy = Get-RustToolchainPolicy
$RequiredRustToolchain = $RustToolchainPolicy.Channel

if (-not (Test-Path -LiteralPath $NativeProcessToolPath -PathType Leaf)) {
    throw "WINDOWS_NATIVE_PROCESS_TOOL_NOT_FOUND"
}
. $NativeProcessToolPath

if (-not (Test-Path -LiteralPath $DependencyBootstrapPath -PathType Leaf)) {
    throw "WINDOWS_DEPENDENCY_BOOTSTRAP_NOT_FOUND"
}
. $DependencyBootstrapPath

function Write-Status {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][string]$Status
    )

    Write-Host ("{0,-30} {1}" -f $Name, $Status)
}

function Add-PathDirectory {
    param([Parameter(Mandatory = $true)][string]$Path)

    if ([string]::IsNullOrWhiteSpace($Path) -or -not (Test-Path -LiteralPath $Path -PathType Container)) {
        return
    }

    $Entries = $env:PATH -split ";"
    if ($Entries -notcontains $Path) {
        $env:PATH = "$Path;$env:PATH"
    }
}

function Resolve-ToolCommand {
    param([Parameter(Mandatory = $true)][string]$Name)

    return Get-Command $Name -ErrorAction SilentlyContinue
}

function Import-VisualStudioEnvironment {
    $VsWhereCandidates = @(
        (Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio/Installer/vswhere.exe"),
        (Join-Path $env:ProgramFiles "Microsoft Visual Studio/Installer/vswhere.exe")
    ) | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }

    $VsWhere = $VsWhereCandidates | Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } | Select-Object -First 1
    if ($null -eq $VsWhere) {
        return $false
    }

    $PreviousErrorActionPreference = $ErrorActionPreference
    try {
        $ErrorActionPreference = "Continue"
        $InstallPath = (& $VsWhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null | Select-Object -First 1)
    }
    finally {
        $ErrorActionPreference = $PreviousErrorActionPreference
    }
    if ([string]::IsNullOrWhiteSpace($InstallPath)) {
        return $false
    }

    $VsDevCmd = Join-Path $InstallPath "Common7/Tools/VsDevCmd.bat"
    if (-not (Test-Path -LiteralPath $VsDevCmd -PathType Leaf)) {
        return $false
    }

    $PreviousErrorActionPreference = $ErrorActionPreference
    try {
        $ErrorActionPreference = "Continue"
        $EnvironmentLines = & $env:ComSpec /s /c "`"$VsDevCmd`" -no_logo -arch=x64 -host_arch=x64 >nul && set"
        $VsDevCmdExitCode = $LASTEXITCODE
    }
    finally {
        $ErrorActionPreference = $PreviousErrorActionPreference
    }
    if ($VsDevCmdExitCode -ne 0) {
        return $false
    }

    foreach ($Line in $EnvironmentLines) {
        $Separator = $Line.IndexOf("=")
        if ($Separator -le 0) {
            continue
        }

        $Name = $Line.Substring(0, $Separator)
        $Value = $Line.Substring($Separator + 1)
        Set-Item -Path "Env:$Name" -Value $Value
    }

    return $true
}

function Initialize-ToolPaths {
    Add-PathDirectory -Path (Join-Path $env:USERPROFILE ".cargo/bin")
    Add-PathDirectory -Path (Join-Path $env:ProgramFiles "qemu")
    Add-PathDirectory -Path (Join-Path ${env:ProgramFiles(x86)} "qemu")

    foreach ($SdkRoot in @($env:ANDROID_SDK_ROOT, $env:ANDROID_HOME)) {
        if (-not [string]::IsNullOrWhiteSpace($SdkRoot)) {
            Add-PathDirectory -Path (Join-Path $SdkRoot "platform-tools")
        }
    }

    $LocalAppDataSdk = Join-Path $env:LOCALAPPDATA "Android/Sdk/platform-tools"
    Add-PathDirectory -Path $LocalAppDataSdk
    Initialize-TurkuazVmBootstrapPaths
}

function Test-RustVersion {
    param(
        [Parameter(Mandatory = $true)][string]$RustcPath,
        [Parameter(Mandatory = $true)][string]$RequiredVersion
    )

    $Result = Invoke-TurkuazNativeCapture -FilePath $RustcPath -Arguments @("--version")
    if ($Result.ExitCode -ne 0 -or $Result.Combined -notmatch 'rustc\s+(\d+\.\d+\.\d+)') {
        return $false
    }

    return $Matches[1] -eq $RequiredVersion
}

function Get-RustHost {
    param([Parameter(Mandatory = $true)][string]$RustcPath)

    $Result = Invoke-TurkuazNativeCapture -FilePath $RustcPath -Arguments @("-vV")
    if ($Result.ExitCode -ne 0 -or $Result.Combined -notmatch '(?im)^host:\s*(.+)$') {
        return $null
    }

    return $Matches[1].Trim()
}

function Initialize-RequiredRustToolchain {
    param(
        [Parameter(Mandatory = $true)][string]$RustupPath,
        [Parameter(Mandatory = $true)][string]$RequiredVersion,
        [string[]]$RequiredComponents = @()
    )

    $ListResult = Invoke-TurkuazNativeCapture -FilePath $RustupPath -Arguments @("toolchain", "list")
    if ($ListResult.ExitCode -ne 0) {
        return [pscustomobject]@{
            Ready = $false
            Detail = "rustup toolchain list basarisiz: $($ListResult.Combined)"
        }
    }

    $EscapedVersion = [regex]::Escape($RequiredVersion)
    $Installed = $ListResult.Combined -match "(?m)^$EscapedVersion(?:-[^\s]+)?(?:\s|$)"
    if (-not $Installed) {
        Write-Host ""
        Write-Host "Rust toolchain $RequiredVersion bulunamadi; rustup ile kuruluyor..."
        $InstallArguments = New-Object System.Collections.Generic.List[string]
        foreach ($Argument in @("toolchain", "install", $RequiredVersion, "--profile", "minimal")) {
            $InstallArguments.Add($Argument)
        }
        foreach ($Component in $RequiredComponents) {
            $InstallArguments.Add("--component")
            $InstallArguments.Add($Component)
        }

        try {
            Invoke-TurkuazNativeChecked -FilePath $RustupPath -Arguments @($InstallArguments)
        }
        catch {
            return [pscustomobject]@{
                Ready = $false
                Detail = $_.Exception.Message
            }
        }
    }

    return [pscustomobject]@{
        Ready = $true
        Detail = "Rust toolchain $RequiredVersion hazir."
    }
}

function Test-QemuWhpx {
    param([Parameter(Mandatory = $true)][string]$QemuPath)

    $Result = Invoke-TurkuazNativeCapture -FilePath $QemuPath -Arguments @("-accel", "help")
    return $Result.ExitCode -eq 0 -and $Result.Combined -match '(?im)^whpx\s*$'
}

function Test-QemuWhpxRuntime {
    param([Parameter(Mandatory = $true)][string]$QemuPath)

    $ProbeRoot = Join-Path $LogRoot "whpx-probe"
    New-Item -ItemType Directory -Path $ProbeRoot -Force | Out-Null

    $ProbeId = [Guid]::NewGuid().ToString("N")
    $StdOutPath = Join-Path $ProbeRoot "$ProbeId.stdout.log"
    $StdErrPath = Join-Path $ProbeRoot "$ProbeId.stderr.log"
    $Arguments = @(
        "-accel", "whpx",
        "-machine", "q35",
        "-m", "64M",
        "-nodefaults",
        "-display", "none",
        "-monitor", "none",
        "-serial", "none",
        "-S"
    )

    $Process = $null
    try {
        $Process = Start-Process `
            -FilePath $QemuPath `
            -ArgumentList $Arguments `
            -PassThru `
            -WindowStyle Hidden `
            -RedirectStandardOutput $StdOutPath `
            -RedirectStandardError $StdErrPath

        Start-Sleep -Milliseconds 2500
        $Process.Refresh()

        if ($Process.HasExited) {
            $StdErr = if (Test-Path -LiteralPath $StdErrPath) {
                (Get-Content -LiteralPath $StdErrPath -Raw -ErrorAction SilentlyContinue).Trim()
            } else {
                ""
            }
            $FailureDetail = if ([string]::IsNullOrWhiteSpace($StdErr)) {
                "QEMU WHPX probe erken sonlandi. exit=$($Process.ExitCode)"
            } else {
                $StdErr
            }

            return [pscustomobject]@{
                Ready = $false
                Detail = $FailureDetail
            }
        }

        Stop-Process -Id $Process.Id -Force -ErrorAction SilentlyContinue
        try { $Process.WaitForExit(3000) | Out-Null } catch { }

        return [pscustomobject]@{
            Ready = $true
            Detail = "WHPX accelerator runtime probe basarili."
        }
    }
    catch {
        return [pscustomobject]@{
            Ready = $false
            Detail = $_.Exception.Message
        }
    }
    finally {
        if ($null -ne $Process) {
            try {
                $Process.Refresh()
                if (-not $Process.HasExited) {
                    Stop-Process -Id $Process.Id -Force -ErrorAction SilentlyContinue
                }
            }
            catch { }
        }
    }
}

function Test-WebView2Runtime {
    $RegistryPaths = @(
        "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}",
        "HKLM:\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}",
        "HKCU:\Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"
    )

    foreach ($RegistryPath in $RegistryPaths) {
        if (-not (Test-Path -LiteralPath $RegistryPath)) {
            continue
        }

        $Runtime = Get-ItemProperty -LiteralPath $RegistryPath -Name "pv" -ErrorAction SilentlyContinue
        if ($null -ne $Runtime -and -not [string]::IsNullOrWhiteSpace($Runtime.pv) -and $Runtime.pv -ne "0.0.0.0") {
            return $true
        }
    }

    return $false
}

function Get-WindowsHypervisorPlatformState {
    try {
        $Feature = Get-CimInstance -ClassName "Win32_OptionalFeature" -Filter "Name='HypervisorPlatform'" -ErrorAction Stop
        if ($null -eq $Feature) {
            return "UNKNOWN"
        }

        switch ([int]$Feature.InstallState) {
            1 { return "ENABLED" }
            2 { return "DISABLED" }
            3 { return "ABSENT" }
            default { return "UNKNOWN" }
        }
    }
    catch {
        return "UNKNOWN"
    }
}

function Get-FirmwareVirtualizationState {
    try {
        $Processors = @(Get-CimInstance -ClassName "Win32_Processor" -ErrorAction Stop)
        if ($Processors.Count -eq 0) {
            return "UNKNOWN"
        }

        $Values = @($Processors | ForEach-Object { $_.VirtualizationFirmwareEnabled })
        if ($Values -contains $false) {
            return "DISABLED"
        }
        if ($Values -contains $true) {
            return "ENABLED"
        }
        return "UNKNOWN"
    }
    catch {
        return "UNKNOWN"
    }
}

function Get-HypervisorPresentState {
    try {
        $ComputerSystem = Get-CimInstance -ClassName "Win32_ComputerSystem" -ErrorAction Stop
        if ($null -eq $ComputerSystem -or $null -eq $ComputerSystem.HypervisorPresent) {
            return "UNKNOWN"
        }

        if ([bool]$ComputerSystem.HypervisorPresent) {
            return "YES"
        }
        return "NO"
    }
    catch {
        return "UNKNOWN"
    }
}

function Request-DependencyBootstrap {
    param(
        [Parameter(Mandatory = $true)][bool]$NeedsQemu,
        [Parameter(Mandatory = $true)][bool]$NeedsAdb,
        [Parameter(Mandatory = $true)][bool]$NeedsWebView2,
        [Parameter(Mandatory = $true)][bool]$NeedsOpenVpn
    )

    if (-not $NeedsQemu -and -not $NeedsAdb -and -not $NeedsWebView2 -and -not $NeedsOpenVpn) {
        return $false
    }

    if (-not (Test-TurkuazVmWingetAvailable)) {
        return $false
    }

    Write-Host ""
    Write-Host "Eksik Windows test bagimliliklari bulundu:"
    if ($NeedsQemu) { Write-Host "- QEMU (zorunlu)" }
    if ($NeedsAdb) { Write-Host "- Android Platform Tools / ADB (Android testi icin)" }
    if ($NeedsWebView2) { Write-Host "- Microsoft Edge WebView2 Runtime (Desktop icin)" }
    if ($NeedsOpenVpn) { Write-Host "- OpenVPN TAP Adapter (yalniz Advanced TAP/Bridge icin)" }
    Write-Host ""
    $Answer = Read-Host "WinGet ile otomatik kurulsun mu? [E/H]"
    if ($Answer -notmatch '^(?i:e|evet|y|yes)$') {
        return $false
    }

    Install-TurkuazVmWindowsDependencies -InstallQemu $NeedsQemu -InstallAdb $NeedsAdb -InstallWebView2 $NeedsWebView2 -InstallOpenVpn $NeedsOpenVpn
    return $true
}

function Invoke-Preflight {
    if ($env:OS -ne "Windows_NT") {
        throw "WINDOWS_REQUIRED: Bu launcher Windows test makinesi icindir."
    }

    Initialize-ToolPaths
    $VsLoaded = Import-VisualStudioEnvironment

    $Rustup = Resolve-ToolCommand -Name "rustup"
    $Rustc = Resolve-ToolCommand -Name "rustc"
    $Cargo = Resolve-ToolCommand -Name "cargo"

    $RustToolchainProbe = if ($null -ne $Rustup) {
        Initialize-RequiredRustToolchain `
            -RustupPath $Rustup.Source `
            -RequiredVersion $RequiredRustToolchain `
            -RequiredComponents $RustToolchainPolicy.Components
    } else {
        [pscustomobject]@{ Ready = $true; Detail = "rustup bulunamadi; mevcut rustc dogrudan dogrulanacak." }
    }
    $Qemu = Resolve-ToolCommand -Name "qemu-system-x86_64"
    $QemuImg = Resolve-ToolCommand -Name "qemu-img"
    $Adb = Resolve-ToolCommand -Name "adb"
    $Curl = Resolve-ToolCommand -Name "curl.exe"
    $Tar = Resolve-ToolCommand -Name "tar.exe"
    $WebView2Ready = Test-WebView2Runtime

    $BootstrapApplied = Request-DependencyBootstrap `
        -NeedsQemu ($null -eq $Qemu -or $null -eq $QemuImg) `
        -NeedsAdb ($null -eq $Adb) `
        -NeedsWebView2 (-not $WebView2Ready) `
        -NeedsOpenVpn $false

    if ($BootstrapApplied) {
        Initialize-ToolPaths
        $Qemu = Resolve-ToolCommand -Name "qemu-system-x86_64"
        $QemuImg = Resolve-ToolCommand -Name "qemu-img"
        $Adb = Resolve-ToolCommand -Name "adb"
        $Curl = Resolve-ToolCommand -Name "curl.exe"
        $Tar = Resolve-ToolCommand -Name "tar.exe"
        $WebView2Ready = Test-WebView2Runtime
    }

    $Tapctl = Resolve-ToolCommand -Name "tapctl.exe"
    if ($null -eq $Tapctl) {
        $OpenVpnTapctl = Join-Path $env:ProgramFiles "OpenVPN\bin\tapctl.exe"
        if (Test-Path -LiteralPath $OpenVpnTapctl -PathType Leaf) {
            $Tapctl = [pscustomobject]@{ Source = $OpenVpnTapctl }
        }
    }

    $ManagedNetworkProbe = if ($null -ne $Qemu) {
        [pscustomobject]@{ Ready = $true; Detail = "QEMU User NAT hazir; TAP/OpenVPN gerekmez." }
    } else {
        [pscustomobject]@{ Ready = $false; Detail = "qemu-system-x86_64 bulunamadi." }
    }

    $RustHost = if ($null -ne $Rustc) { Get-RustHost -RustcPath $Rustc.Source } else { $null }
    $WhpxFeatureState = Get-WindowsHypervisorPlatformState
    $FirmwareVirtualizationState = Get-FirmwareVirtualizationState
    $HypervisorPresentState = Get-HypervisorPresentState
    $WhpxRuntimeProbe = if ($null -ne $Qemu -and (Test-QemuWhpx -QemuPath $Qemu.Source) -and $WhpxFeatureState -eq "ENABLED") {
        Test-QemuWhpxRuntime -QemuPath $Qemu.Source
    } else {
        [pscustomobject]@{ Ready = $false; Detail = "WHPX runtime probe prerequisite eksik." }
    }

    Write-Host ""
    Write-Host "TurkuazVM Launcher v$WorkspacePackageVersion - Preflight"
    Write-Host ""
    Write-Status -Name "Visual Studio Build Env" -Status $(if ($VsLoaded) { "OK" } else { "NOT_FOUND" })
    Write-Status -Name "rustup" -Status $(if ($null -ne $Rustup) { "OK" } else { "OPTIONAL_NOT_FOUND" })
    Write-Status -Name "Rust toolchain" -Status $(if ($RustToolchainProbe.Ready) { $RequiredRustToolchain } else { "FAILED" })
    Write-Status -Name "rustc" -Status $(if ($null -ne $Rustc) { "OK" } else { "NOT_FOUND" })
    Write-Status -Name "cargo" -Status $(if ($null -ne $Cargo) { "OK" } else { "NOT_FOUND" })
    Write-Status -Name "Rust host" -Status $(if ($null -ne $RustHost) { $RustHost } else { "UNKNOWN" })
    Write-Status -Name "qemu-system-x86_64" -Status $(if ($null -ne $Qemu) { "OK" } else { "NOT_FOUND" })
    Write-Status -Name "qemu-img" -Status $(if ($null -ne $QemuImg) { "OK" } else { "NOT_FOUND" })
    Write-Status -Name "Windows Hypervisor Platform" -Status $WhpxFeatureState
    Write-Status -Name "Firmware Virtualization" -Status $FirmwareVirtualizationState
    Write-Status -Name "Windows Hypervisor Present" -Status $HypervisorPresentState
    Write-Status -Name "QEMU WHPX Runtime" -Status $(if ($WhpxRuntimeProbe.Ready) { "OK" } else { "FAILED" })
    Write-Status -Name "adb" -Status $(if ($null -ne $Adb) { "OK" } else { "OPTIONAL_NOT_FOUND" })
    Write-Status -Name "curl.exe" -Status $(if ($null -ne $Curl) { "OK" } else { "NOT_FOUND" })
    Write-Status -Name "tar.exe" -Status $(if ($null -ne $Tar) { "OK" } else { "NOT_FOUND" })
    Write-Status -Name "WebView2 Runtime" -Status $(if ($WebView2Ready) { "OK" } else { "NOT_FOUND" })
    Write-Status -Name "Advanced TAP tapctl" -Status $(if ($null -ne $Tapctl) { "OPTIONAL_OK" } else { "OPTIONAL_NOT_FOUND" })
    Write-Status -Name "Turkuaz NAT Runtime" -Status $(if ($ManagedNetworkProbe.Ready) { "OK" } else { "FAILED" })
    Write-Status -Name "config/turkuazvm.yml" -Status $(if (Test-Path -LiteralPath $ConfigPath -PathType Leaf) { "OK" } else { "NOT_FOUND" })

    if ($WhpxFeatureState -eq "DISABLED") {
        Write-Host ""
        $Answer = Read-Host "Windows Hypervisor Platform kapali. Yonetici izniyle etkinlestirilsin mi? [E/H]"
        if ($Answer -match '^(?i:e|evet|y|yes)$') {
            Enable-TurkuazVmWindowsHypervisorPlatform
            Write-Host ""
            Write-Host "WINDOWS_RESTART_REQUIRED"
            Write-Host "Windows Hypervisor Platform etkinlestirildi. Bilgisayari yeniden baslatip TurkuazVM-Start.cmd dosyasini tekrar calistir."
            throw "WINDOWS_RESTART_REQUIRED"
        }
    }

    $Failures = New-Object System.Collections.Generic.List[string]
    if (-not $RustToolchainProbe.Ready) { $Failures.Add("Rust toolchain hazirlanamadi: $($RustToolchainProbe.Detail)") }
    if ($null -eq $Rustc) { $Failures.Add("Rust compiler bulunamadi. rustup veya Rust toolchain kurulumu gerekli.") }
    if ($null -eq $Cargo) { $Failures.Add("Cargo bulunamadi. Rust toolchain kurulumu gerekli.") }
    if ($null -eq $Qemu) { $Failures.Add("qemu-system-x86_64 bulunamadi. Launcher WinGet bootstrap ile QEMU kurabilir.") }
    if ($null -eq $QemuImg) { $Failures.Add("qemu-img bulunamadi. Launcher WinGet bootstrap ile QEMU kurabilir.") }
    if ($null -eq $Curl) { $Failures.Add("curl.exe bulunamadi. Android otomatik image kurulumu icin Windows curl gereklidir.") }
    if ($null -eq $Tar) { $Failures.Add("tar.exe bulunamadi. Android otomatik image arsivlerini dogrulamak ve acmak icin Windows tar gereklidir.") }
    if (-not $WebView2Ready) { $Failures.Add("WebView2 Runtime bulunamadi. Desktop UI icin gereklidir.") }
    if (-not $ManagedNetworkProbe.Ready) { $Failures.Add("Turkuaz NAT runtime probe basarisiz: $($ManagedNetworkProbe.Detail)") }
    if (-not (Test-Path -LiteralPath $ConfigPath -PathType Leaf)) { $Failures.Add("config/turkuazvm.yml bulunamadi.") }

    if ($null -ne $Rustc -and -not (Test-RustVersion -RustcPath $Rustc.Source -RequiredVersion $RequiredRustToolchain)) {
        $Failures.Add("Rust surumu policy ile uyusmuyor. Gerekli toolchain: $RequiredRustToolchain")
    }

    if ($RustHost -like "*-pc-windows-msvc" -and -not $VsLoaded -and $null -eq (Resolve-ToolCommand -Name "link.exe")) {
        $Failures.Add("MSVC Rust host icin Visual Studio C++ Build Tools bulunamadi.")
    }

    if ($null -ne $Qemu -and -not (Test-QemuWhpx -QemuPath $Qemu.Source)) {
        $Failures.Add("Kurulu QEMU WHPX accelerator sunmuyor. Windows build icin WHPX gerekli.")
    }

    if ($WhpxFeatureState -eq "DISABLED" -or $WhpxFeatureState -eq "ABSENT") {
        $Failures.Add("Windows Hypervisor Platform etkin degil. WHPX hizlandirma icin gerekli.")
    }

    if ($null -ne $Qemu -and $WhpxFeatureState -eq "ENABLED" -and -not $WhpxRuntimeProbe.Ready) {
        $Failures.Add("QEMU WHPX runtime probe basarisiz: $($WhpxRuntimeProbe.Detail)")
    }

    if ($Failures.Count -gt 0) {
        Write-Host ""
        Write-Host "BASLATMA ENGELLENDI"
        foreach ($Failure in $Failures) {
            Write-Host "- $Failure"
        }
        throw "LAUNCHER_PREFLIGHT_FAILED"
    }

    if ($null -eq $Adb) {
        Write-Host ""
        Write-Host "UYARI: adb bulunamadi. Desktop acilabilir ancak Android first-boot/Guest Agent testi tamamlanamaz."
    }

    if ($WhpxFeatureState -eq "UNKNOWN") {
        Write-Host ""
        Write-Host "UYARI: Windows Hypervisor Platform durumu okunamadi. QEMU runtime testi son karari verecek."
    }

    if ($FirmwareVirtualizationState -eq "DISABLED" -and $WhpxRuntimeProbe.Ready) {
        Write-Host ""
        Write-Host "UYARI: WMI firmware virtualization DISABLED raporladi ancak QEMU WHPX runtime testi basarili. WMI sonucu bloke edici olarak kullanilmadi."
    }

    Write-Host ""
    Write-Host "PREFLIGHT_OK"
}

function Invoke-StructureVerification {
    $Verifier = Join-Path $PSScriptRoot "verify_structure.ps1"
    if (-not (Test-Path -LiteralPath $Verifier -PathType Leaf)) {
        throw "STRUCTURE_VERIFIER_NOT_FOUND"
    }

    & $Verifier
}

function Invoke-DesktopStart {
    Push-Location $ProjectRoot
    try {
        $env:TURKUAZVM_CONFIG = $ConfigPath
        $DesktopExecutable = Join-Path $ProjectRoot "target/debug/turkuazvm-desktop.exe"
        $EngineExecutable = Join-Path $ProjectRoot "target/debug/turkuazvm-engine.exe"
        $DisplayExecutable = Join-Path $ProjectRoot "target/debug/turkuazvm-display.exe"

        $ExistingDesktop = @(
            Get-TurkuazNativeProcessesByExecutablePath -ExecutablePaths @($DesktopExecutable)
        ) | Select-Object -First 1
        if ($null -ne $ExistingDesktop) {
            Write-Host ""
            Write-Host "TurkuazVM Desktop bu workspace icin zaten calisiyor; yeniden build yapilmadi."
            Write-Host "DESKTOP_ALREADY_RUNNING pid=$($ExistingDesktop.ProcessId)"
            return
        }

        $StaleRuntimeProcesses = @(
            Stop-TurkuazNativeProcessesByExecutablePath `
                -ExecutablePaths @($EngineExecutable, $DisplayExecutable) `
                -TimeoutMs 8000
        )
        if ($StaleRuntimeProcesses.Count -gt 0) {
            $StoppedIds = @($StaleRuntimeProcesses | ForEach-Object { $_.ProcessId }) -join ","
            Write-Host "Stale Engine/Display processleri kapatildi: pid=$StoppedIds"
        }

        Wait-TurkuazNativeFileRelease `
            -Paths @($DesktopExecutable, $EngineExecutable, $DisplayExecutable) `
            -TimeoutMs 8000

        Write-Host ""
        Write-Host "TurkuazVM build basliyor..."
        Invoke-TurkuazNativeChecked -FilePath "cargo" -Arguments @(
            "build",
            "-p", "turkuazvm-engine",
            "-p", "turkuazvm-display",
            "-p", "turkuazvm-desktop"
        )

        if (-not (Test-Path -LiteralPath $DesktopExecutable -PathType Leaf)) {
            throw "DESKTOP_EXECUTABLE_NOT_FOUND: $DesktopExecutable"
        }

        Write-Host ""
        Write-Host "TurkuazVM Desktop baslatiliyor..."
        $DesktopProcess = Start-Process `
            -FilePath $DesktopExecutable `
            -WorkingDirectory $ProjectRoot `
            -PassThru

        if ($null -eq $DesktopProcess) {
            throw "DESKTOP_PROCESS_START_FAILED"
        }

        Start-Sleep -Milliseconds 750
        $DesktopProcess.Refresh()
        if ($DesktopProcess.HasExited) {
            throw "DESKTOP_PROCESS_EARLY_EXIT: exit=$($DesktopProcess.ExitCode)"
        }

        Write-Host "DESKTOP_STARTED pid=$($DesktopProcess.Id)"
    }
    finally {
        Pop-Location
    }
}

New-Item -ItemType Directory -Path $LogRoot -Force | Out-Null
$Timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$LogPath = Join-Path $LogRoot "launcher-$Timestamp.log"

try {
    Start-Transcript -Path $LogPath -Force | Out-Null
    $Script:TranscriptStarted = $true

    Invoke-Preflight

    Invoke-StructureVerification
    Invoke-DesktopStart
}
catch {
    Write-Host ""
    Write-Host "TURKUAZVM_LAUNCH_FAILED"
    Write-Host $_.Exception.Message
    Write-Host "Log: $LogPath"
    if (-not $NoPause) {
        Write-Host ""
        Read-Host "Kapatmak icin Enter"
    }
    exit 1
}
finally {
    if ($Script:TranscriptStarted) {
        Stop-Transcript | Out-Null
    }
}

exit 0
