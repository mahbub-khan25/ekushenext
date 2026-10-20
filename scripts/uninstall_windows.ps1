# Ekushe Next - Clean Windows Uninstaller
$ErrorActionPreference = "SilentlyContinue"
$installDir = Join-Path $env:LOCALAPPDATA "Programs\Ekushe"

Get-Process -Name "ekushe" -ErrorAction SilentlyContinue | Stop-Process -Force

$tsfDll = Join-Path $installDir "ekushe_tsf.dll"
if (Test-Path $tsfDll) {
    & regsvr32.exe /u /s $tsfDll
}

Remove-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "EkusheBanglaKeyboard" -ErrorAction SilentlyContinue
$startMenuShortcut = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\Ekushe Next Bangla Keyboard.lnk"
Remove-Item -Force $startMenuShortcut -ErrorAction SilentlyContinue
$oldShortcut = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\Ekushe Bangla Keyboard.lnk"
Remove-Item -Force $oldShortcut -ErrorAction SilentlyContinue

if (Test-Path $installDir) {
    Remove-Item -Recurse -Force $installDir
}

Write-Host "[OK] Ekushe Next has been cleanly uninstalled." -ForegroundColor Green
