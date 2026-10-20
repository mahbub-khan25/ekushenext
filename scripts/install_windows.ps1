# Ekushe Next - Windows One-Step Installer
# Builds or copies the release binaries (ekushe.exe, ekushe_tsf.dll, ekushe-cli.exe),
# installs icons & Bangladeshi fonts (Kalpurush, SolaimanLipi, Nikosh, Siyam Rupali)
# to %LOCALAPPDATA%\Programs\Ekushe and user Windows Fonts, registers TSF profile in HKCU,
# creates Start Menu shortcut with ekushe.ico, and optionally enables Run-at-Startup.

param(
    [switch]$NoLaunch,
    [switch]$EnableStartup = $true
)

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$installDir = Join-Path $env:LOCALAPPDATA "Programs\Ekushe"
$iconsDir = Join-Path $installDir "assets\icons"
$fontsDir = Join-Path $installDir "assets\fonts"

Write-Host "Installing Ekushe Next Bangla Keyboard to $installDir ..." -ForegroundColor Cyan

# Stop any running ekushe instance so binaries can be updated cleanly
Get-Process -Name "ekushe" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 250

New-Item -ItemType Directory -Force -Path $installDir | Out-Null
New-Item -ItemType Directory -Force -Path $iconsDir | Out-Null
New-Item -ItemType Directory -Force -Path $fontsDir | Out-Null

# Copy icons, logo, and bundled Bangladeshi fonts
Copy-Item -Force (Join-Path $repoRoot "logo.png") $installDir
Copy-Item -Force (Join-Path $repoRoot "assets\icons\*") $iconsDir
Copy-Item -Force (Join-Path $repoRoot "assets\fonts\*") $fontsDir
Write-Host "  [OK] Copied icons and Bangladeshi fonts (Kalpurush, SolaimanLipi, Nikosh, Siyam Rupali)" -ForegroundColor Green

# Also install Bangladeshi fonts into user's Windows Fonts so all apps on the PC render Bangla cleanly
$userWinFontsDir = Join-Path $env:LOCALAPPDATA "Microsoft\Windows\Fonts"
New-Item -ItemType Directory -Force -Path $userWinFontsDir | Out-Null
$fontRegKey = "HKCU:\Software\Microsoft\Windows NT\CurrentVersion\Fonts"
if (-not (Test-Path $fontRegKey)) {
    New-Item -Path $fontRegKey -Force | Out-Null
}
$banglaFontMap = @{
    "Kalpurush (TrueType)"    = "Kalpurush.ttf"
    "SolaimanLipi (TrueType)" = "SolaimanLipi.ttf"
    "Nikosh (TrueType)"       = "Nikosh.ttf"
    "Siyam Rupali (TrueType)" = "SiyamRupali.ttf"
    "Hind Siliguri (TrueType)"= "HindSiliguri-Regular.ttf"
}
foreach ($kv in $banglaFontMap.GetEnumerator()) {
    $srcFont = Join-Path $fontsDir $kv.Value
    $dstFont = Join-Path $userWinFontsDir $kv.Value
    if (Test-Path $srcFont) {
        Copy-Item -Force $srcFont $dstFont -ErrorAction SilentlyContinue
        Set-ItemProperty -Path $fontRegKey -Name $kv.Key -Value $dstFont -ErrorAction SilentlyContinue
    }
}
Write-Host "  [OK] Installed Bangladeshi Unicode fonts to User Windows Fonts (Kalpurush default)" -ForegroundColor Green

# Prefer release binaries if present, otherwise debug binaries
$targetDir = Join-Path $repoRoot "target\release"
if (-not (Test-Path (Join-Path $targetDir "ekushe.exe"))) {
    $targetDir = Join-Path $repoRoot "target\debug"
}

foreach ($bin in @("ekushe.exe", "ekushe-cli.exe", "ekushe_tsf.dll", "ekushe_ffi.dll")) {
    $src = Join-Path $targetDir $bin
    if (Test-Path $src) {
        Copy-Item -Force $src $installDir
        Write-Host "  [OK] Copied $bin" -ForegroundColor Green
    }
}

# Register per-user TSF DLL (no admin UAC required since it writes to HKCU\Software\Classes)
$tsfDll = Join-Path $installDir "ekushe_tsf.dll"
if (Test-Path $tsfDll) {
    & regsvr32.exe /s $tsfDll
    Write-Host "  [OK] Registered Ekushe Next TSF Text Service (HKCU)" -ForegroundColor Green
}

# Create Start Menu shortcut with ekushe.ico
$wsh = New-Object -ComObject WScript.Shell
$exePath = Join-Path $installDir "ekushe.exe"
$icoPath = Join-Path $iconsDir "ekushe.ico"

$startMenuShortcut = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\Ekushe Next Bangla Keyboard.lnk"
$sc1 = $wsh.CreateShortcut($startMenuShortcut)
$sc1.TargetPath = $exePath
$sc1.WorkingDirectory = $installDir
$sc1.IconLocation = $icoPath
$sc1.Description = "Ekushe Next Bangla Keyboard & Spell Checker"
$sc1.Save()
Write-Host "  [OK] Created Start Menu shortcut: $startMenuShortcut" -ForegroundColor Green

if ($EnableStartup) {
    $runKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
    Set-ItemProperty -Path $runKey -Name "EkusheBanglaKeyboard" -Value "`"$exePath`" --minimized"
    Write-Host "  [OK] Enabled automatic launch in Windows System Tray at login" -ForegroundColor Green
}

Write-Host "`n[SUCCESS] Ekushe Next installed with Kalpurush Bangladeshi font! Double-Tap Shift anytime to switch between Normal Keyboard (EN) and Ekushe Next (Bangla)." -ForegroundColor Yellow

if (-not $NoLaunch -and (Test-Path $exePath)) {
    Start-Process -FilePath $exePath -WorkingDirectory $installDir
}
