Add-Type -AssemblyName System.Drawing

$size = 256
$bmp = New-Object System.Drawing.Bitmap($size, $size)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
$g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality

$g.Clear([System.Drawing.Color]::Transparent)

# Background squircle: Dark sleek tile (#121214) with subtle rounded corners
$rect = New-Object System.Drawing.Rectangle(12, 12, 232, 232)
$radius = 54
$path = New-Object System.Drawing.Drawing2D.GraphicsPath
$path.AddArc($rect.X, $rect.Y, $radius, $radius, 180, 90)
$path.AddArc($rect.Right - $radius, $rect.Y, $radius, $radius, 270, 90)
$path.AddArc($rect.Right - $radius, $rect.Bottom - $radius, $radius, $radius, 0, 90)
$path.AddArc($rect.X, $rect.Bottom - $radius, $radius, $radius, 90, 90)
$path.CloseFigure()

$bgBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 18, 18, 20))
$g.FillPath($bgBrush, $path)

# Subtle border
$borderPen = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(60, 255, 255, 255), 2)
$g.DrawPath($borderPen, $path)

# Render the exact 6 sound bars from music-bars.svg:
# viewBox: 24x24. Scale factor = 256 / 24
$scale = 256.0 / 24.0
$penWidth = 1.7 * $scale

$barPen = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(255, 245, 245, 247), $penWidth)
$barPen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
$barPen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round

# The 6 vertical paths from music-bars.svg:
# 1: M4 10V13
$g.DrawLine($barPen, [float](4.0 * $scale), [float](10.0 * $scale), [float](4.0 * $scale), [float](13.0 * $scale))
# 2: M7 7L7 16
$g.DrawLine($barPen, [float](7.0 * $scale), [float](7.0 * $scale), [float](7.0 * $scale), [float](16.0 * $scale))
# 3: M10 4L10 19
$g.DrawLine($barPen, [float](10.0 * $scale), [float](4.0 * $scale), [float](10.0 * $scale), [float](19.0 * $scale))
# 4: M13 7L13 16
$g.DrawLine($barPen, [float](13.0 * $scale), [float](7.0 * $scale), [float](13.0 * $scale), [float](16.0 * $scale))
# 5: M16 10V13
$g.DrawLine($barPen, [float](16.0 * $scale), [float](10.0 * $scale), [float](16.0 * $scale), [float](13.0 * $scale))
# 6: M19 7L19 16
$g.DrawLine($barPen, [float](19.0 * $scale), [float](7.0 * $scale), [float](19.0 * $scale), [float](16.0 * $scale))

$g.Flush()

# Save PNG
$outPng = "assets\icon.png"
$bmp.Save($outPng, [System.Drawing.Imaging.ImageFormat]::Png)
Write-Host "Saved $outPng from music-bars.svg"

# Save ICO
$outIco = "assets\icon.ico"
$iconStream = New-Object System.IO.MemoryStream
$bmp.Save($iconStream, [System.Drawing.Imaging.ImageFormat]::Png)
$pngBytes = $iconStream.ToArray()

$icoFs = New-Object System.IO.FileStream($outIco, [System.IO.FileMode]::Create)
$bw = New-Object System.IO.BinaryWriter($icoFs)
$bw.Write([UInt16]0)
$bw.Write([UInt16]1)
$bw.Write([UInt16]1)
$bw.Write([Byte]0)
$bw.Write([Byte]0)
$bw.Write([Byte]0)
$bw.Write([Byte]0)
$bw.Write([UInt16]1)
$bw.Write([UInt16]32)
$bw.Write([UInt32]$pngBytes.Length)
$bw.Write([UInt32]22)
$bw.Write($pngBytes)
$bw.Flush()
$bw.Close()
$icoFs.Close()
Write-Host "Saved $outIco from music-bars.svg"

$g.Dispose()
$bmp.Dispose()
