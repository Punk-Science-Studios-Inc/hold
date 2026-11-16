$ErrorActionPreference = 'Stop'

$tools = Split-Path -Parent $MyInvocation.MyCommand.Definition

Get-Process -Name 'hold' -ErrorAction SilentlyContinue |
    Where-Object { $_.Path -and $_.Path.StartsWith($tools, [System.StringComparison]::OrdinalIgnoreCase) } |
    Stop-Process -Force
