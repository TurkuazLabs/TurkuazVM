# 📄 Dosya Yolu: /turkuazvm/scripts/verify_structure.ps1
# 📌 Amac: TurkuazVM dosya header, version, workspace ve dependency sinirlarini dogrular
# 📌 Modul - PowerShell
# Version: 0.41.2
# Aciklama: Cargo oncesi workspace kontratlarini fail-closed dogrular; Baglanti Merkezi, VM Kontrol Merkezi ve mevcut Android SDK runtime kontratlarini korur
# Bagimli Oldugu Katman: Tool

$ErrorActionPreference = "Stop"

$ProjectRoot = Split-Path -Parent $PSScriptRoot
$TextExtensions = @(".rs", ".toml", ".md", ".ps1", ".yml", ".yaml", ".html", ".css", ".js", ".json5", ".sh", ".service", ".example", ".gitignore", ".cmd", ".kt", ".xml", ".bp", ".mk", ".py")
$RequiredHeaderTokens = @(
    "Dosya Yolu:",
    "Amac:",
    "Modul -",
    "Version:",
    "Aciklama:",
    "Bagimli Oldugu Katman:"
)
$VersionHeaderPattern = 'Version:\s+\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?(?:\s|$)'

$RuntimeDataRoot = Join-Path $ProjectRoot "data"
$TargetRoot = Join-Path $ProjectRoot "target"

function Convert-ToProjectRelativePath {
    param([Parameter(Mandatory = $true)][string]$FullName)

    $RelativePath = $FullName.Substring($ProjectRoot.Length) -replace '\\', '/'
    if (-not $RelativePath.StartsWith("/")) {
        $RelativePath = "/$RelativePath"
    }

    return $RelativePath
}

$PathNormalizationProbe = Convert-ToProjectRelativePath -FullName (Join-Path $ProjectRoot ".gitignore")
if ($PathNormalizationProbe -ne "/.gitignore") {
    throw "WINDOWS_PATH_NORMALIZATION_INVALID: $PathNormalizationProbe"
}

$Files = Get-ChildItem -Path $ProjectRoot -Recurse -File | Where-Object {
    -not $_.FullName.StartsWith($TargetRoot) -and
    -not $_.FullName.StartsWith($RuntimeDataRoot) -and
    ($TextExtensions -contains $_.Extension -or $_.Name -eq ".gitignore")
}

foreach ($File in $Files) {
    $Content = Get-Content -LiteralPath $File.FullName -Raw
    $Header = (($Content -split "`r?`n") | Select-Object -First 7) -join "`n"
    $RelativePath = Convert-ToProjectRelativePath -FullName $File.FullName
    $ExpectedPath = "/turkuazvm$RelativePath"

    if (-not $Header.Contains($ExpectedPath)) {
        throw "HEADER_PATH_INVALID: $RelativePath"
    }

    foreach ($Token in $RequiredHeaderTokens) {
        if (-not $Header.Contains($Token)) {
            throw "HEADER_TOKEN_MISSING: $RelativePath -> $Token"
        }
    }

    if ($Header -notmatch $VersionHeaderPattern) {
        throw "HEADER_VERSION_INVALID: $RelativePath"
    }
}

$RequiredWorkspacePaths = @(
    "apps/engine",
    "apps/desktop",
    "apps/display",
    "crates/android",
    "crates/android-image",
    "crates/core",
    "crates/engine-api",
    "crates/guest",
    "crates/gpu",
    "crates/gaming-input",
    "crates/game-catalog",
    "crates/guest-catalog",
    "crates/guest-agent-protocol",
    "crates/platform",
    "crates/qemu",
    "crates/repositories",
    "crates/storage",
    "crates/transport",
    "config",
    "config/guest-catalog.yml",
    "config/download-sources.yml",
    "docs",
    "packages",
    "guest/android-agent",
    "guest/android-image",
    "scripts",
    "scripts/start_turkuazvm.ps1",
    "scripts/bootstrap_windows_dependencies.ps1",
    "TurkuazVM-Start.cmd",
    "apps/desktop/src-tauri/icons/icon.ico"
)

foreach ($RelativePath in $RequiredWorkspacePaths) {
    if (-not (Test-Path -LiteralPath (Join-Path $ProjectRoot $RelativePath))) {
        throw "REQUIRED_PATH_MISSING: $RelativePath"
    }
}

$WorkspaceCargoContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "Cargo.toml") -Raw
if (-not $WorkspaceCargoContent.Contains('dead_code = "deny"')) {
    throw "BUILD_HYGIENE_DEAD_CODE_DENY_MISSING"
}

$ObsoleteControllerPaths = @(
    "apps/engine/src/controllers/engine_controller.rs",
    "apps/engine/src/controllers/guest_boot_controller.rs",
    "apps/engine/src/controllers/network_controller.rs",
    "apps/engine/src/controllers/storage_controller.rs",
    "apps/engine/src/controllers/vm_controller.rs"
)
foreach ($RelativePath in $ObsoleteControllerPaths) {
    if (Test-Path -LiteralPath (Join-Path $ProjectRoot $RelativePath)) {
        throw "BUILD_HYGIENE_OBSOLETE_CONTROLLER_PRESENT: $RelativePath"
    }
}

$EngineConfigErrorContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/engine/src/config/engine_config.rs") -Raw
foreach ($Token in @("impl fmt::Display for EngineConfigError", "impl std::error::Error for EngineConfigError")) {
    if (-not $EngineConfigErrorContent.Contains($Token)) {
        throw "BUILD_HYGIENE_ENGINE_CONFIG_ERROR_CONTRACT_MISSING: $Token"
    }
}

$DesktopConfigErrorContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/desktop/src-tauri/src/config/desktop_config.rs") -Raw
foreach ($Token in @("impl fmt::Display for DesktopConfigError", "impl std::error::Error for DesktopConfigError")) {
    if (-not $DesktopConfigErrorContent.Contains($Token)) {
        throw "BUILD_HYGIENE_DESKTOP_CONFIG_ERROR_CONTRACT_MISSING: $Token"
    }
}

$EngineApiClientErrorContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/desktop/src-tauri/src/tools/engine_api_client_tool.rs") -Raw
foreach ($Token in @("impl fmt::Display for EngineApiClientError", "impl std::error::Error for EngineApiClientError")) {
    if (-not $EngineApiClientErrorContent.Contains($Token)) {
        throw "BUILD_HYGIENE_ENGINE_API_ERROR_CONTRACT_MISSING: $Token"
    }
}

$EngineApiServerContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/engine/src/tools/engine_api_server_tool.rs") -Raw
if ($EngineApiServerContent.Contains("peer_addr")) {
    throw "BUILD_HYGIENE_UNUSED_PEER_ADDR_PRESENT"
}

$AndroidImageApplicationContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/engine/src/services/android_image_application_service.rs") -Raw
if ($AndroidImageApplicationContent.Contains("pub fn get(&self, image_id")) {
    throw "BUILD_HYGIENE_UNUSED_ANDROID_IMAGE_GET_PRESENT"
}

$EngineConsoleViewContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/engine/src/views/console_view.rs") -Raw
if ($EngineConsoleViewContent.Contains("render_host_capability")) {
    throw "BUILD_HYGIENE_UNUSED_HOST_RENDERER_PRESENT"
}

$CoreRoot = Join-Path $ProjectRoot "crates/core"
$ForbiddenCoreTokens = @(
    "turkuazvm_qemu",
    "turkuazvm_android",
    "turkuazvm_guest",
    "turkuazvm_gpu",
    "turkuazvm_storage",
    "turkuazvm_engine_api",
    "turkuazvm_transport",
    "rustls",
    "serde_json",
    "serde_yaml",
    "serde_yaml_ng",
    "TcpStream",
    "std::process::Command",
    "tauri"
)

foreach ($CoreFile in (Get-ChildItem -Path $CoreRoot -Recurse -Filter "*.rs" -File)) {
    $Content = Get-Content -LiteralPath $CoreFile.FullName -Raw
    foreach ($Token in $ForbiddenCoreTokens) {
        if ($Content.Contains($Token)) {
            throw "CORE_DEPENDENCY_LEAK: $($CoreFile.FullName) -> $Token"
        }
    }
}

$GuestCatalogRoot = Join-Path $ProjectRoot "crates/guest-catalog"
$ForbiddenGuestCatalogTokens = @(
    "turkuazvm_core",
    "turkuazvm_qemu",
    "turkuazvm_android",
    "turkuazvm_guest",
    "turkuazvm_engine_api",
    "serde_json",
    "serde_yaml",
    "serde_yaml_ng",
    "std::process::Command",
    "TcpStream",
    "tauri"
)
foreach ($GuestCatalogFile in (Get-ChildItem -Path $GuestCatalogRoot -Recurse -Filter "*.rs" -File)) {
    $Content = Get-Content -LiteralPath $GuestCatalogFile.FullName -Raw
    foreach ($Token in $ForbiddenGuestCatalogTokens) {
        if ($Content.Contains($Token)) {
            throw "GUEST_CATALOG_DEPENDENCY_LEAK: $($GuestCatalogFile.FullName) -> $Token"
        }
    }
}

$AndroidRoot = Join-Path $ProjectRoot "crates/android"
$ForbiddenAndroidTokens = @(
    "turkuazvm_core",
    "turkuazvm_qemu",
    "turkuazvm_guest",
    "turkuazvm_gpu",
    "turkuazvm_engine_api",
    "serde_json",
    "serde_yaml",
    "serde_yaml_ng",
    "std::process::Command",
    "tauri"
)

foreach ($AndroidFile in (Get-ChildItem -Path $AndroidRoot -Recurse -Filter "*.rs" -File)) {
    $Content = Get-Content -LiteralPath $AndroidFile.FullName -Raw
    foreach ($Token in $ForbiddenAndroidTokens) {
        if ($Content.Contains($Token)) {
            throw "ANDROID_DEPENDENCY_LEAK: $($AndroidFile.FullName) -> $Token"
        }
    }
}


$GamingInputRoot = Join-Path $ProjectRoot "crates/gaming-input"
$ForbiddenGamingInputTokens = @(
    "turkuazvm_core",
    "turkuazvm_qemu",
    "turkuazvm_android",
    "turkuazvm_guest",
    "turkuazvm_engine_api",
    "serde_json",
    "serde_yaml",
    "serde_yaml_ng",
    "std::process::Command",
    "tauri"
)

foreach ($GamingInputFile in (Get-ChildItem -Path $GamingInputRoot -Recurse -Filter "*.rs" -File)) {
    $Content = Get-Content -LiteralPath $GamingInputFile.FullName -Raw
    foreach ($Token in $ForbiddenGamingInputTokens) {
        if ($Content.Contains($Token)) {
            throw "GAMING_INPUT_DEPENDENCY_LEAK: $($GamingInputFile.FullName) -> $Token"
        }
    }
}

$GameCatalogRoot = Join-Path $ProjectRoot "crates/game-catalog"
$ForbiddenGameCatalogTokens = @(
    "turkuazvm_core",
    "turkuazvm_qemu",
    "turkuazvm_android",
    "turkuazvm_guest",
    "turkuazvm_gaming_input",
    "turkuazvm_engine_api",
    "serde_json",
    "serde_yaml",
    "serde_yaml_ng",
    "std::process::Command",
    "tauri"
)
foreach ($GameCatalogFile in (Get-ChildItem -Path $GameCatalogRoot -Recurse -Filter "*.rs" -File)) {
    $Content = Get-Content -LiteralPath $GameCatalogFile.FullName -Raw
    foreach ($Token in $ForbiddenGameCatalogTokens) {
        if ($Content.Contains($Token)) { throw "GAME_CATALOG_DEPENDENCY_LEAK: $($GameCatalogFile.FullName) -> $Token" }
    }
}

$GpuRoot = Join-Path $ProjectRoot "crates/gpu"
$ForbiddenGpuTokens = @(
    "turkuazvm_core",
    "turkuazvm_qemu",
    "turkuazvm_guest",
    "turkuazvm_engine_api",
    "serde_json",
    "serde_yaml",
    "serde_yaml_ng",
    "std::process::Command",
    "tauri"
)

foreach ($GpuFile in (Get-ChildItem -Path $GpuRoot -Recurse -Filter "*.rs" -File)) {
    $Content = Get-Content -LiteralPath $GpuFile.FullName -Raw
    foreach ($Token in $ForbiddenGpuTokens) {
        if ($Content.Contains($Token)) {
            throw "GPU_DEPENDENCY_LEAK: $($GpuFile.FullName) -> $Token"
        }
    }
}

$DesktopTauriConfig = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/desktop/src-tauri/tauri.conf.json5") -Raw
if (-not $DesktopTauriConfig.Contains('"icons/icon.ico"')) {
    throw "DESKTOP_TAURI_ICON_CONFIG_MISSING: icons/icon.ico"
}

$DesktopIconPath = Join-Path $ProjectRoot "apps/desktop/src-tauri/icons/icon.ico"
$DesktopIconBytes = [System.IO.File]::ReadAllBytes($DesktopIconPath)
if ($DesktopIconBytes.Length -lt 6) {
    throw "DESKTOP_ICON_INVALID: TOO_SMALL"
}
$DesktopIconReserved = [BitConverter]::ToUInt16($DesktopIconBytes, 0)
$DesktopIconType = [BitConverter]::ToUInt16($DesktopIconBytes, 2)
$DesktopIconCount = [BitConverter]::ToUInt16($DesktopIconBytes, 4)
if ($DesktopIconReserved -ne 0 -or $DesktopIconType -ne 1 -or $DesktopIconCount -lt 1) {
    throw "DESKTOP_ICON_INVALID: ICO_HEADER"
}

$DesktopCargo = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/desktop/src-tauri/Cargo.toml") -Raw
foreach ($Token in @("turkuazvm-core", "turkuazvm-qemu", "turkuazvm-storage", "turkuazvm-platform", "turkuazvm-repositories", "turkuazvm-android", "turkuazvm-gpu", "turkuazvm-gaming-input", "turkuazvm-game-catalog", "turkuazvm-guest-catalog", "turkuazvm-android-image")) {
    if ($DesktopCargo.Contains($Token)) {
        throw "DESKTOP_DIRECT_DEPENDENCY_LEAK: $Token"
    }
}

$RequiredFoundationFiles = @(
    "crates/core/src/ports/storage_port.rs",
    "crates/storage/src/tools/qemu_img_tool.rs",
    "crates/repositories/src/repositories/yaml_vm_repository.rs",
    "crates/core/src/ports/network_port.rs",
    "crates/platform/src/tools/native_network_tool.rs",
    "crates/core/src/domain/guest_boot.rs",
    "apps/display/src/main.rs",
    "apps/display/src/tools/rfb_client_tool.rs",
    "crates/android/src/domain/runtime_profile.rs",
    "crates/android/src/services/android_runtime_service.rs",
    "crates/android/src/services/android_package_service.rs",
    "crates/android/src/services/android_input_service.rs",
    "crates/guest/src/tools/adb_runtime_tool.rs",
    "crates/repositories/src/repositories/yaml_android_profile_repository.rs",
    "apps/engine/src/services/android_application_service.rs",
    "docs/ANDROID_RUNTIME.md",
    "docs/architecture/android-runtime.md",
    "docs/examples/android.yml",
    "crates/gpu/src/domain/capability.rs",
    "crates/gpu/src/domain/profile.rs",
    "crates/gpu/src/ports/host_gpu_probe_port.rs",
    "crates/gpu/src/ports/hypervisor_gpu_probe_port.rs",
    "crates/gpu/src/services/gaming_gpu_service.rs",
    "crates/platform/src/tools/native_gpu_probe_tool.rs",
    "crates/qemu/src/tools/qemu_gpu_probe_tool.rs",
    "docs/GAMING_GPU.md",
    "docs/architecture/gaming-gpu.md",
    "docs/examples/gpu.yml",
    "crates/gaming-input/src/domain/profile.rs",
    "crates/gaming-input/src/domain/event.rs",
    "crates/gaming-input/src/domain/plan.rs",
    "crates/gaming-input/src/ports/game_input_profile_repository_port.rs",
    "crates/gaming-input/src/ports/host_gamepad_port.rs",
    "crates/gaming-input/src/services/game_input_profile_service.rs",
    "crates/gaming-input/src/services/gaming_input_translator_service.rs",
    "crates/repositories/src/repositories/yaml_game_input_profile_repository.rs",
    "apps/engine/src/services/gaming_input_application_service.rs",
    "apps/display/src/tools/gaming_input_bridge_tool.rs",
    "apps/display/src/tools/gamepad_input_tool.rs",
    "crates/game-catalog/src/domain/game.rs",
    "crates/game-catalog/src/services/game_catalog_service.rs",
    "crates/repositories/src/repositories/yaml_game_catalog_repository.rs",
    "crates/guest-catalog/src/services/guest_catalog_service.rs",
    "crates/repositories/src/repositories/yaml_guest_catalog_repository.rs",
    "crates/guest-agent-protocol/src/lib.rs",
    "crates/android/src/ports/android_guest_agent_port.rs",
    "crates/guest/src/tools/android_guest_agent_tool.rs",
    "guest/android-agent/Android.bp",
    "guest/android-agent/AndroidManifest.xml",
    "guest/android-agent/src/com/turkuazvm/inputagent/services/TurkuazInputAccessibilityService.kt",
    "guest/android-agent/src/com/turkuazvm/inputagent/tools/GuestAgentServerTool.kt",
    "guest/android-agent/src/com/turkuazvm/inputagent/tools/TouchGestureTool.kt",
    "docs/ANDROID_GUEST_AGENT.md",
    "docs/GAME_PROFILES.md",
    "docs/architecture/game-profiles.md",
    "docs/GAMING_INPUT.md",
    "docs/architecture/gaming-input.md",
    "docs/examples/gaming-input.yml",
    "crates/android-image/src/domain/image.rs",
    "crates/android-image/src/domain/artifact.rs",
    "crates/core/src/domain/runtime_media.rs",
    "crates/core/src/commands/recover_vm_command.rs",
    "crates/android-image/src/services/android_image_service.rs",
    "crates/android-image/src/ports/android_image_repository_port.rs",
    "crates/android-image/src/ports/android_image_builder_port.rs",
    "crates/android-image/src/ports/android_image_distribution_port.rs",
    "crates/repositories/src/repositories/yaml_android_image_repository.rs",
    "crates/guest/src/tools/aosp_android_image_tool.rs",
    "crates/guest/src/tools/android_ci_distribution_tool.rs",
    "crates/guest/src/tools/android_composite_disk_tool.rs",
    "crates/guest/src/tools/android_runtime_media_tool.rs",
    "apps/engine/src/services/android_image_application_service.rs",
    "apps/desktop/src-tauri/src/tools/local_file_viewer_tool.rs",
    "apps/desktop/src-tauri/src/tools/download_settings_tool.rs",
    "guest/android-image/scripts/prepare_aosp_source.sh",
    "guest/android-image/scripts/build_turkuaz_android_image.sh",
    "guest/android-image/scripts/assemble_turkuaz_android_composite.py",
    "guest/android-image/aosp-overlay/device/turkuazvm/cuttlefish/AndroidProducts.mk",
    "guest/android-image/aosp-overlay/device/turkuazvm/cuttlefish/turkuazvm_cf_x86_64_phone.mk",
    "docs/ANDROID_IMAGE.md",
    "docs/architecture/android-image-foundation.md",
    "docs/examples/android-image.yml"
)

foreach ($RelativePath in $RequiredFoundationFiles) {
    if (-not (Test-Path -LiteralPath (Join-Path $ProjectRoot $RelativePath))) {
        throw "FOUNDATION_FILE_MISSING: $RelativePath"
    }
}

$WorkspaceManifestPath = Join-Path $ProjectRoot "Cargo.toml"
$WorkspaceVersion = $null
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
        $WorkspaceVersion = $Matches[1]
        break
    }
}
if ([string]::IsNullOrWhiteSpace($WorkspaceVersion)) {
    throw "WORKSPACE_VERSION_NOT_FOUND"
}

$ReleaseStatusPath = Join-Path $ProjectRoot ("RELEASE_STATUS_v{0}.yml" -f $WorkspaceVersion)
if (-not (Test-Path -LiteralPath $ReleaseStatusPath -PathType Leaf)) {
    throw "RELEASE_STATUS_NOT_FOUND: v$WorkspaceVersion"
}
$ReleaseStatusContent = Get-Content -LiteralPath $ReleaseStatusPath -Raw

$ExpectedReleaseStatusName = "RELEASE_STATUS_v$WorkspaceVersion.yml"
$ExpectedManifestName = "MANIFEST_SHA256_v$WorkspaceVersion.txt"
$StaleReleaseStatus = @(Get-ChildItem -LiteralPath $ProjectRoot -File -Filter "RELEASE_STATUS_v*.yml" | Where-Object { $_.Name -ne $ExpectedReleaseStatusName })
if ($StaleReleaseStatus.Count -gt 0) {
    throw "STALE_RELEASE_STATUS_IN_ROOT: $($StaleReleaseStatus.Name -join ', ')"
}
$StaleReleaseManifest = @(Get-ChildItem -LiteralPath $ProjectRoot -File -Filter "MANIFEST_SHA256_v*.txt" | Where-Object { $_.Name -ne $ExpectedManifestName })
if ($StaleReleaseManifest.Count -gt 0) {
    throw "STALE_RELEASE_MANIFEST_IN_ROOT: $($StaleReleaseManifest.Name -join ', ')"
}
foreach ($Pattern in @("PATCH_INTEGRATION*.md", "VALIDATION_*.md", "RELEASE_GATE_STATUS_*.yml", "MANIFEST_SHA256_CUMULATIVE*.txt", "RECOVERY_WORK_STATUS.yml", "FULL_MANIFEST_SHA256.txt", "MIGRATION_*_SUPERSESSION_*.yml")) {
    $LegacyRootArtifact = @(Get-ChildItem -LiteralPath $ProjectRoot -File -Filter $Pattern)
    if ($LegacyRootArtifact.Count -gt 0) {
        throw "LEGACY_RELEASE_ARTIFACT_IN_ROOT: $($LegacyRootArtifact.Name -join ', ')"
    }
}
if ($ReleaseStatusContent -notmatch '(?m)^\s*engine_api_version:\s*([0-9]+)\s*$') {
    throw "RELEASE_ENGINE_API_VERSION_NOT_FOUND"
}
$ReleaseEngineApiVersion = [int]$Matches[1]
if ($ReleaseEngineApiVersion -le 0) {
    throw "RELEASE_ENGINE_API_VERSION_INVALID"
}
if ($ReleaseStatusContent -notmatch '(?m)^\s*guest_agent_protocol_version:\s*([0-9]+)\s*$') {
    throw "RELEASE_GUEST_AGENT_PROTOCOL_VERSION_NOT_FOUND"
}
$ReleaseGuestAgentProtocolVersion = [int]$Matches[1]
if ($ReleaseGuestAgentProtocolVersion -le 0) {
    throw "RELEASE_GUEST_AGENT_PROTOCOL_VERSION_INVALID"
}

$EngineApiContract = Join-Path $ProjectRoot "crates/engine-api/src/lib.rs"
$EngineApiContent = Get-Content -LiteralPath $EngineApiContract -Raw
if ($EngineApiContent -notmatch 'ENGINE_API_VERSION:\s*u16\s*=\s*([0-9]+)\s*;') {
    throw "ENGINE_API_VERSION_NOT_FOUND"
}
$EngineApiVersion = [int]$Matches[1]
if ($EngineApiVersion -ne $ReleaseEngineApiVersion) {
    throw "ENGINE_API_VERSION_INVALID: source=$EngineApiVersion release=$ReleaseEngineApiVersion"
}
foreach ($Token in @("ConfigureAndroidRuntime", "WaitAndroidReady", "InstallAndroidApk", "InjectAndroidInput", "GuestProfileDto", "GetGpuCapabilities", "GpuCapabilitiesDto", "GetGamingInputCapabilities", "ConfigureGamingInputProfile", "InjectGamingInputEvent", "GamingInputProfileDto", "GetGuestAgentStatus", "ListGameCatalog", "ListGuestCatalog", "DetectGames", "GetGameCompatibility", "ApplyGameProfile", "GameCatalogEntryDto", "GuestTemplateDto", "GameCompatibilityDto", "GameCatalogMaturityDto", "GameCatalogGpuBackendDto", "GameCompatibilityStatusDto", "ListAndroidImages", "DefineAndroidImage", "PrepareAndroidImageBuild", "RegisterAndroidImageBuild", "InstallAndroidImageDistribution", "CancelAndroidImageDistribution", "CleanupAndroidImageDistribution", "AssignAndroidImage", "GetAndroidImageAssignment", "AndroidImageDto", "AndroidImageBuildPlanDto", "AndroidImageAssignmentDto", "AndroidImageInstallProgressDto", "AndroidImageInstallStageDto", "ResizeVmDisk", "DeleteVmDisk", "UpdateVm", "DeleteVm", "ConfigureInstallerMedia", "CancelInstallerMediaDownload", "VmDiskDto", "VmNetworkDto", "bytes_per_second", "eta_seconds")) {
    if (-not $EngineApiContent.Contains($Token)) {
        throw "ENGINE_API_CONTRACT_MISSING: $Token"
    }
}

$EngineConfigPath = Join-Path $ProjectRoot "apps/engine/src/config/engine_config.rs"
$EngineConfigContent = Get-Content -LiteralPath $EngineConfigPath -Raw
if (-not $EngineConfigContent.Contains("Non-loopback Engine API requires TLS and token authentication")) {
    throw "REMOTE_SECURITY_GUARD_MISSING"
}
if (-not $EngineConfigContent.Contains("android.adb.host_ip must be loopback")) {
    throw "ANDROID_ADB_LOOPBACK_GUARD_MISSING"
}
if (-not $EngineConfigContent.Contains("android.adb port range is invalid")) {
    throw "ANDROID_ADB_PORT_GUARD_MISSING"
}
if (-not $EngineConfigContent.Contains("android.adb timeout values must be greater than zero")) {
    throw "ANDROID_ADB_TIMEOUT_GUARD_MISSING"
}
if (-not $EngineConfigContent.Contains('required_path(&config.android.package_root, "android.package_root")')) {
    throw "ANDROID_PACKAGE_ROOT_CONFIG_GUARD_MISSING"
}
foreach ($Token in @("android.guest_agent.host_ip must be loopback", "android.guest_agent port range is invalid", "android.guest_agent timeout values must be greater than zero", "android.image.distribution.minimum_free_disk_gib must be at least 16", 'required_path(&config.game_catalog.path, "game_catalog.path")', 'required_path(&config.guest_catalog.path, "guest_catalog.path")')) {
    if (-not $EngineConfigContent.Contains($Token)) { throw "ENGINE_CONFIG_GUARD_MISSING: $Token" }
}

$DisplayServiceContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/desktop/src-tauri/src/services/desktop_service.rs") -Raw
if (-not $DisplayServiceContent.Contains("Remote native display transport is not available")) {
    throw "DESKTOP_DISPLAY_LOCAL_GUARD_MISSING"
}

$DesktopConfigContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/desktop/src-tauri/src/config/desktop_config.rs") -Raw
foreach ($Token in @("long_request_timeout", "long_request_timeout_ms", "MIN_LONG_REQUEST_TIMEOUT_MS", "long_request_timeout_ms must be >= request_timeout_ms")) {
    if (-not $DesktopConfigContent.Contains($Token)) { throw "DESKTOP_LONG_TIMEOUT_CONFIG_MISSING: $Token" }
}

$DesktopApiClientContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/desktop/src-tauri/src/tools/engine_api_client_tool.rs") -Raw
foreach ($Token in @("send_with_timeout", "set_read_timeout(Some(timeout))", "set_write_timeout(Some(timeout))")) {
    if (-not $DesktopApiClientContent.Contains($Token)) { throw "DESKTOP_REQUEST_TIMEOUT_TOOL_MISSING: $Token" }
}

$EngineProcessToolContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/desktop/src-tauri/src/tools/engine_process_tool.rs") -Raw
foreach ($Token in @("EngineProcessHandle", "data/logs/engine", "diagnostic_tail", ".stdout(Stdio::from(stdout_file))", ".stderr(Stdio::from(log_file))")) {
    if (-not $EngineProcessToolContent.Contains($Token)) { throw "DESKTOP_ENGINE_SUPERVISION_TOOL_MISSING: $Token" }
}
foreach ($Token in @("engine_processes", "spawn_local_engine", "take_engine_exit_detail", "engine_not_ready_message")) {
    if (-not $DisplayServiceContent.Contains($Token)) { throw "DESKTOP_ENGINE_SUPERVISION_SERVICE_MISSING: $Token" }
}

foreach ($Token in @("uses_long_request_timeout", "EngineAction::StartVm", "EngineAction::WaitAndroidReady", "EngineAction::InstallAndroidApk")) {
    if (-not $DisplayServiceContent.Contains($Token)) { throw "DESKTOP_LONG_TIMEOUT_ROUTING_MISSING: $Token" }
}

$MainConfigContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "config/turkuazvm.yml") -Raw
foreach ($Token in @("schema_version: 22", "request_timeout_ms:", "long_request_timeout_ms:", "guest_catalog:", "path: config/guest-catalog.yml", "downloads:", "sources_path: config/download-sources.yml", "http:", "curl_binary:", "connect_timeout_seconds:", "retry_count:", "retry_delay_seconds:")) {
    if (-not $MainConfigContent.Contains($Token)) { throw "DESKTOP_MAIN_CONFIG_MISSING: $Token" }
}

$RequestTimeoutMatch = [regex]::Match($MainConfigContent, '(?m)^\s*request_timeout_ms:\s*(\d+)\s*$')
if (-not $RequestTimeoutMatch.Success) {
    throw "DESKTOP_REQUEST_TIMEOUT_MAIN_CONFIG_INVALID"
}
$LongRequestTimeoutMatch = [regex]::Match($MainConfigContent, '(?m)^\s*long_request_timeout_ms:\s*(\d+)\s*$')
if (-not $LongRequestTimeoutMatch.Success) {
    throw "DESKTOP_LONG_TIMEOUT_MAIN_CONFIG_INVALID"
}

$RequestTimeoutMs = [uint64]::Parse($RequestTimeoutMatch.Groups[1].Value)
$LongRequestTimeoutMs = [uint64]::Parse($LongRequestTimeoutMatch.Groups[1].Value)
if ($RequestTimeoutMs -eq 0) {
    throw "DESKTOP_REQUEST_TIMEOUT_MAIN_CONFIG_ZERO"
}
if ($LongRequestTimeoutMs -lt 30000) {
    throw "DESKTOP_LONG_TIMEOUT_MAIN_CONFIG_TOO_SMALL: $LongRequestTimeoutMs"
}
if ($LongRequestTimeoutMs -lt $RequestTimeoutMs) {
    throw "DESKTOP_LONG_TIMEOUT_MAIN_CONFIG_LT_REQUEST: request=$RequestTimeoutMs long=$LongRequestTimeoutMs"
}
foreach ($Token in @("default_disk:", "format: qcow2", "bus: virtio", "portable_image:", "extension: .tvmimg", "container: zip64", "private_copy_default: true", "default_network_id: turkuaz-net-01", "private_network_id: turkuaz-private-01", "default_device_model: virtio_net_pci", "default_profile: managed_nat", "managed_helper_path: scripts/network_windows_managed.ps1")) {
    if (-not $MainConfigContent.Contains($Token)) { throw "STORAGE_NETWORK_MAIN_CONFIG_MISSING: $Token" }
}

$StorageHostToolContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/storage/src/tools/host_storage_tool.rs") -Raw
foreach ($Token in @("fs2::total_space", "fs2::available_space", "StorageHostPort")) {
    if (-not $StorageHostToolContent.Contains($Token)) { throw "STORAGE_HOST_PROBE_MISSING: $Token" }
}

$AndroidDistributionToolContentEarly = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/guest/src/tools/android_ci_distribution_tool.rs") -Raw
foreach ($Token in @("fs2::available_space", "free disk path prepare failed")) {
    if (-not $AndroidDistributionToolContentEarly.Contains($Token)) { throw "ANDROID_FREE_DISK_SAFE_PROBE_MISSING: $Token" }
}
if ($AndroidDistributionToolContentEarly.Contains("System.IO.DriveInfo")) {
    throw "ANDROID_FREE_DISK_POWERSHELL_PARSER_PRESENT"
}

$StorageHostServiceContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/core/src/services/storage_host_service.rs") -Raw
foreach ($Token in @("PortableImagePolicy", "default_disk_size_gib", "private_copy_default")) {
    if (-not $StorageHostServiceContent.Contains($Token)) { throw "STORAGE_POLICY_SERVICE_MISSING: $Token" }
}

$EngineApplicationStorageNetworkContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/engine/src/services/engine_application_service.rs") -Raw
foreach ($Token in @("StorageService", "StorageHostService", "NetworkService", "EngineAction::GetStorageOverview", "EngineAction::CreateVmDisk", "EngineAction::GetNetworkOverview", "EngineAction::AttachDefaultNetwork")) {
    if (-not $EngineApplicationStorageNetworkContent.Contains($Token)) { throw "STORAGE_NETWORK_APPLICATION_MISSING: $Token" }
}

$DesktopView = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/desktop/ui/index.html") -Raw
$DesktopScriptContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/desktop/ui/app.js") -Raw
$DesktopWorkspaceViewContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/desktop/ui/views/vm_workspace_view.js") -Raw
$DesktopCombinedScriptContent = "$DesktopScriptContent`n$DesktopWorkspaceViewContent"
$DesktopControllerContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/desktop/src-tauri/src/controllers/desktop_controller.rs") -Raw

foreach ($Token in @("storage-button", "storage-modal", "storage-disk-form", "storage-disk-id", "network-button", "network-modal", "network-attach-form", "network-id")) {
    if (-not $DesktopView.Contains($Token)) { throw "STORAGE_NETWORK_DESKTOP_VIEW_MISSING: $Token" }
}
foreach ($Token in @("get_storage_overview", "create_vm_disk", "get_network_overview", "attach_default_network", "AttachDefaultNetworkRequest")) {
    if (-not $DesktopControllerContent.Contains($Token)) { throw "STORAGE_NETWORK_DESKTOP_CONTROLLER_MISSING: $Token" }
}
foreach ($Token in @("openStorageModal", "create_vm_disk", "openNetworkModal", "attach_network_profile", "suggestUniqueVmId", "nextResourceId", "refreshStorageDiskSuggestion", "refreshNetworkIdSuggestion")) {
    if (-not $DesktopScriptContent.Contains($Token)) { throw "STORAGE_NETWORK_DESKTOP_SCRIPT_MISSING: $Token" }
}

$PortableImageSpecContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "docs/TVMIMG_FORMAT.md") -Raw
foreach ($Token in @(".tvmimg", "ZIP64", "private_copy_default: true", "os-private.qcow2", ".qcow2", ".vmdk", ".vdi")) {
    if (-not $PortableImageSpecContent.Contains($Token)) { throw "TVMIMG_FORMAT_SPEC_MISSING: $Token" }
}

$NativeDisplayContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/display/src/views/native_display_view.rs") -Raw
$NativeDisplayContracts = @(
    @{ Name = "mutable_surface_buffer"; Pattern = "surface\.buffer_mut\(\)" },
    @{ Name = "one_to_one_copy_fast_path"; Pattern = "buffer\.copy_from_slice\(&self\.frame\.pixels\)" },
    @{ Name = "cached_scale_map"; Pattern = "(?s)self\.scale_map\s*\.ensure\(" },
    @{ Name = "scaled_pixel_write"; Pattern = "buffer\[[^\]]+\]\s*=\s*self\.frame\.pixels\[[^\]]+\]" },
    @{ Name = "softbuffer_present"; Pattern = "buffer\.present\(\)" }
)
foreach ($Contract in $NativeDisplayContracts) {
    if (-not [regex]::IsMatch($NativeDisplayContent, $Contract.Pattern)) {
        throw "SOFTBUFFER_DISPLAY_CONTRACT_MISSING: $($Contract.Name)"
    }
}
foreach ($ForbiddenToken in @("softbuffer::{Context, Pixel, Surface}", "surface.next_buffer()", "pixels_iter()", "Pixel::new_rgb")) {
    if ($NativeDisplayContent.Contains($ForbiddenToken)) {
        throw "SOFTBUFFER_LEGACY_API_PRESENT: $ForbiddenToken"
    }
}

$VmRepositoryContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/repositories/src/repositories/yaml_vm_repository.rs") -Raw
if (-not $VmRepositoryContent.Contains("MANIFEST_SCHEMA_VERSION: u16 = 6")) {
    throw "MANIFEST_SCHEMA_VERSION_INVALID"
}

$AndroidRepositoryContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/repositories/src/repositories/yaml_android_profile_repository.rs") -Raw
if (-not $AndroidRepositoryContent.Contains("PROFILE_SCHEMA_VERSION: u16 = 1")) {
    throw "ANDROID_PROFILE_SCHEMA_VERSION_INVALID"
}

$AndroidPackageServiceContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/android/src/services/android_package_service.rs") -Raw
foreach ($Token in @("fs::canonicalize(package_root)", "canonical_candidate.starts_with(&canonical_root)", "InvalidApkRelativePath")) {
    if (-not $AndroidPackageServiceContent.Contains($Token)) {
        throw "ANDROID_PACKAGE_ROOT_GUARD_MISSING: $Token"
    }
}

$AndroidApplicationContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/engine/src/services/android_application_service.rs") -Raw
foreach ($Token in @("ANDROID_NETWORK_ID", "AdbPortExhausted", "AndroidNetworkMismatch", "DetachNetworkCommand", "TcpListener::bind")) {
    if (-not $AndroidApplicationContent.Contains($Token)) {
        throw "ANDROID_PROVISIONING_GUARD_MISSING: $Token"
    }
}

$DesktopView = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/desktop/ui/index.html") -Raw
$DesktopStyleContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/desktop/ui/styles.css") -Raw
$DesktopThemeContracts = @(
    @{ Name = "light_color_scheme"; Pattern = "color-scheme:\s*light" },
    @{ Name = "panel_token"; Pattern = "--panel:\s*#[0-9A-Fa-f]{6}" },
    @{ Name = "text_token"; Pattern = "--text:\s*#[0-9A-Fa-f]{6}" },
    @{ Name = "accent_token"; Pattern = "--accent:\s*#[0-9A-Fa-f]{6}" },
    @{ Name = "sidebar_token"; Pattern = "--sidebar-bg:\s*#[0-9A-Fa-f]{6}" },
    @{ Name = "sidebar_component"; Pattern = "(?m)^\.sidebar\s*\{" },
    @{ Name = "workspace_component"; Pattern = "(?m)^\.workspace-panel\s*\{" },
    @{ Name = "metric_component"; Pattern = "(?m)^\.metric-card\s*\{" }
)
foreach ($Contract in $DesktopThemeContracts) {
    if (-not [regex]::IsMatch($DesktopStyleContent, $Contract.Pattern)) {
        throw "DESKTOP_LIGHT_THEME_CONTRACT_MISSING: $($Contract.Name)"
    }
}
foreach ($LegacyDarkToken in @("background: #071211", "--panel: #0c1918", "--text: #e8f6f5")) {
    if ($DesktopStyleContent.Contains($LegacyDarkToken)) { throw "DESKTOP_DARK_THEME_TOKEN_PRESENT: $LegacyDarkToken" }
}

$DesktopUxContracts = @(
    @{ Name = "vm_quick_filter"; ViewToken = "vm-filter-group"; ScriptToken = "setVmQuickFilter" },
    @{ Name = "vm_sort"; ViewToken = "vm-sort-select"; ScriptToken = "setVmSortMode" },
    @{ Name = "vm_view_toggle"; ViewToken = "compact-view-button"; ScriptToken = "setVmViewMode" },
    @{ Name = "session_activity"; ViewToken = "activity-list"; ScriptToken = "recordActivity" },
    @{ Name = "toast_feedback"; ViewToken = "toast-stack"; ScriptToken = "showToast" },
    @{ Name = "keyboard_shortcuts"; ViewToken = "shortcuts-modal"; ScriptToken = "closeTopModal" },
    @{ Name = "sidebar_collapse"; ViewToken = "sidebar-collapse-button"; ScriptToken = "toggleSidebar" },
    @{ Name = "bulk_vm_actions"; ViewToken = "bulk-start-button"; ScriptToken = "handleBulkVmAction" },
    @{ Name = "post_create_storage_flow"; ViewToken = "storage-next-button"; ScriptToken = "handleStorageFlowNext" },
    @{ Name = "post_create_android_image_flow"; ViewToken = "android-image-next-button"; ScriptToken = "handleAndroidImageFlowNext" },
    @{ Name = "post_create_network_flow"; ViewToken = "network-next-button"; ScriptToken = "handleNetworkFlowNext" },
    @{ Name = "post_create_configuration_summary"; ViewToken = "configuration-complete-modal"; ScriptToken = "openConfigurationCompleteModal" }
)
$DesktopScriptContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/desktop/ui/app.js") -Raw
$DesktopWorkspaceViewContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/desktop/ui/views/vm_workspace_view.js") -Raw
$DesktopCombinedScriptContent = "$DesktopScriptContent`n$DesktopWorkspaceViewContent"
foreach ($Contract in $DesktopUxContracts) {
    if (-not $DesktopView.Contains($Contract.ViewToken)) {
        throw "DESKTOP_UX_VIEW_CONTRACT_MISSING: $($Contract.Name)"
    }
    if (-not $DesktopScriptContent.Contains($Contract.ScriptToken)) {
        throw "DESKTOP_UX_SCRIPT_CONTRACT_MISSING: $($Contract.Name)"
    }
}

foreach ($Token in @("fn prepare_vm_tcp_access(", "VM_STATE_RUNNING", "self.stop_vm(vm_id.clone())", "Ok(_) => self.start_vm(vm_id)")) {
    if (-not $DisplayServiceContent.Contains($Token)) { throw "CONNECTION_RUNNING_VM_SERVICE_CONTRACT_MISSING: $Token" }
}
foreach ($Token in @("const vmCanReconfigure = vmStopped || vmRunning;", "const restartsRunningVm = vmState === STATE_RUNNING;", "Host portu eklendi ve VM yeniden baslatildi.")) {
    if (-not $DesktopScriptContent.Contains($Token)) { throw "CONNECTION_RUNNING_VM_VIEW_CONTRACT_MISSING: $Token" }
}
foreach ($ForbiddenToken in @("elements.prepareSshAccess.disabled = !vmStopped;", "elements.prepareRdpAccess.disabled = !vmStopped;", "calisan VM config'i degistirilmez")) {
    if ($DesktopScriptContent.Contains($ForbiddenToken)) { throw "CONNECTION_RUNNING_VM_DEAD_END_RETURNED: $ForbiddenToken" }
}

$DesktopCompletenessContracts = @(
    @{ Name = "disk_management"; ViewToken = "storage-disk-list"; ScriptToken = "handleStorageDiskAction" },
    @{ Name = "network_detach"; ViewToken = "network-attachment-list"; ScriptToken = "handleNetworkDetach" },
    @{ Name = "local_logs"; ViewToken = "logs-modal"; ScriptToken = "openLogsModal" },
    @{ Name = "installer_media"; ViewToken = "installer-media-modal"; ScriptToken = "openInstallerMediaModal" },
    @{ Name = "vm_edit"; ViewToken = "vm-edit-modal"; ScriptToken = "handleVmEdit" },
    @{ Name = "vm_delete"; ViewToken = 'data-action="delete-vm"'; ScriptToken = 'invoke("delete_vm"' }
)
foreach ($Contract in $DesktopCompletenessContracts) {
    if (-not $DesktopView.Contains($Contract.ViewToken) -and -not $DesktopCombinedScriptContent.Contains($Contract.ViewToken)) {
        throw "DESKTOP_COMPLETENESS_VIEW_CONTRACT_MISSING: $($Contract.Name)"
    }
    if (-not $DesktopScriptContent.Contains($Contract.ScriptToken)) {
        throw "DESKTOP_COMPLETENESS_SCRIPT_CONTRACT_MISSING: $($Contract.Name)"
    }
}

foreach ($StaleToken in @("create-assign-image", "create-add-network", "handlePostCreateImage", "handlePostCreateNetwork")) {
    if ($DesktopView.Contains($StaleToken) -or $DesktopCombinedScriptContent.Contains($StaleToken)) {
        throw "DESKTOP_STALE_POST_CREATE_BYPASS_PRESENT: $StaleToken"
    }
}

foreach ($Token in @("android-modal", "vm-guest-profile", "adb-status", "android-package-list")) {
    if (-not $DesktopView.Contains($Token)) {
        throw "ANDROID_DESKTOP_VIEW_MISSING: $Token"
    }
}



foreach ($Token in @("home-button", "home-page", "home-overview", "machines-page", "workspace-section-title")) {
    if (-not $DesktopView.Contains($Token)) { throw "V037_WORKSPACE_VIEW_MISSING: $Token" }
}
foreach ($Token in @("renderHome", "renderVmDetail", "inline-snapshot-content", "inline-log-content")) {
    if (-not $DesktopWorkspaceViewContent.Contains($Token) -and -not $DesktopScriptContent.Contains($Token)) {
        throw "V037_WORKSPACE_MODULE_MISSING: $Token"
    }
}
if ($DesktopView.Contains("window.__TAURI__=")) { throw "V037_PREVIEW_MOCK_PRESENT" }

$GpuConfigContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "config/turkuazvm.yml") -Raw
foreach ($Token in @("gpu:", "mode: auto", "hostmem_mib:", "allow_experimental_android_gfxstream: false")) {
    if (-not $GpuConfigContent.Contains($Token)) {
        throw "GPU_CONFIG_MISSING: $Token"
    }
}

$GpuServiceContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/gpu/src/services/gaming_gpu_service.rs") -Raw
foreach ($Token in @("ExperimentalBackendDisabled", "allow_experimental_android_gfxstream", "GpuHostPlatform::Linux", "GpuBackend::VirglVenus", "GpuBackend::Gfxstream")) {
    if (-not $GpuServiceContent.Contains($Token)) {
        throw "GPU_POLICY_GUARD_MISSING: $Token"
    }
}

$QemuGpuProbeContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/qemu/src/tools/qemu_gpu_probe_tool.rs") -Raw
foreach ($Token in @("virtio-gpu", "virtio-gpu-gl", "virtio-gpu-rutabaga", "gfxstream-vulkan", "x-gfxstream-gles", "x-gfxstream-composer")) {
    if (-not $QemuGpuProbeContent.Contains($Token)) {
        throw "QEMU_GPU_PROBE_MISSING: $Token"
    }
}

$QemuCommandBuilderContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/qemu/src/tools/qemu_command_builder.rs") -Raw
foreach ($Token in @("GpuBackend::Virtio2d", "GpuBackend::VirglVenus", "GpuBackend::Gfxstream", "egl-headless,gl=on", "ExperimentalGpuBackendNotEnabled")) {
    if (-not $QemuCommandBuilderContent.Contains($Token)) {
        throw "QEMU_GPU_LAUNCH_STRATEGY_MISSING: $Token"
    }
}

foreach ($Token in @("gaming-button", "gpu-modal", "gpu-status", "gpu-effective", "gpu-vulkan", "gpu-gfxstream")) {
    if (-not $DesktopView.Contains($Token)) {
        throw "GPU_DESKTOP_VIEW_MISSING: $Token"
    }
}



$GamingInputRepositoryContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/repositories/src/repositories/yaml_game_input_profile_repository.rs") -Raw
if (-not $GamingInputRepositoryContent.Contains("PROFILE_SCHEMA_VERSION: u16 = 1")) {
    throw "GAMING_INPUT_PROFILE_SCHEMA_VERSION_INVALID"
}

$GamingInputServiceContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/gaming-input/src/services/gaming_input_translator_service.rs") -Raw
foreach ($Token in @("GamingInputEvent::MouseCapture", "GamingKey::W", "GamingKey::A", "GamingKey::S", "GamingKey::D", "TouchFrame")) {
    if (-not $GamingInputServiceContent.Contains($Token)) {
        throw "GAMING_INPUT_TRANSLATOR_MISSING: $Token"
    }
}

$DisplayGamingBridgeContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/display/src/tools/gaming_input_bridge_tool.rs") -Raw
foreach ($Token in @("INPUT_QUEUE_CAPACITY", "InjectGamingInputEvent", "try_send")) {
    if (-not $DisplayGamingBridgeContent.Contains($Token)) {
        throw "DISPLAY_GAMING_INPUT_BRIDGE_MISSING: $Token"
    }
}

foreach ($Token in @("gaming-input-modal", "gaming-input-profile-form", "android-gaming-input-button", "gaming-binding-list")) {
    if (-not $DesktopView.Contains($Token)) {
        throw "GAMING_INPUT_DESKTOP_VIEW_MISSING: $Token"
    }
}


$GuestProtocolContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/guest-agent-protocol/src/lib.rs") -Raw
if ($GuestProtocolContent -notmatch 'GUEST_AGENT_PROTOCOL_VERSION:\s*u16\s*=\s*([0-9]+)\s*;') {
    throw "GUEST_AGENT_PROTOCOL_VERSION_NOT_FOUND"
}
$GuestProtocolVersion = [int]$Matches[1]
if ($GuestProtocolVersion -ne $ReleaseGuestAgentProtocolVersion) {
    throw "GUEST_AGENT_PROTOCOL_VERSION_INVALID: source=$GuestProtocolVersion release=$ReleaseGuestAgentProtocolVersion"
}
$GuestProtocolKotlinContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "guest/android-agent/src/com/turkuazvm/inputagent/tools/TvgbSecureChannel.kt") -Raw
if ($GuestProtocolKotlinContent -notmatch 'PROTOCOL_VERSION\s*=\s*([0-9]+)') {
    throw "ANDROID_GUEST_AGENT_PROTOCOL_VERSION_NOT_FOUND"
}
$GuestProtocolKotlinVersion = [int]$Matches[1]
if ($GuestProtocolKotlinVersion -ne $ReleaseGuestAgentProtocolVersion) {
    throw "ANDROID_GUEST_AGENT_PROTOCOL_VERSION_INVALID: kotlin=$GuestProtocolKotlinVersion release=$ReleaseGuestAgentProtocolVersion"
}
$GuestProtocolResourceContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "guest/android-agent/res/values/config.xml") -Raw
if ($GuestProtocolResourceContent -notmatch '<integer\s+name="guest_agent_protocol_version">([0-9]+)</integer>') {
    throw "ANDROID_GUEST_AGENT_PROTOCOL_RESOURCE_VERSION_NOT_FOUND"
}
$GuestProtocolResourceVersion = [int]$Matches[1]
if ($GuestProtocolResourceVersion -ne $ReleaseGuestAgentProtocolVersion) {
    throw "ANDROID_GUEST_AGENT_PROTOCOL_RESOURCE_VERSION_INVALID: resource=$GuestProtocolResourceVersion release=$ReleaseGuestAgentProtocolVersion"
}
foreach ($Token in @("ApplyTouchFrame", "persistent_multi_touch", "continuation_api")) {
    if (-not $GuestProtocolContent.Contains($Token)) { throw "GUEST_AGENT_PROTOCOL_MISSING: $Token" }
}

$GuestAgentToolContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/guest/src/tools/android_guest_agent_tool.rs") -Raw
foreach ($Token in @("forward", "--remove", "ApplyTouchFrame", "GuestAgentAction::Capabilities", "provision_and_wait_ready", "SETTING_ENABLED_ACCESSIBILITY_SERVICES", "impl Drop for AndroidGuestAgentTool")) {
    if (-not $GuestAgentToolContent.Contains($Token)) { throw "GUEST_AGENT_HOST_TOOL_MISSING: $Token" }
}

$AndroidAgentGestureContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "guest/android-agent/src/com/turkuazvm/inputagent/tools/TouchGestureTool.kt") -Raw
foreach ($Token in @("continueStroke", "GestureDescription.Builder", "dispatchGesture", "active")) {
    if (-not $AndroidAgentGestureContent.Contains($Token)) { throw "ANDROID_GUEST_AGENT_GESTURE_MISSING: $Token" }
}

$GameCatalogDomainContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/game-catalog/src/domain/game.rs") -Raw
foreach ($Token in @("CatalogKey", "CatalogMouseButton", "PointerIdCollision", "DuplicateBindingSource", "emulator_disclosure_required")) {
    if (-not $GameCatalogDomainContent.Contains($Token)) { throw "GAME_CATALOG_DOMAIN_MISSING: $Token" }
}
if ($GameCatalogDomainContent.Contains("Key(String)") -or $GameCatalogDomainContent.Contains("MouseButton(String)")) {
    throw "GAME_CATALOG_MAGIC_STRING_SOURCE_LEAK"
}
if (-not $GameCatalogDomainContent.Contains("CatalogGpuBackend") -or $GameCatalogDomainContent.Contains("preferred_gpu_backends: Vec<String>")) {
    throw "GAME_CATALOG_GPU_MAGIC_STRING_LEAK"
}

$GameCatalogConfigContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "config/game-catalog.yml") -Raw
foreach ($Token in @("com.tencent.ig", "com.pubg.krmobile", "com.vng.pubgmobile", "com.rekoo.pubgm", "emulator_disclosure_required: true")) {
    if (-not $GameCatalogConfigContent.Contains($Token)) { throw "GAME_CATALOG_PUBG_PROFILE_MISSING: $Token" }
}

$GamepadToolContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/display/src/tools/gamepad_input_tool.rs") -Raw
foreach ($Token in @("Gilrs", "EventType::ButtonPressed", "EventType::AxisChanged", "AXIS_LEFT_X", "BUTTON_SOUTH")) {
    if (-not $GamepadToolContent.Contains($Token)) { throw "GAMEPAD_ADAPTER_MISSING: $Token" }
}

foreach ($Token in @("game-catalog-list", "game-detect-button", "android-guest-agent-status", "gamepad_button:0")) {
    if (-not $DesktopView.Contains($Token)) { throw "GAME_PROFILE_DESKTOP_VIEW_MISSING: $Token" }
}

$DesktopScriptContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/desktop/ui/app.js") -Raw
foreach ($Token in @('data-game-action="compatibility"', 'data-game-action="apply"', 'closest("[data-game-action]")')) {
    if (-not $DesktopScriptContent.Contains($Token)) { throw "GAME_PROFILE_DESKTOP_SCRIPT_MISSING: $Token" }
}

$DesktopControllerContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/desktop/src-tauri/src/controllers/desktop_controller.rs") -Raw
foreach ($Token in @("get_guest_agent_status", "list_game_catalog", "list_guest_catalog", "detect_games", "get_game_compatibility", "apply_game_profile")) {
    if (-not $DesktopControllerContent.Contains($Token)) { throw "GAME_PROFILE_DESKTOP_CONTROLLER_MISSING: $Token" }
}


$AndroidImageRoot = Join-Path $ProjectRoot "crates/android-image"
$ForbiddenAndroidImageTokens = @(
    "turkuazvm_core",
    "turkuazvm_qemu",
    "turkuazvm_guest",
    "turkuazvm_engine_api",
    "serde_json",
    "serde_yaml",
    "serde_yaml_ng",
    "std::process::Command",
    "tauri"
)
foreach ($AndroidImageFile in (Get-ChildItem -Path $AndroidImageRoot -Recurse -Filter "*.rs" -File)) {
    $Content = Get-Content -LiteralPath $AndroidImageFile.FullName -Raw
    foreach ($Token in $ForbiddenAndroidImageTokens) {
        if ($Content.Contains($Token)) { throw "ANDROID_IMAGE_DEPENDENCY_LEAK: $($AndroidImageFile.FullName) -> $Token" }
    }
}

$AndroidImageRepositoryContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/repositories/src/repositories/yaml_android_image_repository.rs") -Raw
$ImageSchemaVersionMatch = [regex]::Match($AndroidImageRepositoryContent, 'const\s+IMAGE_SCHEMA_VERSION:\s*u16\s*=\s*(\d+)\s*;')
if (-not $ImageSchemaVersionMatch.Success) { throw "ANDROID_IMAGE_REPOSITORY_SCHEMA_VERSION_MISSING" }
$ImageSchemaVersion = [int]$ImageSchemaVersionMatch.Groups[1].Value
if ($ImageSchemaVersion -lt 3) { throw "ANDROID_IMAGE_REPOSITORY_SCHEMA_VERSION_UNSUPPORTED: $ImageSchemaVersion" }
if ($ImageSchemaVersion -lt 4 -or -not $AndroidImageRepositoryContent.Contains("sdk_emulator")) { throw "ANDROID_IMAGE_REPOSITORY_SDK_SCHEMA_MISSING" }
foreach ($Token in @("LEGACY_IMAGE_SCHEMA_V2: u16 = 2", "LEGACY_IMAGE_SCHEMA_VERSION: u16 = 1", "ASSIGNMENT_SCHEMA_VERSION: u16 = 2", "LEGACY_ASSIGNMENT_SCHEMA_VERSION: u16 = 1", "AndroidImageState::Installing", "Arc<Mutex<()>>", ".backup", "boot_attempts", "sha256")) {
    if (-not $AndroidImageRepositoryContent.Contains($Token)) { throw "ANDROID_IMAGE_REPOSITORY_MISSING: $Token" }
}

$AndroidImageBuilderContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/guest/src/tools/aosp_android_image_tool.rs") -Raw
foreach ($Token in @("MIN_AOSP_FREE_DISK_GIB: u64 = 400", "Sha256", "boot.img", "super.img", "userdata.img", "bootloader.qemu", "composite.img", "ARM translation is intentionally not included")) {
    if (-not $AndroidImageBuilderContent.Contains($Token)) { throw "ANDROID_IMAGE_BUILDER_MISSING: $Token" }
}

$AospPrepareContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "guest/android-image/scripts/prepare_aosp_source.sh") -Raw
foreach ($Token in @("android-latest-release", "--partial-clone", "--no-use-superproject", "repo sync -c", "MINIMUM_FREE_GIB=400")) {
    if (-not $AospPrepareContent.Contains($Token)) { throw "AOSP_PREPARE_SCRIPT_MISSING: $Token" }
}

$AospBuildContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "guest/android-image/scripts/build_turkuaz_android_image.sh") -Raw
foreach ($Token in @("turkuazvm_cf_x86_64_phone", "aosp_current", "TurkuazInputAgent", "boot.img", "super.img", "userdata.img", "bootloader.qemu", "composite.img", "assemble_turkuaz_android_composite.py", "build-info.yml")) {
    if (-not $AospBuildContent.Contains($Token)) { throw "AOSP_BUILD_SCRIPT_MISSING: $Token" }
}


$RuntimeMediaContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/guest/src/tools/android_runtime_media_tool.rs") -Raw
foreach ($Token in @("VmRuntimeMediaPlan", "CompositeDisk", "Bootloader", ".join(image.id.as_str())", "os-private.qcow2", "pflash.img", "QEMU_IMG_CONVERT", "QEMU_IMG_OUTPUT_FORMAT_FLAG")) {
    if (-not $RuntimeMediaContent.Contains($Token)) { throw "ANDROID_RUNTIME_MEDIA_MISSING: $Token" }
}
foreach ($ForbiddenToken in @("os-overlay.qcow2", "QEMU_IMG_BACKING_FILE_FLAG", "QEMU_IMG_BACKING_FORMAT_FLAG")) {
    if ($RuntimeMediaContent.Contains($ForbiddenToken)) { throw "ANDROID_SHARED_BACKING_POLICY_VIOLATION: $ForbiddenToken" }
}

$QemuRuntimeContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/qemu/src/tools/qemu_command_builder.rs") -Raw
foreach ($Token in @("VmRuntimeMediaPlan", "AndroidRuntimeMediaPlan", "if=pflash", "virtio-blk-pci-non-transitional", "turkuaz-android-os")) {
    if (-not $QemuRuntimeContent.Contains($Token)) { throw "ANDROID_QEMU_BOOT_MEDIA_MISSING: $Token" }
}

$VmLifecycleContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/core/src/services/vm_lifecycle_service.rs") -Raw
foreach ($Token in @("recover_vm", "RecoveryNotSafe", "VmFailureKind::HypervisorStart", "VmFailureKind::NetworkPrepare", "VmFailureKind::NetworkBind")) {
    if (-not $VmLifecycleContent.Contains($Token)) { throw "VM_SAFE_RECOVERY_MISSING: $Token" }
}

$CompositeBuilderContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "guest/android-image/scripts/assemble_turkuaz_android_composite.py") -Raw
foreach ($Token in @("GPT_ENTRY_COUNT", "uboot_env", "boot_android virtio 0#misc", "super", "userdata", "metadata")) {
    if (-not $CompositeBuilderContent.Contains($Token)) { throw "ANDROID_COMPOSITE_BUILDER_MISSING: $Token" }
}

$MainConfigContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "config/turkuazvm.yml") -Raw
foreach ($Token in @("schema_version: 22", "image:", "source_root:", "build_script: guest/android-image/scripts/build_turkuaz_android_image.sh", "distribution:", "minimum_free_disk_gib: 32")) {
    if (-not $MainConfigContent.Contains($Token)) { throw "ANDROID_IMAGE_CONFIG_MISSING: $Token" }
}
foreach ($LegacyToken in @("branch: aosp-android-latest-release", "target: aosp_cf_x86_64_only_phone-userdebug")) {
    if ($MainConfigContent.Contains($LegacyToken)) { throw "ANDROID_IMAGE_CONFIG_DUPLICATE_RESOLVER_POLICY: $LegacyToken" }
}

foreach ($Token in @("android-images-button", "android-images-modal", "android-image-assignment-status", "android-image-select", "android-image-assign", "android-image-quick-vm")) {
    if (-not $DesktopView.Contains($Token)) { throw "ANDROID_IMAGE_DESKTOP_VIEW_MISSING: $Token" }
}

foreach ($Token in @("list_android_images", "define_android_image", "prepare_android_image_build", "register_android_image_build", "install_android_image_distribution", "cancel_android_image_distribution", "cleanup_android_image_distribution", "open_android_image_install_log", "assign_android_image", "get_android_image_assignment")) {
    if (-not $DesktopControllerContent.Contains($Token)) { throw "ANDROID_IMAGE_DESKTOP_CONTROLLER_MISSING: $Token" }
}


$AndroidDistributionPortContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/android-image/src/ports/android_image_distribution_port.rs") -Raw
foreach ($Token in @("AndroidImageDistributionPort", "AndroidImageDistributionRegistration", "AndroidImageDistributionProgress", "AndroidImageDistributionStage", "AndroidImageCapabilities", "prepare_install", "cancel", "cleanup", "bytes_per_second", "eta_seconds")) {
    if (-not $AndroidDistributionPortContent.Contains($Token)) { throw "ANDROID_DISTRIBUTION_PORT_MISSING: $Token" }
}

$DownloadSourcesConfigContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "config/download-sources.yml") -Raw
foreach ($Token in @("schema_version: 5", "paths:", "installer_media: ./data/installer-media", "android_images: ./data/android-image-builds", "artifact_cache: ./data/cache/artifacts", "android_source_cache: ./data/cache/android-distribution-sources.yml", "android_sdk_tools: ./data/tools/android-sdk", "sources:", "android_release_policy:", "android_sdk:", "repository_base_url: https://dl.google.com/android/repository", "package_index_url: https://dl.google.com/android/repository/repository2-1.xml", "emulator_package_path: emulator", "architecture: x86_64", "variant_priority:", "system_image_indexes:", "api_levels:", "android_ci:", "base_url:", "official_base_url:", "default_branch:", "default_target:", "branch_templates:", "target_candidates:", "channels:", "branch_hints:", "allow_device_bootloader_fallback:", '"17":', '"10":')) {
    if (-not $DownloadSourcesConfigContent.Contains($Token)) { throw "ANDROID_DOWNLOAD_SOURCE_CONFIG_MISSING: $Token" }
}
foreach ($Token in @("installer_media_source_cache:", "linux_media:", "use_catalog_fallback:", "discovery_mode:", "directory_index", "official_page_mirrors", "ubuntu:", "debian:", "fedora:", "rocky-linux:", "linux-mint:")) {
    if (-not $DownloadSourcesConfigContent.Contains($Token)) { throw "LINUX_DOWNLOAD_SOURCE_CONFIG_MISSING: $Token" }
}
$LinuxMediaResolverContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/guest-catalog/src/services/installer_media_source_resolver_service.rs") -Raw
$LinuxMediaProviderContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/guest/src/tools/linux_installer_media_provider_tool.rs") -Raw
$InstallerMediaDownloadServiceContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/engine/src/services/installer_media_download_application_service.rs") -Raw
foreach ($Token in @("InstallerMediaSourceResolverService", "self.provider.resolve(request, provider_policy)", "self.cache.load", "LastKnownGoodCache", "CatalogFallback", "catalog_filename != source.filename")) {
    if (-not $LinuxMediaResolverContent.Contains($Token)) { throw "LINUX_MEDIA_RESOLVER_MISSING: $Token" }
}
foreach ($Token in @("LinuxInstallerMediaProviderTool", "OfficialPageMirrors", "stable_fedora_filename", "resolve_official_page_mirrors", "mirror_urls_for_filename", "checksum_filename")) {
    if (-not $LinuxMediaProviderContent.Contains($Token)) { throw "LINUX_MEDIA_PROVIDER_MISSING: $Token" }
}
foreach ($Token in @("InstallerMediaSourceResolverService", "LinuxInstallerMediaProviderTool", "HttpDownloadTool", "fetch_expected_sha256", "provider: template.product_id")) {
    if (-not $InstallerMediaDownloadServiceContent.Contains($Token)) { throw "LINUX_MEDIA_DOWNLOAD_SERVICE_MISSING: $Token" }
}
if ($InstallerMediaDownloadServiceContent.Contains("Command::new(curl_binary)")) { throw "LINUX_MEDIA_DIRECT_CURL_RETURNED" }
foreach ($Token in @("DownloadHttpEngineConfig", "downloads.http.curl_binary", "download_http")) {
    if (-not $EngineConfigContent.Contains($Token)) { throw "SHARED_DOWNLOAD_HTTP_CONFIG_MISSING: $Token" }
}
if ($EngineConfigContent.Contains("distribution_curl_binary") -or $MainConfigContent -match '(?s)android:\s+.*?image:\s+.*?distribution:\s+.*?curl_binary:') {
    throw "ANDROID_LOCAL_DOWNLOAD_TRANSPORT_RETURNED"
}
foreach ($Token in @("download_sources.sources.android_ci.base_url", "download_sources.sources.android_ci.official_base_url", "download_sources.sources.android_release_policy", "download_sources.paths.android_source_cache", "download_sources.paths.android_sdk_tools", "download_sources.sources.android_sdk.repository_base_url", "download_sources.sources.android_sdk.package_index_url", "download_sources.sources.android_sdk.api_levels", "AndroidSdkEngineConfig", "AndroidReleaseProviderConfig::AndroidSdk", "parse_android_distribution_channels", "parse_android_distribution_providers", "distribution_official_fallback", "distribution_branch_templates", "distribution_target_candidates", "distribution_channels", "distribution_providers", "source_cache_path")) {
    if (-not $EngineConfigContent.Contains($Token)) { throw "ANDROID_DOWNLOAD_SOURCE_ENGINE_WIRING_MISSING: $Token" }
}
if ($EngineConfigContent -match '(?s)struct\s+GuestConfig\s*\{[^}]*installer_media_download_root:\s*String') { throw "ENGINE_DOWNLOAD_CONFIG_DEAD_CODE_RISK: GuestConfig.installer_media_download_root" }
if ($EngineConfigContent -match '(?s)struct\s+ArtifactCacheConfig\s*\{[^}]*\broot:\s*String') { throw "ENGINE_DOWNLOAD_CONFIG_DEAD_CODE_RISK: ArtifactCacheConfig.root" }
if ($EngineConfigContent -match '(?s)struct\s+AndroidImageConfig\s*\{[^}]*output_root:\s*String') { throw "ENGINE_DOWNLOAD_CONFIG_DEAD_CODE_RISK: AndroidImageConfig.output_root" }

$AndroidDistributionToolContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/guest/src/tools/android_ci_distribution_tool.rs") -Raw
$AndroidSourceProviderContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/guest/src/tools/android_ci_source_provider_tool.rs") -Raw
$AndroidSourceResolverContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/android-image/src/services/android_distribution_source_resolver_service.rs") -Raw
$AndroidSourceCacheContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/repositories/src/repositories/yaml_android_distribution_source_cache_repository.rs") -Raw
$AndroidDistributionRouterContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/guest/src/tools/android_distribution_router_tool.rs") -Raw
foreach ($Token in @("AndroidDistributionRouterTool", "AndroidDistributionProvider::AndroidSdk", "AndroidDistributionProvider::AndroidCi", "AndroidDistributionProvider::SourceBuild", "Goruntu Merkezi > Build Plan")) {
    if (-not $AndroidDistributionRouterContent.Contains($Token)) { throw "ANDROID_DISTRIBUTION_ROUTER_MISSING: $Token" }
}
$AndroidSdkDistributionToolContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/guest/src/tools/android_sdk_distribution_tool.rs") -Raw
foreach ($Token in @("AndroidSdkDistributionTool", "system-images;android-{api_level};{variant};{}", "remote_package_block", "parse_archive", "host-os", "emulator.exe", "AndroidImageRuntimeKind::SdkEmulator", "validate_sdk_emulator_candidate", "install.log")) {
    if (-not $AndroidSdkDistributionToolContent.Contains($Token)) { throw "ANDROID_SDK_DISTRIBUTION_TOOL_MISSING: $Token" }
}
$AndroidSdkRuntimeToolContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/engine/src/tools/android_sdk_emulator_runtime_tool.rs") -Raw
foreach ($Token in @("AndroidSdkEmulatorRuntimeTool", 'arg("-avd")', 'arg("-port")', 'arg("-accel").arg("auto")', 'env("ANDROID_AVD_HOME"', 'env("ANDROID_SDK_ROOT"', "android-emulator")) {
    if (-not $AndroidSdkRuntimeToolContent.Contains($Token)) { throw "ANDROID_SDK_RUNTIME_TOOL_MISSING: $Token" }
}
$AndroidSdkRuntimeMediaContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/core/src/domain/runtime_media.rs") -Raw
foreach ($Token in @("AndroidSdkEmulatorRuntimeMediaPlan", "AndroidSdkEmulator", "uses_host_network_runtime")) {
    if (-not $AndroidSdkRuntimeMediaContent.Contains($Token)) { throw "ANDROID_SDK_RUNTIME_MEDIA_MISSING: $Token" }
}
$AndroidApplicationServiceContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/engine/src/services/android_application_service.rs") -Raw
$AndroidRuntimeMediaToolContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/guest/src/tools/android_runtime_media_tool.rs") -Raw
foreach ($Token in @("self.settings.adb_host_port_min", "self.settings.adb_host_port_max", "self.is_windows_emulator_adb_port")) {
    if (-not $AndroidApplicationServiceContent.Contains($Token)) { throw "ANDROID_EMULATOR_ADB_PORT_CONFIG_WIRING_MISSING: $Token" }
}
foreach ($Token in @("emulator_console_port_min", "emulator_console_port_max", "self.settings.emulator_console_port_min", "self.settings.emulator_console_port_max")) {
    if (-not $AndroidRuntimeMediaToolContent.Contains($Token)) { throw "ANDROID_EMULATOR_CONSOLE_PORT_CONFIG_WIRING_MISSING: $Token" }
}
if ($AndroidApplicationServiceContent.Contains("(5555..=5681)")) { throw "ANDROID_EMULATOR_ADB_MAGIC_RANGE_RETURNED" }
if ($AndroidRuntimeMediaToolContent.Contains("(5555..=5681)")) { throw "ANDROID_EMULATOR_RUNTIME_MAGIC_RANGE_RETURNED" }
$InvalidAndroidCiRustImport = 'format!("{}/", base_url.trim_end_matches(''/'')),'
if ($AndroidDistributionToolContent.Contains($InvalidAndroidCiRustImport) -or $AndroidSourceProviderContent.Contains($InvalidAndroidCiRustImport)) { throw "ANDROID_CI_RUST_IMPORT_SYNTAX_INVALID: format_macro_in_use_block" }
foreach ($Token in @("status.json", "BUILD_INFO", "/builds/branches/{branch}/", "last_known_good_build", "parse_status_target_candidates", "parse_status_targets", "collect_status_targets", "numeric_build_id", "/builds/submitted/{build_id}/{target}/latest", "cvd-host_package.tar.gz", "DEVICE_IMAGE_PRIMARY", "aosp_cf_x86_64_phone-img", "aosp_cf_x86_64_only_phone-img", "ARTIFACT_PROBE_RANGE", "probe_range", "fetch_text_with_context", "HttpDownloadTool", "allow_device_bootloader_fallback", "host_package_available", "/raw/{BUILD_INFO_FILE}")) {
    if (-not $AndroidSourceProviderContent.Contains($Token)) { throw "ANDROID_SOURCE_PROVIDER_MISSING: $Token" }
}
if ($AndroidSourceProviderContent.Contains("/view/{BUILD_INFO_FILE}")) { throw "ANDROID_BUILD_INFO_VIEW_ENDPOINT_RETURNED" }
$SharedHttpDownloadToolContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/guest/src/tools/http_download_tool.rs") -Raw
foreach ($Token in @("HttpDownloadTool", "HttpRequestContext", "--range", "--max-filesize", "--write-out", "--continue-at", "--retry", "--retry-all-errors", "spawn_file_download", "build_file_download_command", "fetch_headers_with_context")) {
    if (-not $SharedHttpDownloadToolContent.Contains($Token)) { throw "SHARED_HTTP_DOWNLOAD_TOOL_MISSING: $Token" }
}
if ($AndroidSourceProviderContent.Contains("Command::new(&self.settings.curl_binary)") -or $AndroidDistributionToolContent.Contains("Command::new(&self.settings.curl_binary)")) {
    throw "ANDROID_SHARED_HTTP_BYPASS_RETURNED"
}
foreach ($Token in @("AndroidDistributionSourceResolverService", "self.provider.resolve(&request)", "self.cache.load", "source.matches_request", "branch_templates", "branch_hints")) {
    if (-not $AndroidSourceResolverContent.Contains($Token)) { throw "ANDROID_SOURCE_RESOLVER_MISSING: $Token" }
}
foreach ($Token in @("YamlAndroidDistributionSourceCacheRepository", "CACHE_SCHEMA_VERSION", "atomic_write", "resolved_at_unix")) {
    if (-not $AndroidSourceCacheContent.Contains($Token)) { throw "ANDROID_SOURCE_CACHE_MISSING: $Token" }
}
if ($AndroidDistributionToolContent.Contains("status.json") -or $AndroidDistributionToolContent.Contains("parse_last_known_good_build")) { throw "ANDROID_DISTRIBUTION_DISCOVERY_DEAD_CODE_RETURNED" }
foreach ($Token in @("SharedAndroidDistributionSourceResolver", ".resolve(image)", "device_artifact_name", "host_artifact_name", "validate_archive_entries", "extract_x86_64_bootloader", "select_x86_64_bootloader_member", "-xOzf", "sha256_file", "bootloader.qemu", "u-boot.rom", "bootloader_qemu_x86_64", "distribution.yml", "stock_cuttlefish_x86_64", "build_file_download_command", "fetch_headers_with_context", "android_ci_request_context", "download_retry_count", "download_retry_delay_seconds", "download_connect_timeout_seconds", "Resume basarisiz", "available_free_bytes", "install-status.yml", "install.log", "INSTALL_STATUS_SCHEMA_VERSION: u16 = 2", "LEGACY_INSTALL_STATUS_SCHEMA_VERSION: u16 = 1", "parse_content_length", "cancel-requested.flag", "AndroidImageDistributionStage::Cancelling", "stage_started_at_epoch_seconds", "bytes_per_second", "eta_seconds", "ANDROID_CI_BROWSER_USER_AGENT", "CARGO_PKG_VERSION", "copy_x86_64_bootloader_from_device", "bootloader_source")) {
    if (-not $AndroidDistributionToolContent.Contains($Token)) { throw "ANDROID_AUTO_PROVISION_TOOL_MISSING: $Token" }
}

$AndroidCompositeRustContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/guest/src/tools/android_composite_disk_tool.rs") -Raw
foreach ($Token in @("ANDROID_SPARSE_MAGIC", "SPARSE_CHUNK_RAW", "SPARSE_CHUNK_FILL", "GPT_ENTRY_COUNT", "uboot_env", "boot_android virtio 0#misc", "composite partition")) {
    if (-not $AndroidCompositeRustContent.Contains($Token)) { throw "ANDROID_RUST_COMPOSITE_MISSING: $Token" }
}

$AndroidImageServiceContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "crates/android-image/src/services/android_image_service.rs") -Raw
foreach ($Token in @("begin_distribution_install", "run_distribution_install", "cancel_distribution_install", "cleanup_distribution_install", "recover_interrupted_distribution_installs", "distribution_progress", "AndroidImageState::Installing", "ImageAlreadyReady", "ImageInstallNotRunning", "ImageCleanupNotAllowed")) {
    if (-not $AndroidImageServiceContent.Contains($Token)) { throw "ANDROID_AUTO_PROVISION_SERVICE_MISSING: $Token" }
}

foreach ($Token in @('data-android-image-action="install"', 'data-android-image-action="cancel"', 'data-android-image-action="cleanup"', 'data-android-image-action="log"', 'data-android-image-action="assign"', '"Otomatik Kur"', '"Tekrar Dene"', 'image.state === "installing"', 'image.state === "ready"', 'install_android_image_distribution', 'cancel_android_image_distribution', 'cleanup_android_image_distribution', 'open_android_image_install_log', 'setTimeout(() => refreshAndroidImages(false), 3000)', 'image.install_progress', 'ANDROID_INSTALL_STAGE_LABELS', 'install-progress', 'bytes_per_second', 'eta_seconds', 'androidImageQuickVm')) {
    if (-not $DesktopScriptContent.Contains($Token)) { throw "ANDROID_AUTO_PROVISION_DESKTOP_MISSING: $Token" }
}

$AndroidImageApplicationContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/engine/src/services/android_image_application_service.rs") -Raw
foreach ($Token in @("std::thread::spawn", "install_distribution", "cancel_distribution", "cleanup_distribution", "install_progress", "assignment_requires_guest_agent", "recover_interrupted_distribution_installs")) {
    if (-not $AndroidImageApplicationContent.Contains($Token)) { throw "ANDROID_AUTO_PROVISION_APPLICATION_MISSING: $Token" }
}

$LocalLogViewerContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/desktop/src-tauri/src/tools/local_file_viewer_tool.rs") -Raw
foreach ($Token in @("LocalFileViewerTool", "open_data_log", "DATA_DIRECTORY", "fs::canonicalize", "requested.starts_with(&data_root)", "requested.is_file()", '"log" | "txt"', "spawn_text_viewer", "WINDOWS_TEXT_VIEWER", "Command::new(WINDOWS_TEXT_VIEWER)")) {
    if (-not $LocalLogViewerContent.Contains($Token)) { throw "LOCAL_LOG_VIEWER_CONTRACT_MISSING: $Token" }
}

$LocalLogCatalogContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/desktop/src-tauri/src/tools/local_log_catalog_tool.rs") -Raw
foreach ($Token in @("LocalLogCatalogTool", 'data_root.join("logs")', "walk_install_logs", 'Some("install.log")', '"android_image"', "is_log_file")) {
    if (-not $LocalLogCatalogContent.Contains($Token)) { throw "LOCAL_LOG_CATALOG_CONTRACT_MISSING: $Token" }
}

$GuestCatalogConfigContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "config/guest-catalog.yml") -Raw
foreach ($Token in @("schema_version: 4", "linux_media_policies:", "media_kind: network_install", "media_kind: boot", "installer_media:", "installer_media_options:", "mode: direct", "mode: official_page", "checksum_url:", "debian-13.6.0-amd64-netinst.iso", "debian-live-13.6.0-amd64-gnome.iso", "Fedora-Workstation-Live-44-1.7.x86_64.iso", "Fedora-Server-netinst-x86_64-44-1.7.iso", "Fedora-Server-dvd-x86_64-44-1.7.iso", "linuxmint.com/edition.php?id=326", "Rocky-10-latest-x86_64-boot.iso", "Rocky-10-latest-x86_64-minimal.iso", "debian-12.15.0-amd64-netinst.iso", "ubuntu-26.04.1-desktop-amd64.iso", "microsoft.com/software-download/windows11", "android-17-gaming-phone", "android-16-gaming-phone", "android-15-phone", "android-14-phone", "android-13-phone", "android-12l-tablet", "android-12-phone", "android-11-phone", "android-10-phone")) {
    if (-not $GuestCatalogConfigContent.Contains($Token)) { throw "INSTALLER_MEDIA_CATALOG_MISSING: $Token" }
}

$InstallerMediaDownloadServiceContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "apps/engine/src/services/installer_media_download_application_service.rs") -Raw
foreach ($Token in @("InstallerMediaDownloadApplicationService", "thread::spawn", "Sha256", "checksum_url", "InstallerMediaDownloadState::Ready", "InstallerMediaDownloadState::Cancelled", "cancel_requested", "child.kill()", ".part", "SHA256 ({filename}) = ", "valid_sha256", "remove_other_ready_iso_files", "find_ready_iso", "checksum uyusmazligi")) {
    if (-not $InstallerMediaDownloadServiceContent.Contains($Token)) { throw "INSTALLER_MEDIA_DOWNLOAD_SERVICE_MISSING: $Token" }
}

foreach ($Token in @("StartInstallerMediaDownload", "GetInstallerMediaDownload", "CancelInstallerMediaDownload", "AttachDownloadedInstallerMedia", "InstallerMediaDownloadDto", "InstallerMediaSourceDto")) {
    if (-not $EngineApiContent.Contains($Token)) { throw "INSTALLER_MEDIA_ENGINE_API_MISSING: $Token" }
}

foreach ($Token in @("installer-media-download-button", "installer-media-cancel-download-button", "installer-media-attach-downloaded-button", "installer-media-official-page-button", "installer-media-download-progress", "installer-media-options-toggle", "installer-media-source-select", "installer-media-host-architecture", "installer-media-recommended-label")) {
    if (-not $DesktopView.Contains($Token)) { throw "INSTALLER_MEDIA_DESKTOP_VIEW_MISSING: $Token" }
}
foreach ($Token in @("vm-library-pane", "vm-detail-panel", "task-dock", "expert-mode-button")) {
    if (-not $DesktopView.Contains($Token)) { throw "DESKTOP_WORKSPACE_VIEW_MISSING: $Token" }
}
if ($DesktopView.Contains('class="resource-sidebar"')) {
    throw "LEGACY_RESOURCE_SIDEBAR_PRESENT"
}
foreach ($Token in @("start_installer_media_download", "get_installer_media_download", "cancel_installer_media_download", "attach_downloaded_installer_media", "open_external_url", "startInstallerMediaPolling", "cancelInstallerMediaDownload", "installerMediaSourceCompatible", "handleInstallerMediaSourceChange", "media_id: mediaId")) {
    if (-not $DesktopScriptContent.Contains($Token)) { throw "INSTALLER_MEDIA_DESKTOP_SCRIPT_MISSING: $Token" }
}

$LauncherContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "scripts/start_turkuazvm.ps1") -Raw
foreach ($Token in @("Request-DependencyBootstrap", "Get-WindowsHypervisorPlatformState", "Get-FirmwareVirtualizationState", "F3017226-FE2A-4295-8BDF-00C3A9A7E4C5", "Enable-TurkuazVmWindowsHypervisorPlatform", 'Resolve-ToolCommand -Name "curl.exe"', 'Resolve-ToolCommand -Name "tar.exe"')) {
    if (-not $LauncherContent.Contains($Token)) { throw "WINDOWS_LAUNCHER_BOOTSTRAP_MISSING: $Token" }
}

$BootstrapContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "scripts/bootstrap_windows_dependencies.ps1") -Raw
foreach ($Token in @("SoftwareFreedomConservancy.QEMU", "Google.PlatformTools", "Microsoft.EdgeWebView2Runtime", "Microsoft/WinGet/Links", "Microsoft/WinGet/Packages", "WINDOWS_DEPENDENCY_INSTALL_FAILED")) {
    if (-not $BootstrapContent.Contains($Token)) { throw "WINDOWS_DEPENDENCY_BOOTSTRAP_MISSING: $Token" }
}

$WindowsNativeProcessToolContent = Get-Content -LiteralPath (Join-Path $ProjectRoot "tools/windows_native_process_tool.ps1") -Raw
foreach ($Token in @("Get-TurkuazNativeProcessesByExecutablePath", "Stop-TurkuazNativeProcessesByExecutablePath", "Wait-TurkuazNativeFileRelease", "$ProcessMatches = @()", "RUNTIME_PROCESS_STOP_TIMEOUT", "RUNTIME_BINARY_LOCKED")) {
    if (-not $WindowsNativeProcessToolContent.Contains($Token)) { throw "WINDOWS_RUNTIME_BINARY_LOCK_TOOL_MISSING: $Token" }
}
foreach ($Token in @("DESKTOP_ALREADY_RUNNING", "Stale Engine/Display processleri kapatildi", "Wait-TurkuazNativeFileRelease", "target/debug/turkuazvm-engine.exe", "target/debug/turkuazvm-display.exe", "target/debug/turkuazvm-desktop.exe")) {
    if (-not $LauncherContent.Contains($Token)) { throw "WINDOWS_RUNTIME_BINARY_LOCK_LAUNCHER_MISSING: $Token" }
}

Write-Host "STRUCTURE_VERIFY_OK"
