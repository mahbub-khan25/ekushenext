# Type fast into Notepad through the running tray app, with the suggestion popup redrawing on every
# key, and check what reached Notepad. A key that waited too long in the keyboard hook used to reach
# the app unconverted (a Latin "e" inside a Bangla word, letters typed twice).
#   pwsh scripts/ci_windows_typing.ps1 target\release\ekushe.exe
param([string]$Exe = 'target\release\ekushe.exe')
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms
Add-Type @'
using System; using System.Runtime.InteropServices;
using System.Text;
public static class W {
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindowEx(IntPtr p, IntPtr a, string cls, string title);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr SendMessage(IntPtr h, int msg, IntPtr w, StringBuilder l);
  [DllImport("user32.dll")] public static extern IntPtr SendMessage(IntPtr h, int msg, IntPtr w, IntPtr l);
  /// The text of Notepad's edit box (classic Notepad: Edit; Windows 11: RichEditD2DPT)
  public static string Text(IntPtr top) {
    foreach (var cls in new[] { "Edit", "RichEditD2DPT" }) {
      IntPtr e = FindWindowEx(top, IntPtr.Zero, cls, null);
      if (e == IntPtr.Zero) { IntPtr ns = FindWindowEx(top, IntPtr.Zero, "NotepadTextBox", null); if (ns != IntPtr.Zero) e = FindWindowEx(ns, IntPtr.Zero, cls, null); }
      if (e == IntPtr.Zero) continue;
      int n = (int)SendMessage(e, 0x000E, IntPtr.Zero, IntPtr.Zero);
      var sb = new StringBuilder(n + 1);
      SendMessage(e, 0x000D, (IntPtr)(n + 1), sb);
      return sb.ToString();
    }
    return null;
  }
}
'@

# Start in Bangla (the default is English at login)
$cfgDir = Join-Path $env:APPDATA 'Ekushe'
New-Item -ItemType Directory -Force -Path $cfgDir | Out-Null
$cfg = Join-Path $cfgDir 'config.toml'
$old = if (Test-Path $cfg) { Get-Content -Raw $cfg } else { $null }
Set-Content -Path $cfg -Value 'start_language = "Bangla"' -Encoding utf8
$tray = Start-Process -FilePath $Exe -ArgumentList '--minimized' -PassThru
Start-Sleep -Seconds 5
$np = Start-Process notepad -PassThru
Start-Sleep -Seconds 3
[void][W]::SetForegroundWindow($np.MainWindowHandle)
Start-Sleep -Milliseconds 500

# the sentence, as fast as SendKeys sends it
$text = 'amader deshe shopnopuree ami bangla likhi '
$want = 'আমাদের দেশে স্বপ্নপুরী আমি বাংলা লিখি '
[System.Windows.Forms.SendKeys]::SendWait($text)
Start-Sleep -Seconds 2
# and once more, a letter at a time with no pause between keys
foreach ($c in $text.ToCharArray()) { [System.Windows.Forms.SendKeys]::SendWait([string]$c) }
Start-Sleep -Seconds 2

# what the screen shows (for the build's artifacts), then the text box's own text
Add-Type -AssemblyName System.Drawing
$b = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$bmp = New-Object System.Drawing.Bitmap $b.Width, $b.Height
[System.Drawing.Graphics]::FromImage($bmp).CopyFromScreen($b.Location, [System.Drawing.Point]::Empty, $b.Size)
New-Item -ItemType Directory -Force -Path popup-previews | Out-Null
$bmp.Save((Join-Path (Get-Location) 'popup-previews\typing-test-screen.png'))
$got = [W]::Text($np.MainWindowHandle)
Stop-Process -Id $np.Id -Force
Stop-Process -Id $tray.Id -Force
if ($null -ne $old) { Set-Content -Path $cfg -Value $old -NoNewline } else { Remove-Item $cfg }
Write-Host "typed: $text (twice)"
Write-Host "got:   $got"
if ($got -ne ($want + $want)) {
  Write-Host "::error::Notepad got '$got', expected '$want$want'"
  exit 1
}
Write-Host 'fast typing into Notepad OK'
