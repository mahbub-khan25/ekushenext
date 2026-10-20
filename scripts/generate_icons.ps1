# Generates all Windows .ico and .png branding assets from logo.png
$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing

$root = Split-Path -Parent $PSScriptRoot
$srcPath = Join-Path $root "logo.png"
$outDir = Join-Path $root "assets\icons"
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

$src = [System.Drawing.Bitmap]::FromFile($srcPath)
Write-Host "Loaded logo.png: $($src.Width)x$($src.Height) ($($src.PixelFormat))"

# The inner rounded-square badge in the 1024x1024 logo sits around [136..888, 136..888] (752x752).
# Cropping tightly to the squircle with a transparent rounded-rectangle mask makes the icon look
# crisp and fills the full icon slot at 16x16, 24x24, 32x32, 48x48, and 256x256 in the System Tray and Taskbar.
$cropRect = New-Object System.Drawing.Rectangle(136, 136, 752, 752)

function New-RoundedBadgeBitmap {
    param(
        [System.Drawing.Bitmap]$Source,
        [System.Drawing.Rectangle]$Crop,
        [int]$Size,
        [string]$BadgeText = "",
        [System.Drawing.Color]$BadgeBg = [System.Drawing.Color]::Empty
    )

    $bmp = New-Object System.Drawing.Bitmap($Size, $Size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $g.Clear([System.Drawing.Color]::Transparent)

    # Create rounded squircle clip path (radius ~ 22% of size)
    $r = [Math]::Max(4, [int]($Size * 0.22))
    $d = $r * 2
    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $path.AddArc(0, 0, $d, $d, 180, 90)
    $path.AddArc($Size - $d, 0, $d, $d, 270, 90)
    $path.AddArc($Size - $d, $Size - $d, $d, $d, 0, 90)
    $path.AddArc(0, $Size - $d, $d, $d, 90, 90)
    $path.CloseFigure()

    $g.SetClip($path)
    $destRect = New-Object System.Drawing.Rectangle(0, 0, $Size, $Size)
    $g.DrawImage($Source, $destRect, $Crop, [System.Drawing.GraphicsUnit]::Pixel)
    $g.ResetClip()

    # Optional corner pill badge for Tray Status (e.g., बांग / EN indicator)
    if ($BadgeText -ne "") {
        $pillSize = [Math]::Max(8, [int]($Size * 0.44))
        $pillX = $Size - $pillSize
        $pillY = $Size - $pillSize
        $brush = New-Object System.Drawing.SolidBrush($BadgeBg)
        $pen = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(240, 245, 220, 144), [Math]::Max(1.0, $Size * 0.04))
        $g.FillEllipse($brush, $pillX, $pillY, $pillSize - 1, $pillSize - 1)
        $g.DrawEllipse($pen, $pillX, $pillY, $pillSize - 1, $pillSize - 1)
        $pen.Dispose()
        $brush.Dispose()
    }

    $path.Dispose()
    $g.Dispose()
    return $bmp
}

# Write a valid multi-image Windows .ICO file containing PNG-encoded frames (Vista+ standard)
function Write-MultiSizeIco {
    param(
        [string]$Path,
        [System.Drawing.Bitmap[]]$Bitmaps
    )
    $pngBlobs = New-Object System.Collections.Generic.List[byte[]]
    foreach ($b in $Bitmaps) {
        $ms = New-Object System.IO.MemoryStream
        $b.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
        $pngBlobs.Add($ms.ToArray())
        $ms.Dispose()
    }

    $fs = [System.IO.File]::Open($Path, [System.IO.FileMode]::Create)
    $bw = New-Object System.IO.BinaryWriter($fs)

    $count = $Bitmaps.Count
    # ICONDIR header: reserved (0), type (1 = icon), count
    $bw.Write([uint16]0)
    $bw.Write([uint16]1)
    $bw.Write([uint16]$count)

    $offset = 6 + (16 * $count)
    for ($i = 0; $i -lt $count; $i++) {
        $w = $Bitmaps[$i].Width
        $h = $Bitmaps[$i].Height
        $bytes = $pngBlobs[$i]
        $bw.Write([byte]$(if ($w -ge 256) { 0 } else { $w }))
        $bw.Write([byte]$(if ($h -ge 256) { 0 } else { $h }))
        $bw.Write([byte]0)     # palette
        $bw.Write([byte]0)     # reserved
        $bw.Write([uint16]1)   # color planes
        $bw.Write([uint16]32)  # bits per pixel
        $bw.Write([uint32]$bytes.Length)
        $bw.Write([uint32]$offset)
        $offset += $bytes.Length
    }

    for ($i = 0; $i -lt $count; $i++) {
        $bw.Write($pngBlobs[$i])
    }

    $bw.Flush()
    $bw.Close()
    $fs.Close()
}

$sizes = @(16, 24, 32, 48, 64, 128, 256)
$mainBitmaps = @()
$banglaTrayBitmaps = @()
$englishTrayBitmaps = @()

foreach ($sz in $sizes) {
    $b = New-RoundedBadgeBitmap -Source $src -Crop $cropRect -Size $sz
    $mainBitmaps += $b
    $pngPath = Join-Path $outDir "logo_$sz.png"
    $b.Save($pngPath, [System.Drawing.Imaging.ImageFormat]::Png)

    # Bangla active tray icon (crimson/emerald active dot)
    $bBn = New-RoundedBadgeBitmap -Source $src -Crop $cropRect -Size $sz -BadgeText "BN" -BadgeBg ([System.Drawing.Color]::FromArgb(255, 225, 37, 53))
    $banglaTrayBitmaps += $bBn

    # English passthrough tray icon (cool slate dot)
    $bEn = New-RoundedBadgeBitmap -Source $src -Crop $cropRect -Size $sz -BadgeText "EN" -BadgeBg ([System.Drawing.Color]::FromArgb(255, 70, 90, 110))
    $englishTrayBitmaps += $bEn
}

# Also save 512x512 high-DPI PNG for the companion & welcome header
$b512 = New-RoundedBadgeBitmap -Source $src -Crop $cropRect -Size 512
$b512.Save((Join-Path $outDir "logo_512.png"), [System.Drawing.Imaging.ImageFormat]::Png)
$b512.Dispose()

Write-MultiSizeIco -Path (Join-Path $outDir "ekushe.ico") -Bitmaps $mainBitmaps
Write-MultiSizeIco -Path (Join-Path $outDir "tray_bangla.ico") -Bitmaps $banglaTrayBitmaps
Write-MultiSizeIco -Path (Join-Path $outDir "tray_english.ico") -Bitmaps $englishTrayBitmaps

foreach ($b in $mainBitmaps) { $b.Dispose() }
foreach ($b in $banglaTrayBitmaps) { $b.Dispose() }
foreach ($b in $englishTrayBitmaps) { $b.Dispose() }
$src.Dispose()

Write-Host "Generated icons in $outDir :"
Get-ChildItem $outDir | Select-Object Name, Length | Format-Table -AutoSize
