# Build and install for the current Windows user, without administrator privileges.
[CmdletBinding()]
param([switch]$NoStart)
$ErrorActionPreference = 'Stop'
if ([Environment]::OSVersion.Platform -ne 'Win32NT') { throw 'This installer requires Windows.' }
foreach ($dependency in @('cargo', 'pnpm')) {
    if (-not (Get-Command $dependency -ErrorAction SilentlyContinue)) { throw "Missing dependency: $dependency" }
}
$repositoryDirectory = Split-Path $PSScriptRoot -Parent
$installDirectory = Join-Path $env:LOCALAPPDATA 'Fenix'
$dataDirectory = Join-Path $installDirectory 'data'
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$taskName = "FenixPersonal-$($identity.User.Value)"
Push-Location $repositoryDirectory
try {
    & pnpm --dir apps/web install --frozen-lockfile
    if ($LASTEXITCODE -ne 0) { throw 'Frontend dependency installation failed.' }
    & pnpm --dir apps/web build
    if ($LASTEXITCODE -ne 0) { throw 'Frontend build failed.' }
    & cargo build --release --locked -p fenix-server --bin fenix
    if ($LASTEXITCODE -ne 0) { throw 'Rust build failed.' }
    New-Item -ItemType Directory -Force -Path $installDirectory | Out-Null
    foreach ($name in @('bin', 'web', 'logs', 'data')) {
        New-Item -ItemType Directory -Force -Path (Join-Path $installDirectory $name) | Out-Null
    }
    # Only this identity and SYSTEM can access this personal installation.
    & icacls $installDirectory /inheritance:r /grant:r "*$($identity.User.Value):(OI)(CI)F" '*S-1-5-18:(OI)(CI)F' | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Could not apply private installation permissions.' }
    $existing = Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
    if ($existing) {
        Stop-ScheduledTask -TaskName $taskName
        $deadline = (Get-Date).AddSeconds(20)
        while ((Get-ScheduledTask -TaskName $taskName).State -eq 'Running') {
            if ((Get-Date) -ge $deadline) { throw 'Previous Fenix task did not stop; installation files were preserved.' }
            Start-Sleep -Milliseconds 250
        }
        Unregister-ScheduledTask -TaskName $taskName -Confirm:$false
    }
    Copy-Item 'target/release/fenix.exe' (Join-Path $installDirectory 'bin/fenix.exe') -Force
    Copy-Item 'apps/web/dist/*' (Join-Path $installDirectory 'web') -Recurse -Force
    Copy-Item 'packaging/windows/start-fenix.ps1' (Join-Path $installDirectory 'start-fenix.ps1') -Force
    $runner = Join-Path $env:SystemRoot 'System32/WindowsPowerShell/v1.0/powershell.exe'
    $scriptPath = Join-Path $installDirectory 'start-fenix.ps1'
    $action = New-ScheduledTaskAction -Execute $runner -Argument "-NoProfile -NonInteractive -WindowStyle Hidden -File `"$scriptPath`"" -WorkingDirectory $installDirectory
    $trigger = New-ScheduledTaskTrigger -AtLogOn -User $identity.Name
    $principal = New-ScheduledTaskPrincipal -UserId $identity.Name -LogonType Interactive -RunLevel Limited
    $settings = New-ScheduledTaskSettingsSet -ExecutionTimeLimit ([TimeSpan]::Zero) -MultipleInstances IgnoreNew -RestartCount 3 -RestartInterval (New-TimeSpan -Minutes 1) -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries
    Register-ScheduledTask -TaskName $taskName -Action $action -Trigger $trigger -Principal $principal -Settings $settings -Force | Out-Null
    if (-not $NoStart) { Start-ScheduledTask -TaskName $taskName }
    Write-Host "Installed at $installDirectory. Open http://127.0.0.1:4173"
    Write-Host "Configure the owner if needed: & `"$installDirectory\bin\fenix.exe`" owner EMAIL --password-stdin --data-dir `"$dataDirectory`""
    Write-Host 'Browser and Python/PDF/OCR dependencies are installed separately.'
} finally { Pop-Location }
