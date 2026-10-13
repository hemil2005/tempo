$env:PATH = "D:\mingw64\bin;" + $env:PATH

$root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location -LiteralPath $root

$cmd = $args
if ($cmd.Count -eq 0) {
    $cmd = @("build")
}

& cargo @cmd
exit $LASTEXITCODE
