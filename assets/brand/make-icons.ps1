<#
.SYNOPSIS
  Renders the Leafy PDF app icons from the shapes in leafy-logo.svg and leafy-icon-small.svg.

.DESCRIPTION
  Windows PowerShell 5.1 or later (System.Drawing). Draws every size directly at its pixel size, so edges stay crisp.
  Sizes up to 48 px use the simplified small icon; larger sizes use the full logo.
  Writes ../app-icon/leafypdf.ico (16-256 px, PNG entries), leafypdf-256.png and leafypdf-1024.png.
#>
param([string] $OutDir = (Join-Path $PSScriptRoot '..\app-icon'))
Add-Type -AssemblyName System.Drawing
$ErrorActionPreference = 'Stop'

function Color([string] $hex) { [Drawing.ColorTranslator]::FromHtml($hex) }
$Teal = Color '#0F766E'; $Mint = Color '#99F6E4'; $White = Color '#FFFFFF'; $Yellow = Color '#FDE047'

function RoundedRect([double]$x, [double]$y, [double]$w, [double]$h, [double]$r) {
  $p = New-Object Drawing.Drawing2D.GraphicsPath
  $d = 2 * $r
  $p.AddArc($x, $y, $d, $d, 180, 90); $p.AddArc($x + $w - $d, $y, $d, $d, 270, 90)
  $p.AddArc($x + $w - $d, $y + $h - $d, $d, $d, 0, 90); $p.AddArc($x, $y + $h - $d, $d, $d, 90, 90)
  $p.CloseFigure(); $p
}
function Fill($g, [string]$hex, [double]$x, [double]$y, [double]$w, [double]$h, [double]$r) {
  $brush = New-Object Drawing.SolidBrush (Color $hex); $path = RoundedRect $x $y $w $h $r
  $g.FillPath($brush, $path); $brush.Dispose(); $path.Dispose()
}
function Rotated($g, [double]$angle, [double]$cx, [double]$cy, [scriptblock]$draw) {
  $saved = $g.Transform
  $g.TranslateTransform($cx, $cy); $g.RotateTransform($angle); $g.TranslateTransform(-$cx, -$cy)
  & $draw
  $g.Transform = $saved
}

# The full logo, in the SVG's own units: its viewBox is 100 30 180 180.
function Draw-Large($g) {
  Fill $g '#0F766E' 100 30 180 180 48
  Rotated $g -8 189 118 { Fill $g '#99F6E4' 146 64 86 108 10 }
  Fill $g '#FFFFFF' 150 62 86 108 10
  Fill $g '#0F766E' 162 80 40 8 4
  Fill $g '#FDE047' 162 98 62 14 3
  Fill $g '#0F766E' 166 102 54 6 3
  Fill $g '#99F6E4' 162 122 52 7 3
  Fill $g '#99F6E4' 162 138 34 7 3
  Fill $g '#0F766E' 200 134 3 16 1.5
  $pen = New-Object Drawing.Pen (Color '#0F766E'), 2
  $pen.StartCap = 'Round'; $pen.EndCap = 'Round'
  $g.DrawLine($pen, 196, 134, 207, 134); $g.DrawLine($pen, 196, 150, 207, 150); $pen.Dispose()
}
# The simplified small icon: viewBox 0 0 64 64.
function Draw-Small($g) {
  Fill $g '#0F766E' 0 0 64 64 18
  Rotated $g -8 31 33 { Fill $g '#99F6E4' 16 14 30 38 5 }
  Fill $g '#FFFFFF' 18 13 30 38 5
  Fill $g '#FDE047' 23 26 20 6 2
  Fill $g '#99F6E4' 23 37 16 3 1.5
}

function Render([int]$size, [bool]$small) {
  $bmp = New-Object Drawing.Bitmap $size, $size, ([Drawing.Imaging.PixelFormat]::Format32bppArgb)
  $g = [Drawing.Graphics]::FromImage($bmp)
  $g.SmoothingMode = 'AntiAlias'; $g.PixelOffsetMode = 'HighQuality'; $g.Clear([Drawing.Color]::Transparent)
  if ($small) { $g.ScaleTransform($size / 64.0, $size / 64.0); Draw-Small $g }
  else { $g.ScaleTransform($size / 180.0, $size / 180.0); $g.TranslateTransform(-100, -30); Draw-Large $g }
  $g.Dispose(); $bmp
}
# The leading comma keeps PowerShell from unrolling the array into the pipeline as separate bytes.
function PngBytes($bmp) { $ms = New-Object IO.MemoryStream; $bmp.Save($ms, [Drawing.Imaging.ImageFormat]::Png); , $ms.ToArray() }

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
$OutDir = (Resolve-Path $OutDir).Path
$sizes = 16, 20, 24, 32, 40, 48, 64, 96, 128, 256
$entries = foreach ($s in $sizes) { $b = Render $s ($s -le 48); [byte[]]$bytes = PngBytes $b; $b.Dispose(); [pscustomobject]@{ Size = $s; Bytes = $bytes } }

# ICO: ICONDIR, one ICONDIRENTRY per image (PNG-compressed), then the images.
$ms = New-Object IO.MemoryStream; $w = New-Object IO.BinaryWriter $ms
$w.Write([uint16]0); $w.Write([uint16]1); $w.Write([uint16]$entries.Count)
$offset = 6 + 16 * $entries.Count
foreach ($e in $entries) {
  $dim = if ($e.Size -ge 256) { 0 } else { $e.Size }
  $w.Write([byte]$dim); $w.Write([byte]$dim); $w.Write([byte]0); $w.Write([byte]0)
  $w.Write([uint16]1); $w.Write([uint16]32); $w.Write([uint32]$e.Bytes.Length); $w.Write([uint32]$offset)
  $offset += $e.Bytes.Length
}
foreach ($e in $entries) { $w.Write([byte[]]$e.Bytes) }
$w.Flush()
[IO.File]::WriteAllBytes((Join-Path $OutDir 'leafypdf.ico'), $ms.ToArray())

foreach ($s in 256, 1024) { $b = Render $s $false; $b.Save((Join-Path $OutDir "leafypdf-$s.png"), [Drawing.Imaging.ImageFormat]::Png); $b.Dispose() }
"wrote leafypdf.ico ($($entries.Count) sizes: $($sizes -join ', ')), leafypdf-256.png, leafypdf-1024.png in $OutDir"

# A contact sheet for checking by eye (not shipped).
$sheet = New-Object Drawing.Bitmap 1000, 360
$g = [Drawing.Graphics]::FromImage($sheet); $g.Clear([Drawing.Color]::White); $g.InterpolationMode = 'NearestNeighbor'; $g.PixelOffsetMode = 'Half'
$x = 10
foreach ($s in 16, 24, 32, 48) { $b = Render $s $true; $g.DrawImage($b, $x, 10, $s * 4, $s * 4); $x += $s * 4 + 12; $b.Dispose() }
$g.InterpolationMode = 'HighQualityBicubic'
$b = Render 256 $false; $g.DrawImage($b, 720, 10, 256, 256); $b.Dispose()
$g.Dispose(); $sheet.Save((Join-Path $env:TEMP 'leafy-icon-sheet.png'), [Drawing.Imaging.ImageFormat]::Png)
"contact sheet: $(Join-Path $env:TEMP 'leafy-icon-sheet.png')"
