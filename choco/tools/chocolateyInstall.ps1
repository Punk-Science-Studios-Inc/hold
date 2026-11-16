$ErrorActionPreference = 'Stop'

$exe = Join-Path (Split-Path -Parent $MyInvocation.MyCommand.Definition) 'hold.exe'
$run = 'HKLM:\Software\Microsoft\Windows\CurrentVersion\Run'

Set-ItemProperty -Path $run -Name 'Hold' -Value "`"$exe`""

Write-Host 'Hold starts at next login, or type `hold` to start it now.'
