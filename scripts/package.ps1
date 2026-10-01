param(
    [ValidatePattern('^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$')]
    [string]$Version = '0.1.0',
    [switch]$SkipBuild
)
$ErrorActionPreference = 'Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)
$root = (Get-Location).Path
git rev-parse --verify HEAD | Out-Null
if ($LASTEXITCODE -ne 0) {throw 'Commit the source before packaging'}
$dirty = git status --porcelain --untracked-files=normal
if ($LASTEXITCODE -ne 0 -or $dirty) {throw 'Package from a clean source commit'}
$metadataText = cargo metadata --format-version 1 --locked --offline --filter-platform x86_64-pc-windows-msvc
if ($LASTEXITCODE -ne 0) {throw 'Cannot collect locked Windows dependency metadata'}
$metadata = $metadataText | ConvertFrom-Json
$project = $metadata.packages | Where-Object {$_.name -eq 'journey-to-a-black-hole'}
if ($project.version -ne $Version) {throw 'Version must match Cargo.toml'}
if (!$SkipBuild) {
    cargo build --release --locked
    if ($LASTEXITCODE -ne 0) {throw 'Release build failed'}
}
$exe = Join-Path $metadata.target_directory 'release/journey-to-a-black-hole.exe'
if (!(Test-Path -LiteralPath $exe)) {throw 'Build the Windows x64 release executable first'}
$name = "Journey-to-a-Black-Hole-v$Version-windows-x64"
$sourceName = "Journey-to-a-Black-Hole-v$Version-source"
$dist = Join-Path $root 'dist'
$folder = Join-Path $dist $name
if (Test-Path -LiteralPath $folder) {throw "Existing package folder: $folder. Use a fresh output/version."}
New-Item -ItemType Directory -Path $folder -Force | Out-Null
Copy-Item -LiteralPath $exe -Destination (Join-Path $folder 'Journey-to-a-Black-Hole.exe')
Copy-Item -LiteralPath assets,examples,docs,README.md,CONTRIBUTING.md,NOTICE,LICENSE-MIT,LICENSE-APACHE -Destination $folder -Recurse
$licenseRoot = Join-Path $folder 'third-party-licenses'
New-Item -ItemType Directory -Path $licenseRoot | Out-Null
$rows = @('Packages in the locked Windows dependency graph:')
foreach ($package in $metadata.packages) {
    if (!$package.source) {continue}
    $rows += "$($package.name) $($package.version): $($package.license)"
    $packageRoot = Split-Path $package.manifest_path -Parent
    $files = Get-ChildItem -LiteralPath $packageRoot -File | Where-Object {$_.Name -match '^(LICENSE|LICENCE|COPYING|COPYRIGHT|NOTICE)'}
    if ($files.Count -eq 0) {continue}
    $destination = Join-Path $licenseRoot "$($package.name)-$($package.version)"
    New-Item -ItemType Directory -Path $destination | Out-Null
    foreach ($file in $files) {
        $copy = Copy-Item -LiteralPath $file.FullName -Destination $destination -PassThru
        if ($copy.LastWriteTime.Year -lt 1980) {$copy.LastWriteTime = Get-Date}
    }
}
$rows | Set-Content -LiteralPath (Join-Path $licenseRoot 'PACKAGES.txt') -Encoding utf8
@'
@echo off
cd /d "%~dp0"
Journey-to-a-Black-Hole.exe --journey %*
'@ | Set-Content -LiteralPath (Join-Path $folder 'Start Journey.cmd') -Encoding ascii
@'
@echo off
cd /d "%~dp0"
Journey-to-a-Black-Hole.exe --preset 1 %*
'@ | Set-Content -LiteralPath (Join-Path $folder 'Free Exploration.cmd') -Encoding ascii
@'
@echo off
cd /d "%~dp0"
Journey-to-a-Black-Hole.exe --config examples/kerr.cfg %*
'@ | Set-Content -LiteralPath (Join-Path $folder 'Spinning Exploration.cmd') -Encoding ascii
@'
Journey to a Black Hole — Windows x64

Extract the complete folder. No Rust installation is needed.
Start Journey.cmd: guided flight. Free Exploration.cmd: free camera.
Spinning Exploration.cmd: high-quality Kerr comparison.
Keep assets/ beside the executable. Tested on Windows/Vulkan/RTX 4060.

Tab/right mouse captures mouse, Esc releases. WASD/Space/Ctrl translate.
Mouse looks, Q/E rolls, Shift speeds up, Alt slows down, wheel changes speed.
1–7 views. J starts/pauses the Journey, Enter restarts; input takes over.
P parameters/help, H hides UI, T compares Kerr/Schwarzschild.
F7 compares quantized/continuous output, I inspects rays, G shows the sky grid.
M shows numerical error, F11 marks unfinished rays. F12 saves, L reloads.

See README.md and docs/ for equations, controls, performance and limits.
This version runs natively; there is no browser build in this package.
'@ | Set-Content -LiteralPath (Join-Path $folder 'START-HERE.txt') -Encoding utf8
$commit = git rev-parse HEAD
$commit | Set-Content -LiteralPath (Join-Path $folder 'SOURCE-COMMIT.txt') -Encoding ascii
$portableZip = Join-Path $dist "$name.zip"
$sourceZip = Join-Path $dist "$sourceName.zip"
Compress-Archive -LiteralPath $folder -DestinationPath $portableZip -CompressionLevel Optimal
git archive --format=zip "--prefix=$sourceName/" "--output=$sourceZip" HEAD
if ($LASTEXITCODE -ne 0) {throw 'Tracked source archive failed'}
$checksums = @($portableZip,$sourceZip) | ForEach-Object {
    $hash = (Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash.ToLowerInvariant()
    "$hash  $(Split-Path $_ -Leaf)"
}
$checksumPath = Join-Path $dist "Journey-to-a-Black-Hole-v$Version-SHA256SUMS.txt"
$checksums | Set-Content -LiteralPath $checksumPath -Encoding ascii
Write-Output "Portable: $portableZip"
Write-Output "Source: $sourceZip"
Write-Output "Checksums: $checksumPath"
