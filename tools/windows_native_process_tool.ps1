# 📄 Dosya Yolu: /turkuazvm/tools/windows_native_process_tool.ps1
# 📌 Amac: Windows PowerShell 5.1 altinda native process stdout, stderr ve exit-code akislarini PowerShell hata semantiginden ayirir
# 📌 Modul - PowerShell
# Version: 0.40.13
# Aciklama: Native komut exit-code akislarini ve ayni workspace runtime binary processlerinin exact executable path bazli tespit/temizleme islemlerini yonetir
# Bagimli Oldugu Katman: Tool

function ConvertTo-TurkuazNativeArgumentString {
    param([string[]]$Arguments = @())

    $Encoded = foreach ($Argument in $Arguments) {
        if ($null -eq $Argument) {
            '""'
            continue
        }

        $Value = [string]$Argument
        if ($Value -notmatch '[\s"]') {
            $Value
            continue
        }

        $Escaped = $Value -replace '(\\*)"', '$1$1\"'
        $Escaped = $Escaped -replace '(\\+)$', '$1$1'
        '"' + $Escaped + '"'
    }

    return ($Encoded -join ' ')
}


function Get-TurkuazNativeProcessesByExecutablePath {
    param(
        [Parameter(Mandatory = $true)][string[]]$ExecutablePaths
    )

    $Targets = @{}
    foreach ($ExecutablePath in $ExecutablePaths) {
        if ([string]::IsNullOrWhiteSpace($ExecutablePath)) {
            continue
        }

        try {
            $FullPath = [System.IO.Path]::GetFullPath($ExecutablePath)
            $Targets[$FullPath.ToLowerInvariant()] = $FullPath
        }
        catch {
            continue
        }
    }

    if ($Targets.Count -eq 0) {
        return @()
    }

    # PowerShell 5.1 can throw "Argument types do not match" while materializing
    # generic List[object] values through array-subexpression/cmdlet binding.
    # Native process counts are tiny, so a plain PowerShell array is safer here.
    $ProcessMatches = @()
    foreach ($Process in (Get-Process -ErrorAction SilentlyContinue)) {
        $CandidatePath = $null
        try {
            $CandidatePath = $Process.Path
        }
        catch {
            continue
        }

        if ([string]::IsNullOrWhiteSpace($CandidatePath)) {
            continue
        }

        try {
            $NormalizedCandidate = [System.IO.Path]::GetFullPath($CandidatePath).ToLowerInvariant()
        }
        catch {
            continue
        }

        if ($Targets.ContainsKey($NormalizedCandidate)) {
            $ProcessMatches += [pscustomobject]@{
                ProcessId = [int]$Process.Id
                ExecutablePath = [string]$CandidatePath
            }
        }
    }

    return $ProcessMatches
}

function Stop-TurkuazNativeProcessesByExecutablePath {
    param(
        [Parameter(Mandatory = $true)][string[]]$ExecutablePaths,
        [int]$TimeoutMs = 8000
    )

    $Matches = @(Get-TurkuazNativeProcessesByExecutablePath -ExecutablePaths $ExecutablePaths)
    if ($Matches.Count -eq 0) {
        return @()
    }

    foreach ($Match in $Matches) {
        Stop-Process -Id $Match.ProcessId -Force -ErrorAction SilentlyContinue
    }

    $Stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    do {
        $Remaining = @(Get-TurkuazNativeProcessesByExecutablePath -ExecutablePaths $ExecutablePaths)
        if ($Remaining.Count -eq 0) {
            return @($Matches)
        }
        Start-Sleep -Milliseconds 100
    } while ($Stopwatch.ElapsedMilliseconds -lt $TimeoutMs)

    $RemainingIds = @($Remaining | ForEach-Object { $_.ProcessId }) -join ","
    throw "RUNTIME_PROCESS_STOP_TIMEOUT: pid=$RemainingIds"
}

function Wait-TurkuazNativeFileRelease {
    param(
        [Parameter(Mandatory = $true)][string[]]$Paths,
        [int]$TimeoutMs = 8000
    )

    foreach ($Path in $Paths) {
        if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
            continue
        }

        $Stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
        while ($true) {
            $Stream = $null
            try {
                $Stream = [System.IO.File]::Open(
                    $Path,
                    [System.IO.FileMode]::Open,
                    [System.IO.FileAccess]::ReadWrite,
                    [System.IO.FileShare]::None
                )
                break
            }
            catch {
                if ($Stopwatch.ElapsedMilliseconds -ge $TimeoutMs) {
                    throw "RUNTIME_BINARY_LOCKED: $Path"
                }
                Start-Sleep -Milliseconds 100
            }
            finally {
                if ($null -ne $Stream) {
                    $Stream.Dispose()
                }
            }
        }
    }
}

function Invoke-TurkuazNativeChecked {
    param(
        [Parameter(Mandatory = $true)][string]$FilePath,
        [string[]]$Arguments = @()
    )

    $ArgumentString = ConvertTo-TurkuazNativeArgumentString -Arguments $Arguments
    $StartInfo = New-Object System.Diagnostics.ProcessStartInfo
    $StartInfo.FileName = $FilePath
    $StartInfo.Arguments = $ArgumentString
    $StartInfo.UseShellExecute = $false
    $StartInfo.CreateNoWindow = $false

    $Process = New-Object System.Diagnostics.Process
    $Process.StartInfo = $StartInfo
    try {
        if (-not $Process.Start()) {
            throw "NATIVE_COMMAND_START_FAILED: $FilePath"
        }
        $Process.WaitForExit()
        $ExitCode = $Process.ExitCode
    }
    finally {
        $Process.Dispose()
    }

    if ($ExitCode -ne 0) {
        throw "NATIVE_COMMAND_FAILED: $FilePath exit=$ExitCode"
    }
}

function Invoke-TurkuazNativeCapture {
    param(
        [Parameter(Mandatory = $true)][string]$FilePath,
        [string[]]$Arguments = @()
    )

    $StdOutPath = [System.IO.Path]::GetTempFileName()
    $StdErrPath = [System.IO.Path]::GetTempFileName()
    $ArgumentString = ConvertTo-TurkuazNativeArgumentString -Arguments $Arguments

    try {
        $Process = Start-Process `
            -FilePath $FilePath `
            -ArgumentList $ArgumentString `
            -NoNewWindow `
            -Wait `
            -PassThru `
            -RedirectStandardOutput $StdOutPath `
            -RedirectStandardError $StdErrPath

        $StdOut = if (Test-Path -LiteralPath $StdOutPath) {
            Get-Content -LiteralPath $StdOutPath -Raw -ErrorAction SilentlyContinue
        } else {
            ""
        }
        $StdErr = if (Test-Path -LiteralPath $StdErrPath) {
            Get-Content -LiteralPath $StdErrPath -Raw -ErrorAction SilentlyContinue
        } else {
            ""
        }
        $ExitCode = if ($null -eq $Process) { -1 } else { $Process.ExitCode }
        $Combined = (($StdOut, $StdErr) -join [Environment]::NewLine).Trim()

        return [pscustomobject]@{
            ExitCode = $ExitCode
            StdOut = [string]$StdOut
            StdErr = [string]$StdErr
            Combined = [string]$Combined
        }
    }
    finally {
        Remove-Item -LiteralPath $StdOutPath -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath $StdErrPath -Force -ErrorAction SilentlyContinue
    }
}
