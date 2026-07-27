param(
    [int]$Port = 8083,
    [int]$DisplayIndex = 0,
    [switch]$Release
)

$ErrorActionPreference = "Stop"

$Root = Resolve-Path (Join-Path $PSScriptRoot "..")
$ServerDir = Join-Path $Root "rotascope_server2"

if (-not (Get-Command adb -ErrorAction SilentlyContinue)) {
    throw "adb was not found. Install Android platform-tools and add adb.exe to PATH."
}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "cargo was not found. Install Rust and add cargo.exe to PATH."
}

Write-Host "RotaScope USB display"
Write-Host "Workspace: $Root"
Write-Host "Port: $Port"
Write-Host "Display index: $DisplayIndex"
Write-Host ""

$devices = adb devices
if ($devices -notmatch "`tdevice") {
    Write-Warning "No authorized Android device is visible to adb."
    Write-Warning "Connect the phone by USB, enable USB debugging, and accept the authorization prompt."
}

adb reverse "tcp:$Port" "tcp:$Port"
Write-Host "ADB reverse is ready: phone 127.0.0.1:$Port -> PC 127.0.0.1:$Port"

$env:ROTASCOPE_DISPLAY_INDEX = "$DisplayIndex"

Push-Location $ServerDir
try {
    if ($Release) {
        cargo run --release
    } else {
        cargo run
    }
}
finally {
    Pop-Location
}
