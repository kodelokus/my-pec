# Gathers the DLLs my-pec.exe needs that Windows may not have, for the installer to put next to it
# (src-tauri/windows/hooks.nsh): Microsoft's ONNX Runtime, loaded when search by meaning starts, and the Visual C++
# runtime it and the app use. Run by `tauri build` before bundling and by `tauri dev` before starting
# (tauri.windows.conf.json); it also puts onnxruntime.dll beside the debug and release builds.
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$out = Join-Path $root 'src-tauri/windows/runtime'
New-Item -ItemType Directory -Force $out | Out-Null

# Microsoft's own build: it picks AVX2 code at run time, so it starts on any x64 processor. Pinned and checked.
$ort = 'onnxruntime-win-x64-1.31.1'
$sha256 = '5869e26bb25314375f2d5ef1877e3ea06b1543d6c710e6e4a1ec2f7177ab4b2c'
$zip = Join-Path $out "$ort.zip"
if (-not (Test-Path $zip) -or (Get-FileHash $zip -Algorithm SHA256).Hash -ne $sha256) {
    $ProgressPreference = 'SilentlyContinue'
    Invoke-WebRequest "https://github.com/microsoft/onnxruntime/releases/download/v1.31.1/$ort.zip" -OutFile $zip
    if ((Get-FileHash $zip -Algorithm SHA256).Hash -ne $sha256) {
        Remove-Item $zip
        throw "$ort.zip does not match its pinned SHA-256"
    }
}
Add-Type -AssemblyName System.IO.Compression.FileSystem
$archive = [IO.Compression.ZipFile]::OpenRead($zip)
try {
    $entry = $archive.GetEntry("$ort/lib/onnxruntime.dll")
    [IO.Compression.ZipFileExtensions]::ExtractToFile($entry, (Join-Path $out 'onnxruntime.dll'), $true)
} finally {
    $archive.Dispose()
}

$target = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $root 'target' }
foreach ($profile in 'debug', 'release') {
    $dir = Join-Path $target $profile
    New-Item -ItemType Directory -Force $dir | Out-Null
    Copy-Item (Join-Path $out 'onnxruntime.dll') $dir
}

$vs = & "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe" -latest -products * `
    -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
$crt = Get-ChildItem "$vs\VC\Redist\MSVC\*\x64\Microsoft.VC14*.CRT" -Directory |
    Sort-Object { [version]$_.Parent.Parent.Name } | Select-Object -Last 1
if (-not $crt) { throw "No Visual C++ redistributable found under $vs" }
foreach ($dll in 'msvcp140.dll', 'msvcp140_1.dll', 'vcruntime140.dll', 'vcruntime140_1.dll') {
    Copy-Item (Join-Path $crt.FullName $dll) $out
}
