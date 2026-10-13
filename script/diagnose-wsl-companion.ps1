param([string]$Distribution, [string]$User, [string]$WarpaiExecutable)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$report = Join-Path $PSScriptRoot ('warpai-wsl-diagnostic-' + (Get-Date -Format 'yyyyMMdd-HHmmss') + '.txt')
$wsl = Join-Path $env:SystemRoot 'System32\wsl.exe'
if (Test-Path (Join-Path $env:SystemRoot 'Sysnative\wsl.exe')) {
    $wsl = Join-Path $env:SystemRoot 'Sysnative\wsl.exe'
}

function Protect-Text([string]$Text) {
    $Text = [regex]::Replace($Text, '(?s)-----BEGIN [^-]*PRIVATE KEY-----.*?-----END [^-]*PRIVATE KEY-----', 'Sensitive content omitted.')
    $lines = foreach ($line in ($Text.Replace([string][char]0, '') -split '\r?\n')) {
        if ($line -match '(?i)(token|secret|password|authorization|cookie|api[_-]?key)\s*[:=]|Bearer\s+\S+|eyJ[A-Za-z0-9_-]{10,}\.|https?://[^/\s]+:[^/\s]+@|\b[A-Z][A-Z0-9_]{2,}=') {
            'Sensitive content omitted.'
        } else { $line }
    }
    return $lines -join "`n"
}

function Write-Report([string]$Text) {
    $safe = Protect-Text $Text
    Write-Host $safe
    Add-Content -LiteralPath $report -Value $safe -Encoding UTF8
}

function Quote-NativeArgument([string]$Value) {
    # WSL parses the Linux command tail itself; do not quote simple arguments.
    if ($Value.Length -gt 0 -and $Value -notmatch '[\s"]') { return $Value }
    $escaped = [regex]::Replace($Value, '(\\*)"', '$1$1\"')
    return '"' + [regex]::Replace($escaped, '(\\+)$', '$1$1') + '"'
}

function Invoke-Probe([string]$Label, [string[]]$Arguments, [string]$InputText = '', [bool]$Setup = $false) {
    Write-Report "`n=== $Label ==="
    Write-Report ('Executable: ' + $wsl)
    Write-Report ('Arguments: ' + ($Arguments | ConvertTo-Json -Compress))
    $info = New-Object System.Diagnostics.ProcessStartInfo
    $info.FileName = $wsl
    $info.Arguments = ($Arguments | ForEach-Object { Quote-NativeArgument $_ }) -join ' '
    $info.UseShellExecute = $false
    $info.CreateNoWindow = $true
    $info.RedirectStandardInput = $true
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    $info.StandardOutputEncoding = [Text.Encoding]::UTF8
    $info.StandardErrorEncoding = [Text.Encoding]::UTF8
    foreach ($name in @('WARP_AGENT_ENDPOINT', 'WARP_AGENT_CAPABILITY', 'WARP_TERMINAL_SESSION_UUID',
        'WARP_AGENT_LAUNCHES', 'WARP_AGENT_BIN', 'WARP_AGENT_LAUNCH_PATH', 'WARP_AGENT_SESSION_TOKEN', 'VIBE_MCP_SERVERS')) {
        $info.EnvironmentVariables.Remove($name)
    }
    $process = New-Object System.Diagnostics.Process
    $process.StartInfo = $info
    try {
        if (-not $process.Start()) { throw 'Could not start the WSL executable.' }
        $stdoutTask = $process.StandardOutput.ReadToEndAsync()
        $stderrTask = $process.StandardError.ReadToEndAsync()
        if ($InputText.Length -gt 0) {
            $inputBytes = [Text.Encoding]::UTF8.GetBytes($InputText.Replace("`r", ''))
            $process.StandardInput.BaseStream.Write($inputBytes, 0, $inputBytes.Length)
        }
        $process.StandardInput.Close()
        if (-not $process.WaitForExit(180000)) {
            $process.Kill()
            Write-Report 'Exit: host timeout after 180 seconds.'
            return [pscustomobject]@{ Exit = -1; Stdout = '' }
        }
        if (-not [System.Threading.Tasks.Task]::WaitAll([System.Threading.Tasks.Task[]]@($stdoutTask, $stderrTask), 5000)) {
            Write-Report 'Exit: child output streams did not close within five seconds.'
            return [pscustomobject]@{ Exit = -1; Stdout = '' }
        }
        $stdout = $stdoutTask.GetAwaiter().GetResult()
        $stderr = $stderrTask.GetAwaiter().GetResult()
        Write-Report ('Exit code: ' + $process.ExitCode)
        if (($stdout.Length + $stderr.Length) -gt 524288) {
            Write-Report 'Output exceeded 512 KiB; content omitted.'
            return [pscustomobject]@{ Exit = -1; Stdout = '' }
        }
        Write-Report 'STDERR:'
        if ($stderr.Length -gt 0) { Write-Report $stderr } else { Write-Report '(empty)' }
        Write-Report 'STDOUT:'
        if ($Setup) {
            # Setup JSON contains configuration metadata; retain only the validation result.
            try {
                $response = $stdout | ConvertFrom-Json
                Write-Report ('Setup JSON account: ' + $response.account)
                Write-Report ('Discovered Agent count: ' + @($response.available).Count)
                if ($response.account -cne $User) { Write-Report 'FAILED: setup response account mismatch.' }
            } catch {
                Write-Report ('Setup response was not valid JSON; raw metadata omitted. Characters: ' + $stdout.Length)
            }
        } elseif ($stdout.Length -gt 0) { Write-Report $stdout } else { Write-Report '(empty)' }
        return [pscustomobject]@{ Exit = $process.ExitCode; Stdout = $stdout }
    } catch {
        Write-Report ('Launcher exception: ' + $_.Exception.GetType().FullName + ': ' + $_.Exception.Message)
        return [pscustomobject]@{ Exit = -1; Stdout = '' }
    } finally { $process.Dispose() }
}

function Check-Bundle {
    if ([string]::IsNullOrWhiteSpace($WarpaiExecutable)) {
        $running = @(Get-Process -Name warpai -ErrorAction SilentlyContinue)
        foreach ($process in $running) {
            if ($process.Path) { $WarpaiExecutable = $process.Path; break }
        }
    }
    Write-Report "`n=== Windows bundled installer payload ==="
    if ([string]::IsNullOrWhiteSpace($WarpaiExecutable)) {
        Write-Report 'Running Warpai executable was not found; bundle check unavailable.'
        return
    }
    $resources = Join-Path (Split-Path -Parent $WarpaiExecutable) 'resources'
    $archive = Join-Path $resources 'wsl-companion.tar.gz'
    $checksum = $archive + '.sha256'
    Write-Report ('Warpai executable: ' + $WarpaiExecutable)
    if (-not (Test-Path -LiteralPath $archive -PathType Leaf) -or -not (Test-Path -LiteralPath $checksum -PathType Leaf)) {
        Write-Report 'FAILED: bundled archive or checksum is missing.'
        return
    }
    $expected = (Get-Content -LiteralPath $checksum -Raw).Trim()
    if ($expected -notmatch '^[a-fA-F0-9]{64}$') {
        Write-Report 'FAILED: bundle checksum metadata is invalid.'
        return
    }
    $actual = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash
    Write-Report ('Archive bytes: ' + (Get-Item -LiteralPath $archive).Length)
    Write-Report ('Bundle checksum match: ' + ($actual -eq $expected))
}

try {
    Write-Report 'Warpai WSL diagnostic v4: capture actual failures before changing application code.'
    Write-Report 'No install, removal, or Agent configuration changes. Setup uses read-only rescan.'
    Write-Report ('Time: ' + (Get-Date -Format 'o'))
    Write-Report ('Windows: ' + [Environment]::OSVersion.VersionString)
    if (-not (Test-Path -LiteralPath $wsl)) { throw 'The system WSL executable is missing.' }
    $null = Invoke-Probe 'WSL distributions' @('--list', '--verbose')
    if ([string]::IsNullOrWhiteSpace($Distribution)) { $Distribution = Read-Host 'Distribution used in Warpai (for example Ubuntu)' }
    if ([string]::IsNullOrWhiteSpace($User)) { $User = Read-Host 'Linux username shown in Warpai' }
    foreach ($value in @($Distribution, $User)) {
        if ([string]::IsNullOrWhiteSpace($value) -or $value.StartsWith('-') -or
            $value.Length -gt 256 -or $value -match '[\x00-\x1F\x7F]') { throw 'Invalid distribution or username.' }
    }
    Check-Bundle
    $guest = @('--distribution', $Distribution, '--user', $User, '--exec')
    $identity = Invoke-Probe 'Direct Linux account check (no shell)' ($guest + @('/usr/bin/id', '-un'))
    if ($identity.Exit -ne 0 -or $identity.Stdout.Trim() -cne $User) {
        Write-Report 'STOP: Linux account execution failed. Companion checks were not attempted.'
        exit 1
    }
    $homeResult = Invoke-Probe 'Direct Linux HOME check (no shell)' ($guest + @('/usr/bin/printenv', 'HOME'))
    $guestHome = $homeResult.Stdout.Trim()
    if ($homeResult.Exit -ne 0 -or $guestHome -notmatch '^/[^\x00-\x1F\x7F]*$' -or $guestHome.Length -gt 4096) {
        Write-Report 'STOP: confirmed Linux HOME is unavailable. No installation path was assumed.'
        exit 1
    }
    $shell = Invoke-Probe 'Shell executable and stdin' ($guest + @('/bin/sh', '-s')) "printf 'shell-stdin-ok\n'`n"
    if ($shell.Exit -ne 0 -or $shell.Stdout.Trim() -ne 'shell-stdin-ok') {
        Write-Report 'STOP: Linux shell execution failed. Companion checks were not attempted.'
        exit 1
    }
    $companion = $guestHome.TrimEnd('/') + '/.config/.warpai/wsl/bin/warpai-wsl-companion'
    $file = Invoke-Probe 'Installed Companion file' ($guest + @('/bin/ls', '-l', '--', $companion))
    if ($file.Exit -ne 0) {
        Write-Report 'STOP: Companion file is unavailable; installed-service verification cannot run in this state.'
        exit 1
    }
    $null = Invoke-Probe 'Installed Companion version' ($guest + @($companion, '--version'))
    $null = Invoke-Probe 'Installed Git runtime' ($guest + @($companion, '--check-runtime'))
    $emptyRequest = '{"action":{"action":"rescan"},"commands":[]}'
    $agentRequest = '{"action":{"action":"rescan"},"commands":[["codex","codex"],["claude","claude"],["agy","agy"],["gemini","gemini"]]}'
    $null = Invoke-Probe 'Direct setup without Agent discovery' ($guest + @($companion, 'setup')) $emptyRequest $true
    $null = Invoke-Probe 'Direct setup with installed Agent discovery' ($guest + @($companion, 'setup')) $agentRequest $true
    # Reproduce the application's existing post-install command, with stdin carrying only the request.
    $appScript = 'test "$(id -un)" = "$2" || exit 126; test -x "$1" || exit 127; exec "$1" setup'
    $null = Invoke-Probe 'Application post-install verification command' ($guest + @('sh', '-lc', $appScript, 'warpai-wsl-setup', $companion, $User)) $agentRequest $true
    $null = Invoke-Probe 'Companion account service startup (closed control input)' ($guest + @($companion))
    Write-Report "`nFinished. Send the report back; personal paths may be redacted."
} catch {
    Write-Report ('Diagnostic exception: ' + $_.Exception.GetType().FullName + ': ' + $_.Exception.Message)
    exit 1
} finally {
    Write-Host "`nReport: $report"
}
