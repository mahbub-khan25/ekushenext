# Shishir Bangla across upgrades, on a computer without Ekushe (CI runs it before the install and
# uninstall test):
#  - an older Shishir Bangla that an earlier Ekushe installed, registered and loaded (in use), is
#    replaced: the new version goes in under its own name and the font entry points at it, and the
#    old file is removed or, while Windows still has it open, scheduled for removal;
#  - a Shishir Bangla file the user put there, and a font entry the user made, are never touched;
#  - uninstall removes Ekushe's files and keeps the user's.
$ErrorActionPreference = 'Stop'
$setup = "$env:RUNNER_TEMP\Ekushe-Setup.exe"
Copy-Item target\release\ekushe.exe $setup -Force
$root = "$env:LOCALAPPDATA\Programs\Ekushe"
$userFonts = "$env:LOCALAPPDATA\Microsoft\Windows\Fonts"
$fontKey = 'HKCU:\Software\Microsoft\Windows NT\CurrentVersion\Fonts'
New-Item -ItemType Directory -Force $root, $userFonts | Out-Null
if (-not (Test-Path $fontKey)) { New-Item -Path $fontKey -Force | Out-Null }
Add-Type -Namespace EkusheTest -Name Gdi -MemberDefinition '[DllImport("gdi32.dll", CharSet = CharSet.Unicode)] public static extern int AddFontResourceW(string file);'

# `bytes` with the head table's font revision set to `rev` (an "older version" of the same font)
function With-Revision([byte[]]$bytes, [uint32]$rev) {
    $b = [byte[]]$bytes.Clone()
    $n = $b[4] * 256 + $b[5]
    for ($i = 0; $i -lt $n; $i++) {
        $rec = 12 + 16 * $i
        if ([Text.Encoding]::ASCII.GetString($b, $rec, 4) -eq 'head') {
            $off = (([uint32]$b[$rec + 8]) -shl 24) -bor (([uint32]$b[$rec + 9]) -shl 16) -bor (([uint32]$b[$rec + 10]) -shl 8) -bor $b[$rec + 11]
            $r = [BitConverter]::GetBytes($rev); [Array]::Reverse($r)
            [Array]::Copy($r, 0, $b, $off + 4, 4)
        }
    }
    return , $b
}
$hash = { param($f) (Get-FileHash $f -Algorithm SHA256).Hash }

# An older Regular as an earlier Ekushe installed it: its file, Ekushe's list, the font entry, loaded
$old = "$userFonts\ShishirBangla-Regular.ttf"
[IO.File]::WriteAllBytes($old, (With-Revision ([IO.File]::ReadAllBytes("assets\fonts\ShishirBangla-Regular.ttf")) 0x83))
Set-Content -Path "$root\fonts-installed.txt" -Value 'ShishirBangla-Regular.ttf'
Set-ItemProperty -Path $fontKey -Name 'Shishir Bangla (TrueType)' -Value $old
if ([EkusheTest.Gdi]::AddFontResourceW($old) -lt 1) { Write-Error "the older font could not be loaded"; exit 1 }
# The user's own: a Bold file put there by hand (no entry), and a SemiBold entry pointing elsewhere
$mine = "$userFonts\ShishirBangla-Bold.ttf"
[IO.File]::WriteAllBytes($mine, [Text.Encoding]::ASCII.GetBytes('a font the user installed'))
$mineHash = & $hash $mine
$theirSemiBold = "$env:RUNNER_TEMP\MyShishirBanglaSemiBold.ttf"
Copy-Item assets\fonts\ShishirBangla-SemiBold.ttf $theirSemiBold
Set-ItemProperty -Path $fontKey -Name 'Shishir Bangla SemiBold (TrueType)' -Value $theirSemiBold

$p = Start-Process -FilePath $setup -ArgumentList '--auto-install', '--no-launch' -PassThru -NoNewWindow -Wait
Get-Content "$env:TEMP\ekushe-install.log" -Tail 12 -ErrorAction SilentlyContinue
if ($p.ExitCode -ne 0) { Write-Error "install failed ($($p.ExitCode))"; exit 1 }
$reg = Get-ItemProperty $fontKey
$new = "$userFonts\ShishirBangla-Regular-0.003.ttf"
if ($reg.'Shishir Bangla (TrueType)' -ne $new) { Write-Error "the font entry still points at $($reg.'Shishir Bangla (TrueType)')"; exit 1 }
if ((& $hash $new) -ne (& $hash "assets\fonts\ShishirBangla-Regular.ttf")) { Write-Error "the new version was not installed"; exit 1 }
$log = Get-Content "$env:TEMP\ekushe-install.log" -Raw
if ((Test-Path $old) -and ($log -notmatch 'ShishirBangla-Regular\.ttf: in use, removal scheduled')) { Write-Error "the older file was neither removed nor scheduled"; exit 1 }
Write-Host "older Shishir Bangla (in use) replaced by ShishirBangla-Regular-0.003.ttf"
if ((& $hash $mine) -ne $mineHash) { Write-Error "a font Ekushe did not install was overwritten"; exit 1 }
if ((Get-Content "$root\fonts-installed.txt") -contains 'ShishirBangla-Bold.ttf') { Write-Error "the user's own font was listed as Ekushe's"; exit 1 }
if ($reg.'Shishir Bangla SemiBold (TrueType)' -ne $theirSemiBold) { Write-Error "the user's own font entry was changed"; exit 1 }
if (Test-Path "$userFonts\ShishirBangla-SemiBold-0.003.ttf") { Write-Error "SemiBold was installed over the user's own"; exit 1 }
Write-Host "the user's own Shishir Bangla file and font entry left alone"

$u = Start-Process -FilePath "$root\ekushe.exe" -ArgumentList '--uninstall' -PassThru -NoNewWindow -Wait
if ($u.ExitCode -ne 0) { exit $u.ExitCode }
if (-not (Test-Path $mine)) { Write-Error "uninstall removed a font Ekushe did not install"; exit 1 }
if ((Get-ItemProperty $fontKey).'Shishir Bangla SemiBold (TrueType)' -ne $theirSemiBold) { Write-Error "uninstall removed the user's font entry"; exit 1 }
if ((Get-ItemProperty $fontKey).'Shishir Bangla (TrueType)') { Write-Error "uninstall left Ekushe's font entry"; exit 1 }
$log = Get-Content "$env:TEMP\ekushe-install.log" -Raw
foreach ($f in Get-ChildItem "$userFonts\ShishirBangla-*-*.ttf", $old -ErrorAction SilentlyContinue) {
    if ($log -notmatch [regex]::Escape("$($f.Name): in use, removal scheduled")) { Write-Error "uninstall left $($f.Name)"; exit 1 }
}
if ((Test-Path $old) -and -not (Test-Path "$env:APPDATA\Ekushe\fonts-pending-removal.txt")) { Write-Error "a file waiting for removal is not listed for the next install"; exit 1 }
Write-Host "uninstall removed Ekushe's fonts (or scheduled them) and kept the user's"
Remove-Item -Force $mine, $theirSemiBold
Remove-ItemProperty $fontKey -Name 'Shishir Bangla SemiBold (TrueType)'
