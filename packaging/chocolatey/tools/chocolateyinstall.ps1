# @task T023
# @epic T014
$ErrorActionPreference = 'Stop'

$packageName = 'ferrous-forge'
$toolsDir = "$(Split-Path -Parent $MyInvocation.MyCommand.Definition)"
$url64 = 'https://github.com/kryptobaseddev/ferrous-forge/releases/download/v1.7.6/ferrous-forge-windows-x86_64.zip'
$checksum64 = 'PLACEHOLDER_CHECKSUM'

$packageArgs = @{
    packageName    = $packageName
    unzipLocation  = $toolsDir
    url64bit       = $url64
    checksum64     = $checksum64
    checksumType64 = 'sha256'
}

Install-ChocolateyZipPackage @packageArgs

# Add to PATH if not already present
$installPath = Join-Path $toolsDir 'ferrous-forge.exe'
Install-ChocolateyPath -PathToInstall $toolsDir -PathType 'User'

Write-Host "Ferrous Forge has been installed successfully!" -ForegroundColor Green
Write-Host "Run 'ferrous-forge init' to set up system-wide standards" -ForegroundColor Yellow
Write-Host "Run 'ferrous-forge --help' to see available commands" -ForegroundColor Yellow
