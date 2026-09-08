param([switch]$Test)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root
$env:CARGO_TARGET_DIR = Join-Path $Root "target"

if ($Test) {
    cargo test --locked -p kaku2okur-native
    if ($LASTEXITCODE -ne 0) { throw "Cargo tests failed ($LASTEXITCODE)" }
    exit 0
}

cargo build --locked -p kaku2okur-native --release
if ($LASTEXITCODE -ne 0) { throw "Cargo build failed ($LASTEXITCODE)" }

$Dist = Join-Path $Root "dist\windows"
New-Item -ItemType Directory -Force -Path $Dist | Out-Null
Copy-Item "target\release\Kaku2Okur.exe" "$Dist\Kaku2Okur.exe" -Force
Copy-Item "apps\native\assets\icon.ico" "$Dist\icon.ico" -Force
Copy-Item "THIRD_PARTY_NOTICES.md" "$Dist\THIRD_PARTY_NOTICES.md" -Force
Copy-Item "apps\native\assets\icons\LICENSE" "$Dist\Lucide-LICENSE.txt" -Force
Write-Host "Built $Dist\Kaku2Okur.exe"
