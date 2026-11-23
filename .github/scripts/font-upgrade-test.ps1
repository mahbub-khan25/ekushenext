# Shishir Bangla across upgrades, on a clean Ekushe (run after the install/uninstall test):
#  - an older Shishir Bangla that Ekushe installed is replaced by this version;
#  - a Shishir Bangla file Ekushe did not install is never touched, nor removed at uninstall.
$ErrorActionPreference = 'Stop'
$setup = "$env:RUNNER_TEMP\Ekushe-Setup.exe"
Copy-Item target\release\ekushe.exe $setup -Force
$root = "$env:LOCALAPPDATA\Programs\Ekushe"
$userFonts = "$env:LOCALAPPDATA\Microsoft\Windows\Fonts"
New-Item -ItemType Directory -Force $root, $userFonts | Out-Null

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
$bundled = [IO.File]::ReadAllBytes("assets\fonts\ShishirBangla-Regular.ttf")
$older = With-Revision $bundled 0x83
# An older Regular that an earlier Ekushe installed (listed as Ekushe's), and a Bold put there by hand
[IO.File]::WriteAllBytes("$userFonts\ShishirBangla-Regular.ttf", $older)
Set-Content -Path "$root\fonts-installed.txt" -Value 'ShishirBangla-Regular.ttf'
$mine = [Text.Encoding]::ASCII.GetBytes('a font the user installed')
[IO.File]::WriteAllBytes("$userFonts\ShishirBangla-Bold.ttf", $mine)

$p = Start-Process -FilePath $setup -ArgumentList '--auto-install', '--no-launch' -PassThru -NoNewWindow -Wait
Get-Content "$env:TEMP\ekushe-install.log" -Tail 8 -ErrorAction SilentlyContinue
if ($p.ExitCode -ne 0) { Write-Error "install failed ($($p.ExitCode))"; exit 1 }
$hash = { param($f) (Get-FileHash $f -Algorithm SHA256).Hash }
if ((& $hash "$userFonts\ShishirBangla-Regular.ttf") -ne (& $hash "assets\fonts\ShishirBangla-Regular.ttf")) { Write-Error "the older Shishir Bangla was not replaced"; exit 1 }
if ((& $hash "$userFonts\ShishirBangla-Bold.ttf") -ne ((Get-FileHash -InputStream ([IO.MemoryStream]::new($mine)) -Algorithm SHA256).Hash)) { Write-Error "a font Ekushe did not install was overwritten"; exit 1 }
if ((Get-Content "$root\fonts-installed.txt") -contains 'ShishirBangla-Bold.ttf') { Write-Error "the user's own font was listed as Ekushe's"; exit 1 }
Write-Host "older Shishir Bangla updated, the user's own file left alone"

$u = Start-Process -FilePath "$root\ekushe.exe" -ArgumentList '--uninstall' -PassThru -NoNewWindow -Wait
if ($u.ExitCode -ne 0) { exit $u.ExitCode }
if (-not (Test-Path "$userFonts\ShishirBangla-Bold.ttf")) { Write-Error "uninstall removed a font Ekushe did not install"; exit 1 }
Remove-Item -Force "$userFonts\ShishirBangla-Bold.ttf"
Remove-ItemProperty 'HKCU:\Software\Microsoft\Windows NT\CurrentVersion\Fonts' -Name 'Shishir Bangla Bold (TrueType)' -ErrorAction SilentlyContinue
Write-Host "uninstall kept the user's own font"
