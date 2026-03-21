# @task T023
# @epic T014
$ErrorActionPreference = 'Stop'

$packageName = 'ferrous-forge'
$toolsDir = "$(Split-Path -Parent $MyInvocation.MyCommand.Definition)"

# Remove binary
$binaryPath = Join-Path $toolsDir 'ferrous-forge.exe'
if (Test-Path $binaryPath) {
    Remove-Item $binaryPath -Force
}

Write-Host "Ferrous Forge has been uninstalled." -ForegroundColor Green
Write-Host "Note: PATH entry may remain. Remove manually if desired." -ForegroundColor Yellow
