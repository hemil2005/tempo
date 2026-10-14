$env:PATH = "D:\mingw64\bin;" + $env:PATH
$env:CC = "D:\mingw64\bin\gcc.exe"
$env:CXX = "D:\mingw64\bin\g++.exe"
$env:LIBOPUS_LIB_DIR = "D:\rust-audio-converter\thirdparty\install\lib"
$env:OPUS_STATIC = "1"

$root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location -LiteralPath $root

$cmd = $args
if ($cmd.Count -eq 0) {
    $cmd = @("build")
}

& cargo @cmd
exit $LASTEXITCODE
