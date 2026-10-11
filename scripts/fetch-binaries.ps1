# fetch-binaries.ps1 — download static FFmpeg/FFprobe builds for Windows and
# place them where Tauri's `externalBin` (sidecar) expects them:
#
#   src-tauri/binaries/ffmpeg-x86_64-pc-windows-msvc.exe
#   src-tauri/binaries/ffprobe-x86_64-pc-windows-msvc.exe
#
# Source: BtbN GPL builds (GitHub) — includes NVENC / QSV encoders.
#
# Usage:  powershell -ExecutionPolicy Bypass -File scripts/fetch-binaries.ps1
#         [-Triple x86_64-pc-windows-msvc]
param(
    [string]$Triple = $env:DROPSHRINK_TRIPLE
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$BinDir = Join-Path $RepoRoot "src-tauri\binaries"
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null

if (-not $Triple) {
    $arch = $env:PROCESSOR_ARCHITECTURE
    $Triple = switch ($arch) {
        "ARM64" { "aarch64-pc-windows-msvc" }
        default { "x86_64-pc-windows-msvc" }
    }
}

$FfOut = Join-Path $BinDir "ffmpeg-$Triple.exe"
$FpOut = Join-Path $BinDir "ffprobe-$Triple.exe"

if ((Test-Path $FfOut) -and (Test-Path $FpOut)) {
    Write-Host "OK Sidecar binaries already present: $BinDir"
    exit 0
}

$Tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("dropshrink-ffmpeg-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $Tmp | Out-Null
try {
    Write-Host ">> Downloading BtbN static FFmpeg for Windows..."
    $Zip = Join-Path $Tmp "ff.zip"
    Invoke-WebRequest `
        -Uri "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip" `
        -OutFile $Zip -MaximumRetryCount 3 -RetryIntervalSec 3

    Write-Host ">> Extracting..."
    Expand-Archive -Path $Zip -DestinationPath $Tmp -Force
    $BinFolder = Get-ChildItem -Path $Tmp -Directory -Filter "ffmpeg-*-win64-gpl" | Select-Object -First 1

    Copy-Item (Join-Path $BinFolder.FullName "bin\ffmpeg.exe") $FfOut -Force
    Copy-Item (Join-Path $BinFolder.FullName "bin\ffprobe.exe") $FpOut -Force

    Write-Host "OK Installed: $FfOut"
    Write-Host "OK Installed: $FpOut"
}
finally {
    Remove-Item -Recurse -Force $Tmp -ErrorAction SilentlyContinue
}

Write-Host "Done."
