# Package script for Tempo (Windows x64)
# Produces a clean, portable release zip in dist/

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location -LiteralPath $root

Write-Host "==> Building optimized release binaries..." -ForegroundColor Cyan
powershell -ExecutionPolicy Bypass -File .\build.ps1 build --release

$distDir = Join-Path $root "dist"
$pkgDir = Join-Path $distDir "tempo-windows-x64"
$zipFile = Join-Path $distDir "tempo-windows-x64.zip"

if (Test-Path $pkgDir) { Remove-Item -Recurse -Force $pkgDir }
if (Test-Path $zipFile) { Remove-Item -Force $zipFile }
New-Item -ItemType Directory -Path $pkgDir | Out-Null

Write-Host "==> Assembling release files into $pkgDir..." -ForegroundColor Cyan
Copy-Item "target\release\tempo.exe" -Destination "$pkgDir\"
Copy-Item "target\release\tempo-cli.exe" -Destination "$pkgDir\"
Copy-Item "README.md" -Destination "$pkgDir\"
Copy-Item "LICENSE" -Destination "$pkgDir\"
Copy-Item "THIRDPARTY.md" -Destination "$pkgDir\"
Copy-Item "assets\icon.ico" -Destination "$pkgDir\"

Write-Host "==> Compressing into $zipFile..." -ForegroundColor Cyan
Compress-Archive -Path "$pkgDir\*" -DestinationPath $zipFile -Force

$zipSizeMb = [math]::Round((Get-Item $zipFile).Length / 1MB, 2)
Write-Host "==> Packaging complete! Output: $zipFile ($zipSizeMb MB)" -ForegroundColor Green
