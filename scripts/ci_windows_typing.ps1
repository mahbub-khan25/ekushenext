# Type fast into Notepad through the running tray app, with the suggestion popup redrawing on every
# key, and check what reached Notepad. A key that waited too long in the keyboard hook used to reach
# the app unconverted (a Latin "e" inside a Bangla word, letters typed twice).
#   pwsh scripts/ci_windows_typing.ps1 target\release\ekushe.exe
param([string]$Exe = 'target\release\ekushe.exe')
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms
Add-Type @'
using System; using System.Runtime.InteropServices;
public static class W {
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
}
'@

$tray = Start-Process -FilePath $Exe -ArgumentList '--minimized' -PassThru
Start-Sleep -Seconds 5
$np = Start-Process notepad -PassThru
Start-Sleep -Seconds 3
[void][W]::SetForegroundWindow($np.MainWindowHandle)
Start-Sleep -Milliseconds 500

# Bangla on (Ctrl+Space), then the sentence, as fast as SendKeys sends it
[System.Windows.Forms.SendKeys]::SendWait('^ ')
Start-Sleep -Milliseconds 800
$text = 'amader deshe shopnopuree ami bangla likhi '
$want = 'আমাদের দেশে স্বপ্নপুরী আমি বাংলা লিখি '
[System.Windows.Forms.SendKeys]::SendWait($text)
Start-Sleep -Seconds 2
# and once more, a letter at a time with no pause between keys
foreach ($c in $text.ToCharArray()) { [System.Windows.Forms.SendKeys]::SendWait([string]$c) }
Start-Sleep -Seconds 2

[System.Windows.Forms.SendKeys]::SendWait('^ ')   # back to English for the shortcuts
Start-Sleep -Milliseconds 500
Set-Clipboard -Value ''
[System.Windows.Forms.SendKeys]::SendWait('^(a)')
Start-Sleep -Milliseconds 300
[System.Windows.Forms.SendKeys]::SendWait('^(c)')
Start-Sleep -Milliseconds 800
$got = (Get-Clipboard -Raw)
Stop-Process -Id $np.Id -Force
Stop-Process -Id $tray.Id -Force
Write-Host "typed: $text (twice)"
Write-Host "got:   $got"
if ($got -ne ($want + $want)) {
  Write-Host "::error::Notepad got '$got', expected '$want$want'"
  exit 1
}
Write-Host 'fast typing into Notepad OK'
