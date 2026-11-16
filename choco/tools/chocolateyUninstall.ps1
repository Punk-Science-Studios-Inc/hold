$ErrorActionPreference = 'Stop'

Remove-ItemProperty -Path 'HKLM:\Software\Microsoft\Windows\CurrentVersion\Run' -Name 'Hold' -ErrorAction SilentlyContinue
