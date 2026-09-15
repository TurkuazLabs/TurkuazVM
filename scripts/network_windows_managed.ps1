# 📄 Dosya Yolu: /turkuazvm/scripts/network_windows_managed.ps1
# 📌 Amac: Windows hostta TurkuazVM managed TAP fabric, Network Bridge, WinNAT ve servis yayinlama islemlerini yonetir
# 📌 Modul - PowerShell
# Version: 0.31.2
# Aciklama: Transactional rollback, stale bridge recovery, TAP driver probe, GUID discovery ve WinNAT diagnostics saglar
# Bagimli Oldugu Katman: Tool

param(
    [Parameter(Mandatory = $true)][ValidateSet("ensure", "cleanup", "publish", "unpublish", "probe")][string]$Action,
    [string]$TapName = "",
    [string]$FabricId = "turkuaz-net-01",
    [string]$SubnetCidr = "192.168.240.0/24",
    [string]$GatewayIp = "192.168.240.1",
    [ValidateSet("true", "false")][string]$EnableNat = "true",
    [string]$GuestIp = "",
    [ValidateSet("TCP", "UDP")][string]$Protocol = "TCP",
    [int]$HostPort = 0,
    [int]$GuestPort = 0,
    [string]$StateRoot = ".\data"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$NatEnabled = $EnableNat -eq "true"

$Script:TapHardwareId = "tap0901"
$Script:AnchorPrefix = "TurkuazVM-Anchor"
$Script:BridgePrefix = "TurkuazVM-Bridge"
$Script:NatPrefix = "TurkuazVM-NAT"
$Script:StateDirectory = Join-Path $StateRoot "runtime\network\fabric"
$Script:LogDirectory = Join-Path $StateRoot "logs\network"
$Script:LogFile = Join-Path $Script:LogDirectory "windows-managed-network.log"
$Script:BridgeDiscoveryAttempts = 40
$Script:BridgeDiscoveryDelayMs = 500
$Script:BridgeBindingComponentId = "ms_bridge"

function Ensure-TurkuazDirectory {
    param([Parameter(Mandatory = $true)][string]$Path)
    if (-not (Test-Path -LiteralPath $Path -PathType Container)) {
        New-Item -ItemType Directory -Path $Path -Force | Out-Null
    }
}

function Write-TurkuazNetworkLog {
    param([Parameter(Mandatory = $true)][string]$Message)
    Ensure-TurkuazDirectory -Path $Script:LogDirectory
    $Timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss.fff"
    Add-Content -LiteralPath $Script:LogFile -Value "$Timestamp action=$Action fabric=$FabricId tap=$TapName :: $Message" -Encoding UTF8
}

function Assert-TurkuazAdministrator {
    $Identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $Principal = New-Object Security.Principal.WindowsPrincipal($Identity)
    if (-not $Principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
        throw "TURKUAZ_NETWORK_ADMIN_REQUIRED"
    }
}

function Get-TurkuazSafeToken {
    param([Parameter(Mandatory = $true)][string]$Value)
    return ($Value -replace "[^A-Za-z0-9_-]", "-")
}

function Get-TurkuazTapctl {
    $Command = Get-Command "tapctl.exe" -ErrorAction SilentlyContinue
    if ($null -ne $Command) { return $Command.Source }

    $Candidates = @((Join-Path $env:ProgramFiles "OpenVPN\bin\tapctl.exe"))
    if (-not [string]::IsNullOrWhiteSpace(${env:ProgramFiles(x86)})) {
        $Candidates += (Join-Path ${env:ProgramFiles(x86)} "OpenVPN\bin\tapctl.exe")
    }
    foreach ($Candidate in $Candidates) {
        if (Test-Path -LiteralPath $Candidate -PathType Leaf) { return $Candidate }
    }
    throw "TURKUAZ_TAPCTL_NOT_FOUND"
}

function Invoke-TurkuazTapctl {
    param([Parameter(Mandatory = $true)][string[]]$Arguments)
    $Tapctl = Get-TurkuazTapctl
    $Output = (& $Tapctl @Arguments 2>&1 | Out-String).Trim()
    $ExitCode = $LASTEXITCODE
    Write-TurkuazNetworkLog -Message "tapctl args=$($Arguments -join ' ') exit=$ExitCode output=$Output"
    return [pscustomobject]@{ ExitCode = $ExitCode; Output = $Output }
}

function Remove-TurkuazTapByName {
    param([Parameter(Mandatory = $true)][string]$Name, [switch]$IgnoreFailure)
    $Adapter = Get-NetAdapter -Name $Name -ErrorAction SilentlyContinue
    if ($null -eq $Adapter -and -not $IgnoreFailure) { return }
    $Result = Invoke-TurkuazTapctl -Arguments @("delete", $Name)
    if ($Result.ExitCode -ne 0 -and -not $IgnoreFailure) {
        throw "TURKUAZ_TAP_DELETE_FAILED:${Name}:exit=$($Result.ExitCode):output=$($Result.Output)"
    }
}

function Ensure-TurkuazTap {
    param([Parameter(Mandatory = $true)][string]$Name)
    $Existing = Get-NetAdapter -Name $Name -ErrorAction SilentlyContinue
    if ($null -ne $Existing) {
        if ($Existing.Status -eq "Disabled") { Enable-NetAdapter -Name $Name -Confirm:$false }
        Write-TurkuazNetworkLog -Message "tap_reused name=$Name ifindex=$($Existing.ifIndex) status=$($Existing.Status)"
        return (Get-NetAdapter -Name $Name -ErrorAction Stop)
    }

    Write-TurkuazNetworkLog -Message "tap_create_begin name=$Name"
    $CreateResult = Invoke-TurkuazTapctl -Arguments @("create", "--name", $Name, "--hwid", $Script:TapHardwareId)
    if ($CreateResult.ExitCode -ne 0) {
        throw "TURKUAZ_TAP_CREATE_FAILED:${Name}:exit=$($CreateResult.ExitCode):output=$($CreateResult.Output)"
    }

    for ($Attempt = 1; $Attempt -le $Script:BridgeDiscoveryAttempts; $Attempt++) {
        $Created = Get-NetAdapter -Name $Name -ErrorAction SilentlyContinue
        if ($null -ne $Created) {
            if ($Created.Status -eq "Disabled") { Enable-NetAdapter -Name $Name -Confirm:$false }
            $Created = Get-NetAdapter -Name $Name -ErrorAction Stop
            Write-TurkuazNetworkLog -Message "tap_create_ready name=$Name ifindex=$($Created.ifIndex) attempt=$Attempt"
            return $Created
        }
        Start-Sleep -Milliseconds $Script:BridgeDiscoveryDelayMs
    }
    Remove-TurkuazTapByName -Name $Name -IgnoreFailure
    throw "TURKUAZ_TAP_ADAPTER_TIMEOUT:$Name"
}

function Test-TurkuazTapDriverProbe {
    $ProbeName = "TurkuazVM-TAP-Probe-$PID"
    Remove-TurkuazTapByName -Name $ProbeName -IgnoreFailure
    try {
        $CreateResult = Invoke-TurkuazTapctl -Arguments @("create", "--name", $ProbeName, "--hwid", $Script:TapHardwareId)
        if ($CreateResult.ExitCode -ne 0) {
            throw "TURKUAZ_TAP_DRIVER_PROBE_FAILED:exit=$($CreateResult.ExitCode):output=$($CreateResult.Output)"
        }
        for ($Attempt = 1; $Attempt -le $Script:BridgeDiscoveryAttempts; $Attempt++) {
            $Adapter = Get-NetAdapter -Name $ProbeName -ErrorAction SilentlyContinue
            if ($null -ne $Adapter) {
                Write-TurkuazNetworkLog -Message "tap_driver_probe_ready name=$ProbeName ifindex=$($Adapter.ifIndex) attempt=$Attempt"
                return $true
            }
            Start-Sleep -Milliseconds $Script:BridgeDiscoveryDelayMs
        }
        throw "TURKUAZ_TAP_DRIVER_PROBE_TIMEOUT:$ProbeName"
    } finally {
        Remove-TurkuazTapByName -Name $ProbeName -IgnoreFailure
    }
}

function Get-TurkuazFabricStatePath {
    param([Parameter(Mandatory = $true)][string]$SafeFabric)
    Ensure-TurkuazDirectory -Path $Script:StateDirectory
    return (Join-Path $Script:StateDirectory "$SafeFabric.json")
}

function Get-TurkuazBridgeGuids {
    $Text = (& netsh bridge list 2>&1 | Out-String)
    $Matches = [regex]::Matches($Text, "\{[0-9A-Fa-f-]{36}\}")
    return @($Matches | ForEach-Object { $_.Value.ToUpperInvariant() } | Select-Object -Unique)
}

function Normalize-TurkuazGuid {
    param([string]$Value)
    if ([string]::IsNullOrWhiteSpace($Value)) { return "" }
    return $Value.Trim().TrimStart('{').TrimEnd('}').ToUpperInvariant()
}

function Get-TurkuazBridgeAdapterByGuid {
    param([Parameter(Mandatory = $true)][string]$BridgeGuid)
    $Wanted = Normalize-TurkuazGuid -Value $BridgeGuid
    if ([string]::IsNullOrWhiteSpace($Wanted)) { return $null }
    return Get-NetAdapter -IncludeHidden -ErrorAction SilentlyContinue |
        Where-Object {
            $null -ne $_.InterfaceGuid -and
            (Normalize-TurkuazGuid -Value $_.InterfaceGuid.ToString()) -eq $Wanted
        } |
        Select-Object -First 1
}

function Wait-TurkuazBridgeAdapter {
    param(
        [Parameter(Mandatory = $true)][string]$BridgeGuid,
        [int[]]$BeforeAdapterIndexes = @(),
        [int[]]$ExcludedAdapterIndexes = @()
    )
    for ($Attempt = 1; $Attempt -le $Script:BridgeDiscoveryAttempts; $Attempt++) {
        $ByGuid = Get-TurkuazBridgeAdapterByGuid -BridgeGuid $BridgeGuid
        if ($null -ne $ByGuid) {
            Write-TurkuazNetworkLog -Message "bridge_adapter_by_guid guid=$BridgeGuid name=$($ByGuid.Name) ifindex=$($ByGuid.ifIndex) attempt=$Attempt"
            return $ByGuid
        }

        $Candidate = Get-NetAdapter -IncludeHidden -ErrorAction SilentlyContinue |
            Where-Object {
                $BeforeAdapterIndexes -notcontains [int]$_.ifIndex -and
                $ExcludedAdapterIndexes -notcontains [int]$_.ifIndex
            } |
            Select-Object -First 1
        if ($null -ne $Candidate) {
            Write-TurkuazNetworkLog -Message "bridge_adapter_by_delta guid=$BridgeGuid name=$($Candidate.Name) ifindex=$($Candidate.ifIndex) attempt=$Attempt"
            return $Candidate
        }
        Start-Sleep -Milliseconds $Script:BridgeDiscoveryDelayMs
    }
    return $null
}

function Save-TurkuazFabricState {
    param(
        [string]$Path,
        [string]$BridgeGuid,
        [int]$BridgeIfIndex,
        [string]$BridgeName,
        [string]$AnchorName,
        [string]$NatName
    )
    [ordered]@{
        schema_version = 3
        bridge_guid = $BridgeGuid
        bridge_if_index = $BridgeIfIndex
        bridge_name = $BridgeName
        anchor_name = $AnchorName
        nat_name = $NatName
    } | ConvertTo-Json | Set-Content -LiteralPath $Path -Encoding UTF8
}

function Load-TurkuazFabricState {
    param([string]$Path)
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) { return $null }
    try {
        return (Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json)
    } catch {
        Write-TurkuazNetworkLog -Message "fabric_state_invalid path=$Path error=$($_.Exception.Message)"
        Remove-Item -LiteralPath $Path -Force -ErrorAction SilentlyContinue
        return $null
    }
}

function Invoke-TurkuazNetshBridge {
    param([Parameter(Mandatory = $true)][string[]]$Arguments)
    $Output = (& netsh @Arguments 2>&1 | Out-String).Trim()
    $ExitCode = $LASTEXITCODE
    Write-TurkuazNetworkLog -Message "netsh args=$($Arguments -join ' ') exit=$ExitCode output=$Output"
    if ($ExitCode -ne 0) {
        throw "TURKUAZ_NETSH_BRIDGE_FAILED:exit=${ExitCode}:args=$($Arguments -join ' '):output=$Output"
    }
    return $Output
}

function Ensure-TurkuazBridgeMember {
    param([Parameter(Mandatory = $true)][int]$IfIndex, [Parameter(Mandatory = $true)][string]$BridgeGuid)
    $Adapter = Get-NetAdapter -IncludeHidden -ErrorAction Stop |
        Where-Object { [int]$_.ifIndex -eq $IfIndex } |
        Select-Object -First 1
    if ($null -eq $Adapter) { throw "TURKUAZ_BRIDGE_MEMBER_ADAPTER_NOT_FOUND:$IfIndex" }

    $Binding = Get-NetAdapterBinding -Name $Adapter.Name -ComponentID $Script:BridgeBindingComponentId -ErrorAction SilentlyContinue
    if ($null -ne $Binding -and $Binding.Enabled) {
        Write-TurkuazNetworkLog -Message "bridge_member_reused ifindex=$IfIndex name=$($Adapter.Name)"
        return
    }
    Invoke-TurkuazNetshBridge -Arguments @("bridge", "add", "adapter=$IfIndex", "bridge=$BridgeGuid") | Out-Null
}

function Get-TurkuazNatSummary {
    $Nats = @(Get-NetNat -ErrorAction SilentlyContinue)
    if ($Nats.Count -eq 0) { return "none" }
    return (($Nats | ForEach-Object { "$($_.Name)=$($_.InternalIPInterfaceAddressPrefix)" }) -join ",")
}

function Ensure-TurkuazNat {
    param([Parameter(Mandatory = $true)][string]$NatName)
    if (-not $NatEnabled) { return }

    $Nat = Get-NetNat -Name $NatName -ErrorAction SilentlyContinue
    if ($null -ne $Nat) {
        if ($Nat.InternalIPInterfaceAddressPrefix -ne $SubnetCidr) {
            throw "TURKUAZ_NAT_PREFIX_MISMATCH:expected=${SubnetCidr}:actual=$($Nat.InternalIPInterfaceAddressPrefix)"
        }
        Write-TurkuazNetworkLog -Message "nat_reused name=$NatName prefix=$SubnetCidr"
        return
    }

    $OtherNats = @(Get-NetNat -ErrorAction SilentlyContinue | Where-Object { $_.Name -ne $NatName })
    if ($OtherNats.Count -gt 0) {
        $Summary = Get-TurkuazNatSummary
        throw "TURKUAZ_WINNAT_CONFLICT:requested=${SubnetCidr}:existing=$Summary"
    }

    Write-TurkuazNetworkLog -Message "nat_create_begin name=$NatName prefix=$SubnetCidr"
    New-NetNat -Name $NatName -InternalIPInterfaceAddressPrefix $SubnetCidr | Out-Null
    Write-TurkuazNetworkLog -Message "nat_create_ready name=$NatName prefix=$SubnetCidr"
}

function Ensure-TurkuazFabric {
    param([Parameter(Mandatory = $true)][string]$VmTapName)

    $SafeFabric = Get-TurkuazSafeToken -Value $FabricId
    $AnchorName = "$($Script:AnchorPrefix)-$SafeFabric"
    $BridgeName = "$($Script:BridgePrefix)-$SafeFabric"
    $NatName = "$($Script:NatPrefix)-$SafeFabric"
    $StatePath = Get-TurkuazFabricStatePath -SafeFabric $SafeFabric
    $AnchorExisted = $null -ne (Get-NetAdapter -Name $AnchorName -ErrorAction SilentlyContinue)
    $VmTapExisted = $null -ne (Get-NetAdapter -Name $VmTapName -ErrorAction SilentlyContinue)
    $CreatedBridge = $false
    $BeforeBridgeGuids = @()
    $BridgeGuid = ""
    $BridgeAdapter = $null

    Write-TurkuazNetworkLog -Message "ensure_begin subnet=$SubnetCidr gateway=$GatewayIp nat=$NatEnabled"
    $Anchor = Ensure-TurkuazTap -Name $AnchorName
    $VmTap = Ensure-TurkuazTap -Name $VmTapName

    try {
        $State = Load-TurkuazFabricState -Path $StatePath
        $BridgeGuid = if ($null -ne $State) { [string]$State.bridge_guid } else { "" }
        $BridgeIfIndex = 0
        if ($null -ne $State -and $null -ne $State.PSObject.Properties["bridge_if_index"]) {
            $BridgeIfIndex = [int]$State.bridge_if_index
        }

        if (-not [string]::IsNullOrWhiteSpace($BridgeGuid)) {
            $BridgeAdapter = Wait-TurkuazBridgeAdapter -BridgeGuid $BridgeGuid -ExcludedAdapterIndexes @([int]$Anchor.ifIndex, [int]$VmTap.ifIndex)
        }
        if ($null -eq $BridgeAdapter -and $BridgeIfIndex -gt 0) {
            $BridgeAdapter = Get-NetAdapter -IncludeHidden -ErrorAction SilentlyContinue |
                Where-Object { [int]$_.ifIndex -eq $BridgeIfIndex } |
                Select-Object -First 1
        }
        if ($null -eq $BridgeAdapter) {
            $BridgeAdapter = Get-NetAdapter -Name $BridgeName -ErrorAction SilentlyContinue
        }

        # v0.31.1 veya daha eski bir ensure, bridge rename sonrasinda NAT/state asamasinda
        # kaldiysa state dosyasi olmayabilir. Turkuaz adli bridge adapterinin InterfaceGuid
        # degerini netsh bridge list ile dogrulayip fabric state'i yeniden kur.
        if ([string]::IsNullOrWhiteSpace($BridgeGuid) -and $null -ne $BridgeAdapter -and $null -ne $BridgeAdapter.InterfaceGuid) {
            $NormalizedBridgeGuid = Normalize-TurkuazGuid -Value ($BridgeAdapter.InterfaceGuid.ToString())
            $CandidateGuid = "{$NormalizedBridgeGuid}"
            $KnownBridgeGuids = @(Get-TurkuazBridgeGuids)
            if ($KnownBridgeGuids -contains $CandidateGuid.ToUpperInvariant()) {
                $BridgeGuid = $CandidateGuid.ToUpperInvariant()
                Write-TurkuazNetworkLog -Message "stale_bridge_recovered_by_name guid=$BridgeGuid name=$($BridgeAdapter.Name) ifindex=$($BridgeAdapter.ifIndex)"
            }
        }

        # Rename asamasindan once yarida kalan eski bir Turkuaz bridge icin guvenli recovery:
        # iki Turkuaz TAP da bridge binding tasiyor ve hostta tek bridge varsa onu yeniden kullan.
        if ([string]::IsNullOrWhiteSpace($BridgeGuid) -and $null -eq $BridgeAdapter) {
            $AnchorBinding = Get-NetAdapterBinding -Name $Anchor.Name -ComponentID $Script:BridgeBindingComponentId -ErrorAction SilentlyContinue
            $VmBinding = Get-NetAdapterBinding -Name $VmTap.Name -ComponentID $Script:BridgeBindingComponentId -ErrorAction SilentlyContinue
            $KnownBridgeGuids = @(Get-TurkuazBridgeGuids)
            if ($null -ne $AnchorBinding -and $AnchorBinding.Enabled -and $null -ne $VmBinding -and $VmBinding.Enabled -and $KnownBridgeGuids.Count -eq 1) {
                $BridgeGuid = [string]$KnownBridgeGuids[0]
                $BridgeAdapter = Wait-TurkuazBridgeAdapter -BridgeGuid $BridgeGuid -ExcludedAdapterIndexes @([int]$Anchor.ifIndex, [int]$VmTap.ifIndex)
                if ($null -ne $BridgeAdapter) {
                    Write-TurkuazNetworkLog -Message "stale_bridge_recovered_by_members guid=$BridgeGuid name=$($BridgeAdapter.Name) ifindex=$($BridgeAdapter.ifIndex)"
                }
            }
        }

        if ([string]::IsNullOrWhiteSpace($BridgeGuid) -or $null -eq $BridgeAdapter) {
            $BeforeBridgeGuids = @(Get-TurkuazBridgeGuids)
            $BeforeAdapterIndexes = @(Get-NetAdapter -IncludeHidden -ErrorAction Stop | ForEach-Object { [int]$_.ifIndex })

            Write-TurkuazNetworkLog -Message "bridge_create_begin anchor_ifindex=$($Anchor.ifIndex) vm_ifindex=$($VmTap.ifIndex) before_guids=$($BeforeBridgeGuids -join ',')"
            try {
                Invoke-TurkuazNetshBridge -Arguments @("bridge", "create", "adapter=$($Anchor.ifIndex)", "adapter=$($VmTap.ifIndex)") | Out-Null
            } catch {
                Write-TurkuazNetworkLog -Message "bridge_create_retry_compat initial_error=$($_.Exception.Message)"
                foreach ($IfIndex in @([int]$Anchor.ifIndex, [int]$VmTap.ifIndex)) {
                    try {
                        Invoke-TurkuazNetshBridge -Arguments @("bridge", "set", "adapter", "id=$IfIndex", "forcecompatmode=enable") | Out-Null
                    } catch {
                        Write-TurkuazNetworkLog -Message "bridge_forcecompat_ignored ifindex=$IfIndex error=$($_.Exception.Message)"
                    }
                }
                Start-Sleep -Milliseconds $Script:BridgeDiscoveryDelayMs
                Invoke-TurkuazNetshBridge -Arguments @("bridge", "create", "adapter=$($Anchor.ifIndex)", "adapter=$($VmTap.ifIndex)") | Out-Null
            }
            $CreatedBridge = $true

            $BridgeGuid = ""
            for ($Attempt = 1; $Attempt -le $Script:BridgeDiscoveryAttempts; $Attempt++) {
                $AfterBridgeGuids = @(Get-TurkuazBridgeGuids)
                $NewGuids = @($AfterBridgeGuids | Where-Object { $BeforeBridgeGuids -notcontains $_ })
                if ($NewGuids.Count -gt 0) {
                    $BridgeGuid = [string]$NewGuids[0]
                    break
                }
                if ($AfterBridgeGuids.Count -eq 1) {
                    $BridgeGuid = [string]$AfterBridgeGuids[0]
                    break
                }
                Start-Sleep -Milliseconds $Script:BridgeDiscoveryDelayMs
            }
            if ([string]::IsNullOrWhiteSpace($BridgeGuid)) {
                throw "TURKUAZ_BRIDGE_GUID_TIMEOUT:before=$($BeforeBridgeGuids -join ','):after=$((Get-TurkuazBridgeGuids) -join ',')"
            }

            $BridgeAdapter = Wait-TurkuazBridgeAdapter -BridgeGuid $BridgeGuid -BeforeAdapterIndexes $BeforeAdapterIndexes -ExcludedAdapterIndexes @([int]$Anchor.ifIndex, [int]$VmTap.ifIndex)
            if ($null -eq $BridgeAdapter) { throw "TURKUAZ_BRIDGE_ADAPTER_TIMEOUT:guid=$BridgeGuid" }
        } else {
            Ensure-TurkuazBridgeMember -IfIndex $Anchor.ifIndex -BridgeGuid $BridgeGuid
            Ensure-TurkuazBridgeMember -IfIndex $VmTap.ifIndex -BridgeGuid $BridgeGuid
        }

        if ($BridgeAdapter.Name -ne $BridgeName) {
            Write-TurkuazNetworkLog -Message "bridge_rename from=$($BridgeAdapter.Name) to=$BridgeName"
            $BridgeAdapter | Rename-NetAdapter -NewName $BridgeName -ErrorAction Stop
            $BridgeAdapter = Get-NetAdapter -Name $BridgeName -ErrorAction Stop
        }
        $BridgeIfIndex = [int]$BridgeAdapter.ifIndex

        $PrefixLength = [int]($SubnetCidr.Split('/')[1])
        $ExistingGateway = Get-NetIPAddress -InterfaceIndex $BridgeIfIndex -AddressFamily IPv4 -ErrorAction SilentlyContinue |
            Where-Object { $_.IPAddress -eq $GatewayIp }
        if ($null -eq $ExistingGateway) {
            Write-TurkuazNetworkLog -Message "gateway_assign ifindex=$BridgeIfIndex ip=$GatewayIp prefix=$PrefixLength"
            Get-NetIPAddress -InterfaceIndex $BridgeIfIndex -AddressFamily IPv4 -ErrorAction SilentlyContinue |
                Where-Object { $_.PrefixOrigin -ne "WellKnown" } |
                Remove-NetIPAddress -Confirm:$false -ErrorAction SilentlyContinue
            New-NetIPAddress -InterfaceIndex $BridgeIfIndex -IPAddress $GatewayIp -PrefixLength $PrefixLength | Out-Null
        }

        Ensure-TurkuazNat -NatName $NatName

        Save-TurkuazFabricState -Path $StatePath -BridgeGuid $BridgeGuid -BridgeIfIndex $BridgeIfIndex -BridgeName $BridgeName -AnchorName $AnchorName -NatName $NatName
        Write-TurkuazNetworkLog -Message "ensure_ready bridge_guid=$BridgeGuid bridge_ifindex=$BridgeIfIndex bridge=$BridgeName nat=$NatName"
        return [ordered]@{ BridgeGuid = $BridgeGuid; BridgeIfIndex = $BridgeIfIndex; BridgeName = $BridgeName; NatName = $NatName; TapName = $VmTapName }
    } catch {
        $Failure = $_.Exception.Message
        Write-TurkuazNetworkLog -Message "ensure_rollback_begin error=$Failure created_bridge=$CreatedBridge vm_tap_existed=$VmTapExisted anchor_existed=$AnchorExisted"

        if ($CreatedBridge) {
            $RollbackBridgeGuid = $BridgeGuid
            if ([string]::IsNullOrWhiteSpace($RollbackBridgeGuid)) {
                $AfterBridgeGuids = @(Get-TurkuazBridgeGuids)
                $NewGuids = @($AfterBridgeGuids | Where-Object { $BeforeBridgeGuids -notcontains $_ })
                if ($NewGuids.Count -eq 1) { $RollbackBridgeGuid = [string]$NewGuids[0] }
            }
            if (-not [string]::IsNullOrWhiteSpace($RollbackBridgeGuid)) {
                try {
                    Invoke-TurkuazNetshBridge -Arguments @("bridge", "destroy", "bridge=$RollbackBridgeGuid") | Out-Null
                    Write-TurkuazNetworkLog -Message "ensure_rollback_bridge_destroyed guid=$RollbackBridgeGuid"
                } catch {
                    Write-TurkuazNetworkLog -Message "ensure_rollback_bridge_destroy_ignored error=$($_.Exception.Message)"
                }
            }
        } elseif (-not [string]::IsNullOrWhiteSpace($BridgeGuid)) {
            foreach ($RollbackMember in @(
                [pscustomobject]@{ Created = (-not $VmTapExisted); IfIndex = [int]$VmTap.ifIndex; Name = $VmTapName },
                [pscustomobject]@{ Created = (-not $AnchorExisted); IfIndex = [int]$Anchor.ifIndex; Name = $AnchorName }
            )) {
                if (-not $RollbackMember.Created) { continue }
                try {
                    Invoke-TurkuazNetshBridge -Arguments @("bridge", "remove", "adapter=$($RollbackMember.IfIndex)", "bridge=$BridgeGuid") | Out-Null
                } catch {
                    Write-TurkuazNetworkLog -Message "ensure_rollback_member_remove_ignored name=$($RollbackMember.Name) error=$($_.Exception.Message)"
                }
            }
        }

        if (-not $VmTapExisted) {
            Remove-TurkuazTapByName -Name $VmTapName -IgnoreFailure
            Write-TurkuazNetworkLog -Message "ensure_rollback_vm_tap_removed name=$VmTapName"
        }
        if (-not $AnchorExisted) {
            Remove-TurkuazTapByName -Name $AnchorName -IgnoreFailure
            Write-TurkuazNetworkLog -Message "ensure_rollback_anchor_removed name=$AnchorName"
        }

        Write-TurkuazNetworkLog -Message "ensure_rollback_complete"
        throw
    }
}

function Add-TurkuazMapping {
    if (-not $NatEnabled) { throw "TURKUAZ_MAPPING_REQUIRES_NAT" }
    if ([string]::IsNullOrWhiteSpace($GuestIp) -or $HostPort -le 0 -or $GuestPort -le 0) { throw "TURKUAZ_MAPPING_INVALID" }
    $SafeFabric = Get-TurkuazSafeToken -Value $FabricId
    $NatName = "$($Script:NatPrefix)-$SafeFabric"
    $Existing = Get-NetNatStaticMapping -NatName $NatName -ErrorAction SilentlyContinue |
        Where-Object { $_.Protocol -eq $Protocol -and $_.ExternalPort -eq $HostPort }
    if ($null -ne $Existing) { throw "TURKUAZ_MAPPING_CONFLICT:${Protocol}:${HostPort}" }
    Add-NetNatStaticMapping -NatName $NatName -Protocol $Protocol -ExternalIPAddress "0.0.0.0/0" -ExternalPort $HostPort -InternalIPAddress $GuestIp -InternalPort $GuestPort | Out-Null
    Write-TurkuazNetworkLog -Message "mapping_added protocol=$Protocol host=$HostPort guest=${GuestIp}:${GuestPort}"
}

function Remove-TurkuazMapping {
    $SafeFabric = Get-TurkuazSafeToken -Value $FabricId
    $NatName = "$($Script:NatPrefix)-$SafeFabric"
    Get-NetNatStaticMapping -NatName $NatName -ErrorAction SilentlyContinue |
        Where-Object { $_.Protocol -eq $Protocol -and $_.ExternalPort -eq $HostPort } |
        Remove-NetNatStaticMapping -Confirm:$false -ErrorAction SilentlyContinue
    Write-TurkuazNetworkLog -Message "mapping_removed protocol=$Protocol host=$HostPort"
}

function Remove-TurkuazVmTap {
    $SafeFabric = Get-TurkuazSafeToken -Value $FabricId
    $NatName = "$($Script:NatPrefix)-$SafeFabric"
    if (-not [string]::IsNullOrWhiteSpace($GuestIp)) {
        Get-NetNatStaticMapping -NatName $NatName -ErrorAction SilentlyContinue |
            Where-Object { $_.InternalIPAddress -eq $GuestIp } |
            Remove-NetNatStaticMapping -Confirm:$false -ErrorAction SilentlyContinue
    }
    if ([string]::IsNullOrWhiteSpace($TapName)) { return }
    $Adapter = Get-NetAdapter -Name $TapName -ErrorAction SilentlyContinue
    if ($null -eq $Adapter) { return }
    $StatePath = Get-TurkuazFabricStatePath -SafeFabric $SafeFabric
    $State = Load-TurkuazFabricState -Path $StatePath
    if ($null -ne $State -and -not [string]::IsNullOrWhiteSpace([string]$State.bridge_guid)) {
        try {
            Invoke-TurkuazNetshBridge -Arguments @("bridge", "remove", "adapter=$($Adapter.ifIndex)", "bridge=$($State.bridge_guid)") | Out-Null
        } catch {
            Write-TurkuazNetworkLog -Message "cleanup_bridge_remove_ignored error=$($_.Exception.Message)"
        }
    }
    Remove-TurkuazTapByName -Name $TapName -IgnoreFailure
    Write-TurkuazNetworkLog -Message "tap_removed name=$TapName"
}

try {
    Assert-TurkuazAdministrator
    Write-TurkuazNetworkLog -Message "helper_start subnet=$SubnetCidr gateway=$GatewayIp guest=$GuestIp nat=$NatEnabled"
    switch ($Action) {
        "probe" {
            $TapDriverReady = Test-TurkuazTapDriverProbe
            Invoke-TurkuazNetshBridge -Arguments @("bridge", "show", "adapter") | Out-Null
            [ordered]@{
                tapctl = (Get-TurkuazTapctl)
                tap_driver = $(if ($TapDriverReady) { "ready" } else { "failed" })
                netnat = ($null -ne (Get-Command Get-NetNat -ErrorAction SilentlyContinue))
                nat_summary = (Get-TurkuazNatSummary)
                bridge = "ready"
                log = $Script:LogFile
            } | ConvertTo-Json -Compress
        }
        "ensure" {
            if ([string]::IsNullOrWhiteSpace($TapName)) { throw "TURKUAZ_TAP_NAME_REQUIRED" }
            (Ensure-TurkuazFabric -VmTapName $TapName) | ConvertTo-Json -Compress
        }
        "publish" { Add-TurkuazMapping }
        "unpublish" { Remove-TurkuazMapping }
        "cleanup" { Remove-TurkuazVmTap }
    }
    Write-TurkuazNetworkLog -Message "helper_success"
} catch {
    $Detail = $_.Exception.Message
    Write-TurkuazNetworkLog -Message "helper_failure detail=$Detail"
    [Console]::Error.WriteLine("TURKUAZ_NETWORK_HELPER_ERROR:$Detail log=$Script:LogFile")
    exit 1
}
