$ErrorActionPreference = 'Stop'

if ($env:OS -ne 'Windows_NT') {
    throw 'This script must be run in Windows PowerShell on a Windows build machine.'
}

foreach ($tool in @('node', 'npm', 'cargo', 'rustc')) {
    if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) {
        throw "Required tool '$tool' was not found. Install Node.js LTS and Rust with the MSVC toolchain, then reopen PowerShell."
    }
}

Write-Host 'Checking LiteStream Windows build prerequisites...' -ForegroundColor Cyan
$rustInfo = (& rustc -vV | Out-String)
if ($rustInfo -notmatch 'host: .*windows-msvc') {
    Write-Warning 'Rust does not appear to use the Windows MSVC target. Install the MSVC Rust toolchain before continuing.'
}

if (-not (Get-Command cl.exe -ErrorAction SilentlyContinue)) {
    Write-Warning 'MSVC cl.exe was not found on PATH. Install Visual Studio Build Tools with the C++ desktop workload and Windows SDK if the build fails.'
}

if (-not (Get-Command ffmpeg -ErrorAction SilentlyContinue) -and -not $env:LITESTREAM_FFMPEG) {
    Write-Warning 'FFmpeg is not on PATH. LiteStream can still build, but RTMP streaming will require FFmpeg installed separately or LITESTREAM_FFMPEG set to its executable path.'
}

Push-Location $PSScriptRoot
try {
    Write-Host 'Installing Node dependencies...' -ForegroundColor Cyan
    npm install --include=dev
    if ($LASTEXITCODE -ne 0) { throw "npm install --include=dev failed with exit code $LASTEXITCODE" }

    Write-Host 'Building LiteStream NSIS installer...' -ForegroundColor Cyan
    npm run build:windows
    if ($LASTEXITCODE -ne 0) { throw "npm run build:windows failed with exit code $LASTEXITCODE" }

    $bundlePath = Join-Path $PSScriptRoot 'src-tauri\target\release\bundle\nsis'
    $installers = @(Get-ChildItem -Path $bundlePath -Filter '*.exe' -File -ErrorAction SilentlyContinue)
    if ($installers.Count -eq 0) {
        throw "Build completed, but no NSIS installer .exe was found under $bundlePath"
    }

    Write-Host ''
    Write-Host 'LiteStream installer created:' -ForegroundColor Green
    $installers | ForEach-Object { Write-Host $_.FullName -ForegroundColor Green }
    Write-Host 'Install FFmpeg separately if you need RTMP streaming; it is not bundled.' -ForegroundColor Yellow
}
finally {
    Pop-Location
}
