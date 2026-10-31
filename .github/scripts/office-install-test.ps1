# An office roll-out: the installer runs silently (/S) with a settings file made on a computer that is
# already set up. Checks the settings are in place and nothing asks the user anything.
$ErrorActionPreference = 'Stop'
$setup = "$env:RUNNER_TEMP\Ekushe-Setup.exe"
Copy-Item target\release\ekushe.exe $setup
$preset = "$env:RUNNER_TEMP\office-settings.toml"
Set-Content -Path $preset -Encoding utf8 -Value @'
active_layout = "UniJoy"
start_language = "Bangla"
welcome_done = true
'@
$p = Start-Process -FilePath $setup -ArgumentList '/S', '--no-launch', '--no-desktop-shortcut', '--settings', $preset -PassThru -NoNewWindow -Wait
Get-Content "$env:TEMP\ekushe-install.log" -Tail 12 -ErrorAction SilentlyContinue
if ($p.ExitCode -ne 0) { Write-Error "silent install failed ($($p.ExitCode))"; exit 1 }
$cfg = Get-Content "$env:APPDATA\Ekushe\config.toml" -Raw
if ($cfg -notmatch 'active_layout = "UniJoy"') { Write-Error "the office's layout was not set:`n$cfg"; exit 1 }
if ($cfg -notmatch 'welcome_done = true') { Write-Error "the welcome would still be asked:`n$cfg"; exit 1 }
if (Test-Path "$([Environment]::GetFolderPath('Desktop'))\Ekushe Next Bangla Keyboard.lnk") { Write-Error "--no-desktop-shortcut was ignored"; exit 1 }
if (-not (Test-Path "$env:LOCALAPPDATA\Programs\Ekushe\ekushe.exe")) { Write-Error "not installed"; exit 1 }
Write-Host "silent install with settings OK"

# The same with the settings file simply placed next to the installer (a shared folder)
New-Item -ItemType Directory -Force "$env:RUNNER_TEMP\share" | Out-Null
Copy-Item $setup "$env:RUNNER_TEMP\share\Ekushe-Setup.exe" -Force
$backup = '{"ekushe_backup":1,"files":{"config.toml":"active_layout = \"NationalJatiya\"\nwelcome_done = true\n"}}'
Set-Content -Path "$env:RUNNER_TEMP\share\ekushe-settings.json" -Encoding utf8 -Value $backup
$p = Start-Process -FilePath "$env:RUNNER_TEMP\share\Ekushe-Setup.exe" -ArgumentList '/S', '--no-launch' -PassThru -NoNewWindow -Wait
if ($p.ExitCode -ne 0) { Write-Error "install from the shared folder failed"; exit 1 }
$cfg = Get-Content "$env:APPDATA\Ekushe\config.toml" -Raw
if ($cfg -notmatch 'active_layout = "NationalJatiya"') { Write-Error "ekushe-settings.json next to the installer was not used:`n$cfg"; exit 1 }
Write-Host "settings file next to the installer OK"

# A broken settings file stops the install with an error instead of half-configuring
Set-Content -Path "$env:RUNNER_TEMP\bad.json" -Value '{"ekushe_backup":1,"files":{"config.toml":"max_candidates = \"x\""}}'
$p = Start-Process -FilePath $setup -ArgumentList '/S', '--no-launch', '--settings', "$env:RUNNER_TEMP\bad.json" -PassThru -NoNewWindow -Wait
if ($p.ExitCode -eq 0) { Write-Error "a broken settings file was accepted"; exit 1 }
Write-Host "broken settings file refused OK"

$u = Start-Process -FilePath "$env:LOCALAPPDATA\Programs\Ekushe\ekushe.exe" -ArgumentList '--uninstall' -PassThru -NoNewWindow -Wait
if ($u.ExitCode -ne 0) { exit $u.ExitCode }
Write-Host "uninstalled OK"
