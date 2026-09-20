$ErrorActionPreference = 'Stop'
$setupRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$toolRoot = Join-Path $setupRoot '.tools'
New-Item -ItemType Directory -Force -Path $toolRoot | Out-Null
$bootstrap = Join-Path $toolRoot 'rustup-init.exe'
$uri = 'https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe'
Invoke-WebRequest -UseBasicParsing $uri -OutFile $bootstrap
Invoke-WebRequest -UseBasicParsing "$uri.sha256" -OutFile "$bootstrap.sha256"
$expected = ([IO.File]::ReadAllText("$bootstrap.sha256") -split '\s+')[0].Trim()
$actual = (Get-FileHash -LiteralPath $bootstrap -Algorithm SHA256).Hash.ToLowerInvariant()
if ($actual -ne $expected) { throw 'Rust bootstrap checksum mismatch' }
$env:CARGO_HOME = Join-Path $toolRoot 'cargo'
$env:RUSTUP_HOME = Join-Path $toolRoot 'rustup'
& $bootstrap -y --no-modify-path --profile minimal --default-toolchain 1.90.0 --default-host x86_64-pc-windows-msvc
if ($LASTEXITCODE -ne 0) { throw 'Rust toolchain installation failed' }
