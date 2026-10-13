Add-Type -AssemblyName System.Drawing

$size = 256
$bmp = New-Object System.Drawing.Bitmap($size, $size)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
$g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality

# Clear background with transparent
$g.Clear([System.Drawing.Color]::Transparent)

# Background squircle / rounded rect
$rect = New-Object System.Drawing.Rectangle(8, 8, 240, 240)
$radius = 52
$path = New-Object System.Drawing.Drawing2D.GraphicsPath
$path.AddArc($rect.X, $rect.Y, $radius, $radius, 180, 90)
$path.AddArc($rect.Right - $radius, $rect.Y, $radius, $radius, 270, 90)
$path.AddArc($rect.Right - $radius, $rect.Bottom - $radius, $radius, $radius, 0, 90)
$path.AddArc($rect.X, $rect.Bottom - $radius, $radius, $radius, 90, 90)
$path.CloseFigure()

# Background gradient: Deep indigo to dark slate
$bgBrush = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
    (New-Object System.Drawing.PointF(0, 0)),
    (New-Object System.Drawing.PointF($size, $size)),
    [System.Drawing.Color]::FromArgb(255, 15, 23, 42),
    [System.Drawing.Color]::FromArgb(255, 30, 27, 75)
)
$g.FillPath($bgBrush, $path)

# Subtle border
$borderPen = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(120, 99, 102, 241), 3)
$g.DrawPath($borderPen, $path)

# Draw Tempo soundwave bars (centered, dynamic heights)
# Bars: 7 bars representing rhythmic tempo
$barHeights = @(45, 80, 130, 170, 120, 75, 40)
$barWidth = 14
$barGap = 16
$totalWidth = ($barHeights.Length * $barWidth) + (($barHeights.Length - 1) * $barGap)
$startX = ($size - $totalWidth) / 2
$centerY = $size / 2

for ($i = 0; $i -lt $barHeights.Length; $i++) {
    $h = $barHeights[$i]
    $x = $startX + ($i * ($barWidth + $barGap))
    $y = $centerY - ($h / 2)
    
    $barRect = New-Object System.Drawing.RectangleF($x, $y, $barWidth, $h)
    $barPath = New-Object System.Drawing.Drawing2D.GraphicsPath
    $r = $barWidth
    $barPath.AddArc($barRect.X, $barRect.Y, $r, $r, 180, 90)
    $barPath.AddArc($barRect.Right - $r, $barRect.Y, $r, $r, 270, 90)
    $barPath.AddArc($barRect.Right - $r, $barRect.Bottom - $r, $r, $r, 0, 90)
    $barPath.AddArc($barRect.X, $barRect.Bottom - $r, $r, $r, 90, 90)
    $barPath.CloseFigure()
    
    # Gradient for bars: Cyan (#38bdf8) to Purple/Violet (#a855f7)
    $pt1 = [System.Drawing.PointF]::new([float]$x, [float]$y)
    $pt2 = [System.Drawing.PointF]::new([float]$x, [float]($y + $h))
    $c1 = [System.Drawing.Color]::FromArgb(255, 56, 189, 248)
    $c2 = [System.Drawing.Color]::FromArgb(255, 168, 85, 247)
    $barBrush = [System.Drawing.Drawing2D.LinearGradientBrush]::new($pt1, $pt2, $c1, $c2)
    $g.FillPath($barBrush, $barPath)
    $barPath.Dispose()
    $barBrush.Dispose()
}

# Add subtle tempo rhythm dot / accent
$accentBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 244, 63, 94))
$g.FillEllipse($accentBrush, 120, 48, 16, 16)

$g.Flush()

# Save PNG
$outPng = "assets\icon.png"
$bmp.Save($outPng, [System.Drawing.Imaging.ImageFormat]::Png)
Write-Host "Saved $outPng"

# Generate multi-res ICO file
$sizes = @(256, 128, 64, 48, 32, 16)
$outIco = "assets\icon.ico"

# Save highest resolution as standard icon or stream
$iconStream = New-Object System.IO.MemoryStream
$bmp.Save($iconStream, [System.Drawing.Imaging.ImageFormat]::Png)
$pngBytes = $iconStream.ToArray()

# Construct standard Windows .ICO container holding the 256x256 PNG
$icoFs = New-Object System.IO.FileStream($outIco, [System.IO.FileMode]::Create)
$bw = New-Object System.IO.BinaryWriter($icoFs)

# ICONDIR header
$bw.Write([UInt16]0) # Reserved
$bw.Write([UInt16]1) # Type (1 = ICO)
$bw.Write([UInt16]1) # Number of images (1)

# ICONDIRENTRY (256x256)
$bw.Write([Byte]0)  # Width (0 = 256)
$bw.Write([Byte]0)  # Height (0 = 256)
$bw.Write([Byte]0)  # Color count
$bw.Write([Byte]0)  # Reserved
$bw.Write([UInt16]1) # Color planes
$bw.Write([UInt16]32) # Bits per pixel
$bw.Write([UInt32]$pngBytes.Length) # Image size in bytes
$bw.Write([UInt32]22) # Offset of image data (header 6 + entry 16 = 22)

# Write PNG payload
$bw.Write($pngBytes)
$bw.Flush()
$bw.Close()
$icoFs.Close()

Write-Host "Saved $outIco"

$g.Dispose()
$bmp.Dispose()
