# 📄 Dosya Yolu: /turkuazvm/scripts/doctor.ps1
# 📌 Amac: Windows gelistirme makinesinde Rust, Cargo, QEMU, Android ADB, Gaming GPU ve Android Image dosyalarini kontrol eder
# 📌 Modul - PowerShell
# Version: 0.28.0
# Aciklama: Kurulum yapmadan virtualization, Android Runtime, Gaming GPU ve Android Image developer durumunu raporlar
# Bagimli Oldugu Katman: Tool

$ErrorActionPreference = "SilentlyContinue"

function Test-CommandExists {
    param([Parameter(Mandatory = $true)][string]$Name)

    return $null -ne (Get-Command $Name -ErrorAction SilentlyContinue)
}

$Checks = @(
    @{ Name = "rustc"; Required = $true },
    @{ Name = "cargo"; Required = $true },
    @{ Name = "qemu-system-x86_64"; Required = $true },
    @{ Name = "qemu-img"; Required = $true },
    @{ Name = "adb"; Required = $false },
    @{ Name = "vulkaninfo"; Required = $false }
)

Write-Host "TurkuazVM Doctor v0.28.0"
Write-Host ""

$HasFailure = $false

foreach ($Check in $Checks) {
    $Exists = Test-CommandExists -Name $Check.Name
    $Status = if ($Exists) { "OK" } else { "NOT_FOUND" }

    Write-Host ("{0,-24} {1}" -f $Check.Name, $Status)

    if ($Check.Required -and -not $Exists) {
        $HasFailure = $true
    }
}


$ProjectRoot = Split-Path -Parent $PSScriptRoot
$AndroidImageScripts = @(
    "guest/android-image/scripts/prepare_aosp_source.sh",
    "guest/android-image/scripts/build_turkuaz_android_image.sh"
)

foreach ($RelativePath in $AndroidImageScripts) {
    $AbsolutePath = Join-Path $ProjectRoot $RelativePath
    $Status = if (Test-Path -LiteralPath $AbsolutePath -PathType Leaf) { "OK" } else { "NOT_FOUND" }
    Write-Host ("{0,-24} {1}" -f ("image:" + (Split-Path -Leaf $RelativePath)), $Status)
    if ($Status -ne "OK") {
        $HasFailure = $true
    }
}

Write-Host ("{0,-24} {1}" -f "aosp-build-host", "LINUX_64_REQUIRED")
Write-Host ("{0,-24} {1}" -f "aosp-free-disk", "400_GIB_MIN")

if (Test-CommandExists -Name "qemu-system-x86_64") {
    $DeviceHelp = (& qemu-system-x86_64 -device help 2>&1 | Out-String)
    foreach ($GpuDevice in @("virtio-gpu", "virtio-gpu-gl", "virtio-gpu-rutabaga")) {
        $Status = if ($DeviceHelp.Contains($GpuDevice)) { "AVAILABLE" } else { "NOT_FOUND" }
        Write-Host ("{0,-24} {1}" -f ("qemu:" + $GpuDevice), $Status)
    }
}

if ($HasFailure) {
    exit 1
}

exit 0
