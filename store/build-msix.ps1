# Builds an MSIX package for the Microsoft Store from the Tauri release exe.
# Needs env: MSIX_IDENTITY_NAME, MSIX_PUBLISHER, MSIX_PUBLISHER_DISPLAY (from Partner Center > Product identity)
$ErrorActionPreference = 'Stop'
foreach ($n in 'MSIX_IDENTITY_NAME','MSIX_PUBLISHER','MSIX_PUBLISHER_DISPLAY') {
  if (-not (Get-Item "env:$n" -ErrorAction SilentlyContinue).Value) { throw "Missing repository variable $n (Settings > Secrets and variables > Actions > Variables)" }
}
$pkg = Get-Content package.json -Raw | ConvertFrom-Json
$ver = "$($pkg.version).0"                      # Store requires 4-part version; last part must be 0
$exe = 'src-tauri/target/release/litestream.exe'
if (-not (Test-Path $exe)) { throw "Built exe not found: $exe" }

$layout = 'msix-layout'
Remove-Item $layout -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path "$layout/Assets" | Out-Null
Copy-Item $exe "$layout/LiteStream.exe"

# Tile / logo images made from the app icon
python -m pip install --quiet pillow
python - <<'PY'
from PIL import Image
src = Image.open('src-tauri/icons/icon.png').convert('RGBA')
def tile(name, w, h, scale=0.7):
    bg = Image.new('RGBA', (w, h), (10, 22, 38, 255))
    s = int(min(w, h) * scale)
    ic = src.resize((s, s), Image.LANCZOS)
    bg.paste(ic, ((w - s) // 2, (h - s) // 2), ic)
    bg.save(f'msix-layout/Assets/{name}.png')
tile('StoreLogo', 50, 50, 0.9)
tile('Square44x44Logo', 44, 44, 0.9)
tile('Square71x71Logo', 71, 71)
tile('Square150x150Logo', 150, 150)
tile('Wide310x150Logo', 310, 150, 0.8)
tile('Square310x310Logo', 310, 310)
PY

$manifest = @"
<?xml version="1.0" encoding="utf-8"?>
<Package xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10"
         xmlns:uap="http://schemas.microsoft.com/appx/manifest/uap/windows10"
         xmlns:rescap="http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities"
         IgnorableNamespaces="uap rescap">
  <Identity Name="$env:MSIX_IDENTITY_NAME" Publisher="$env:MSIX_PUBLISHER" Version="$ver" ProcessorArchitecture="x64" />
  <Properties>
    <DisplayName>Litestream</DisplayName>
    <PublisherDisplayName>$env:MSIX_PUBLISHER_DISPLAY</PublisherDisplayName>
    <Logo>Assets\StoreLogo.png</Logo>
  </Properties>
  <Dependencies>
    <TargetDeviceFamily Name="Windows.Desktop" MinVersion="10.0.17763.0" MaxVersionTested="10.0.26100.0" />
  </Dependencies>
  <Resources><Resource Language="en-us" /></Resources>
  <Applications>
    <Application Id="LiteStream" Executable="LiteStream.exe" EntryPoint="Windows.FullTrustApplication">
      <uap:VisualElements DisplayName="LiteStream Studio" Description="Lightweight capture, presentation and streaming studio"
        BackgroundColor="#0A1626" Square150x150Logo="Assets\Square150x150Logo.png" Square44x44Logo="Assets\Square44x44Logo.png">
        <uap:DefaultTile Wide310x150Logo="Assets\Wide310x150Logo.png" Square71x71Logo="Assets\Square71x71Logo.png" Square310x310Logo="Assets\Square310x310Logo.png" />
      </uap:VisualElements>
    </Application>
  </Applications>
  <Capabilities>
    <rescap:Capability Name="runFullTrust" />
    <DeviceCapability Name="microphone" />
    <DeviceCapability Name="webcam" />
  </Capabilities>
</Package>
"@
Set-Content -Path "$layout/AppxManifest.xml" -Value $manifest -Encoding utf8

$makeappx = Get-ChildItem 'C:\Program Files (x86)\Windows Kits\10\bin\*\x64\makeappx.exe' | Sort-Object FullName -Descending | Select-Object -First 1
if (-not $makeappx) { throw 'makeappx.exe not found on this runner' }
New-Item -ItemType Directory -Path out -Force | Out-Null
$out = "out/LiteStream_${ver}_x64.msix"
& $makeappx.FullName pack /d $layout /p $out /o
if ($LASTEXITCODE -ne 0) { throw "makeappx failed ($LASTEXITCODE)" }
Write-Host "Built $out"
