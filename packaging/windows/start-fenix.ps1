# Scheduled-task host for the personal application. Uses the current user's rights.
$ErrorActionPreference = 'Stop'
$env:FENIX_WEB_DIST = Join-Path $PSScriptRoot 'web'
Set-Location $PSScriptRoot
& (Join-Path $PSScriptRoot 'bin/fenix.exe') --data-dir (Join-Path $PSScriptRoot 'data') *>> (Join-Path $PSScriptRoot 'logs/server.log')
exit $LASTEXITCODE
