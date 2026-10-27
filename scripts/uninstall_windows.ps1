# Ekushe Next - Clean Windows Uninstaller
$ErrorActionPreference = "SilentlyContinue"
$installDir = Join-Path $env:LOCALAPPDATA "Programs\Ekushe"

# Ask a running Ekushe to quit (learned words are saved), then wait; force only as a last resort
$running = Get-Process -Name "ekushe" -ErrorAction SilentlyContinue
if ($running) {
    $running | ForEach-Object { $_.CloseMainWindow() | Out-Null }
    if (-not ($running | Wait-Process -Timeout 6 -ErrorAction SilentlyContinue)) {
        Get-Process -Name "ekushe" -ErrorAction SilentlyContinue | Stop-Process -Force
    }
}

# Only if a tester registered the input method (its COM class exists); otherwise regsvr32 would
# raise an admin prompt for nothing
$tsfDll = Join-Path $installDir "ekushe_tsf.dll"
if ((Test-Path $tsfDll) -and (Test-Path "HKLM:\Software\Classes\CLSID\{E21B0945-2102-1952-B09C-0AD55559B326}")) {
    & regsvr32.exe /u /s $tsfDll
}

Remove-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "EkusheBanglaKeyboard" -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\EkusheBanglaKeyboard" -ErrorAction SilentlyContinue

foreach ($dir in @((Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs"), [Environment]::GetFolderPath("Desktop"))) {
    Remove-Item -Force (Join-Path $dir "Ekushe Next Bangla Keyboard.lnk") -ErrorAction SilentlyContinue
    Remove-Item -Force (Join-Path $dir "Ekushe Bangla Keyboard.lnk") -ErrorAction SilentlyContinue
}

# Per-user fonts Ekushe installed (fonts-installed.txt lists only the ones it wrote itself; an
# older install without the list installed all five)
$fonts = @{
    "Kalpurush.ttf"            = "Kalpurush (TrueType)"
    "SolaimanLipi.ttf"         = "SolaimanLipi (TrueType)"
    "SiyamRupali.ttf"          = "Siyam Rupali (TrueType)"
    "HindSiliguri-Regular.ttf" = "Hind Siliguri (TrueType)"
    "HindSiliguri-Bold.ttf"    = "Hind Siliguri Bold (TrueType)"
}
$marker = Join-Path $installDir "fonts-installed.txt"
$ours = if (Test-Path $marker) { Get-Content $marker | Where-Object { $_.Trim() -ne "" } } else { $fonts.Keys }
$userFonts = Join-Path $env:LOCALAPPDATA "Microsoft\Windows\Fonts"
foreach ($file in $ours) {
    if ($fonts.ContainsKey($file)) {
        Remove-ItemProperty -Path "HKCU:\Software\Microsoft\Windows NT\CurrentVersion\Fonts" -Name $fonts[$file] -ErrorAction SilentlyContinue
        Remove-Item -Force (Join-Path $userFonts $file) -ErrorAction SilentlyContinue
    }
}

if (Test-Path $installDir) {
    Remove-Item -Recurse -Force $installDir
}

Write-Host "[OK] Ekushe Next has been cleanly uninstalled. Your settings and learned words in $env:APPDATA\Ekushe were kept." -ForegroundColor Green
