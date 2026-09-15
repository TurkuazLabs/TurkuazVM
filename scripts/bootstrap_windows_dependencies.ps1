# 📄 Dosya Yolu: /turkuazvm/scripts/bootstrap_windows_dependencies.ps1
# 📌 Amac: Windows test makinesinde eksik TurkuazVM runtime bagimliliklarini kontrollu olarak hazirlar
# 📌 Modul - PowerShell
# Version: 0.37.1
# Aciklama: WinGet uzerinden QEMU, OpenVPN TAP, Android Platform Tools ve WebView2 kurulumunu yonetir ve arac yollarini mevcut prosese ekler
# Bagimli Oldugu Katman: Tool

$Script:QemuPackageId = "SoftwareFreedomConservancy.QEMU"
$Script:AdbPackageId = "Google.PlatformTools"
$Script:WebView2PackageId = "Microsoft.EdgeWebView2Runtime"
$Script:OpenVpnPackageId = "OpenVPNTechnologies.OpenVPN"
$Script:WingetSource = "winget"
$Script:QemuProgramDirectory = "qemu"
$Script:WinGetLinksRelativePath = "Microsoft/WinGet/Links"
$Script:WinGetPackagesRelativePath = "Microsoft/WinGet/Packages"

function Add-TurkuazVmBootstrapPath {
    param([Parameter(Mandatory = $true)][string]$Path)

    if ([string]::IsNullOrWhiteSpace($Path) -or -not (Test-Path -LiteralPath $Path -PathType Container)) {
        return
    }

    $Entries = $env:PATH -split ";"
    if ($Entries -notcontains $Path) {
        $env:PATH = "$Path;$env:PATH"
    }
}

function Initialize-TurkuazVmBootstrapPaths {
    Add-TurkuazVmBootstrapPath -Path (Join-Path $env:ProgramFiles $Script:QemuProgramDirectory)
    Add-TurkuazVmBootstrapPath -Path (Join-Path $env:ProgramFiles "OpenVPN\bin")
    if (-not [string]::IsNullOrWhiteSpace(${env:ProgramFiles(x86)})) {
        Add-TurkuazVmBootstrapPath -Path (Join-Path ${env:ProgramFiles(x86)} $Script:QemuProgramDirectory)
    }

    $WingetLinks = Join-Path $env:LOCALAPPDATA $Script:WinGetLinksRelativePath
    Add-TurkuazVmBootstrapPath -Path $WingetLinks

    $WingetPackages = Join-Path $env:LOCALAPPDATA $Script:WinGetPackagesRelativePath
    if (Test-Path -LiteralPath $WingetPackages -PathType Container) {
        $Adb = Get-ChildItem -LiteralPath $WingetPackages -Filter "adb.exe" -File -Recurse -ErrorAction SilentlyContinue |
            Where-Object { $_.FullName -like "*Google.PlatformTools*" } |
            Select-Object -First 1
        if ($null -ne $Adb) {
            Add-TurkuazVmBootstrapPath -Path $Adb.DirectoryName
        }
    }
}

function Test-TurkuazVmWingetAvailable {
    return $null -ne (Get-Command "winget.exe" -ErrorAction SilentlyContinue)
}

function Invoke-TurkuazVmWingetInstall {
    param(
        [Parameter(Mandatory = $true)][string]$PackageId,
        [Parameter(Mandatory = $true)][string]$DisplayName
    )

    $Winget = Get-Command "winget.exe" -ErrorAction SilentlyContinue
    if ($null -eq $Winget) {
        throw "WINDOWS_PACKAGE_MANAGER_NOT_FOUND"
    }

    Write-Host ""
    Write-Host "$DisplayName kuruluyor..."
    try {
        Invoke-TurkuazNativeChecked `
            -FilePath $Winget.Source `
            -Arguments @("install", "--id", $PackageId, "--exact", "--source", $Script:WingetSource, "--accept-package-agreements", "--accept-source-agreements", "--silent")
    }
    catch {
        throw "WINDOWS_DEPENDENCY_INSTALL_FAILED: $PackageId - $($_.Exception.Message)"
    }

    Initialize-TurkuazVmBootstrapPaths
}

function Install-TurkuazVmWindowsDependencies {
    param(
        [bool]$InstallQemu,
        [bool]$InstallAdb,
        [bool]$InstallWebView2,
        [bool]$InstallOpenVpn = $false
    )

    if (-not $InstallQemu -and -not $InstallAdb -and -not $InstallWebView2 -and -not $InstallOpenVpn) {
        return
    }

    if (-not (Test-TurkuazVmWingetAvailable)) {
        throw "WINDOWS_PACKAGE_MANAGER_NOT_FOUND"
    }

    if ($InstallQemu) {
        Invoke-TurkuazVmWingetInstall -PackageId $Script:QemuPackageId -DisplayName "QEMU"
    }
    if ($InstallAdb) {
        Invoke-TurkuazVmWingetInstall -PackageId $Script:AdbPackageId -DisplayName "Android Platform Tools (ADB)"
    }
    if ($InstallWebView2) {
        Invoke-TurkuazVmWingetInstall -PackageId $Script:WebView2PackageId -DisplayName "Microsoft Edge WebView2 Runtime"
    }
    if ($InstallOpenVpn) {
        Invoke-TurkuazVmWingetInstall -PackageId $Script:OpenVpnPackageId -DisplayName "OpenVPN TAP Adapter"
    }
}


function Enable-TurkuazVmWindowsHypervisorPlatform {
    $Arguments = @(
        "/online",
        "/Enable-Feature",
        "/FeatureName:HypervisorPlatform",
        "/All",
        "/NoRestart"
    )

    $Process = Start-Process -FilePath "dism.exe" -ArgumentList $Arguments -Verb RunAs -Wait -PassThru
    if ($null -eq $Process -or $Process.ExitCode -ne 0) {
        throw "WINDOWS_HYPERVISOR_PLATFORM_ENABLE_FAILED"
    }
}
