param([string]$Installer)
$ErrorActionPreference = 'Stop'
$setupRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$config = Get-Content -LiteralPath (Join-Path $setupRoot 'src-tauri/tauri.conf.json') -Raw | ConvertFrom-Json
if ($config.identifier -ne 'app.villow.setup.dev') { throw 'This smoke test is for the development identifier only.' }
if (-not $Installer) {
    $found = @(Get-ChildItem -LiteralPath (Join-Path $setupRoot 'src-tauri/target/release/bundle/nsis') -Filter '*-setup.exe')
    if ($found.Count -ne 1) { throw 'Expected exactly one generated development installer.' }
    $Installer = $found[0].FullName
}
$Installer = (Resolve-Path -LiteralPath $Installer).Path
foreach ($hive in @('HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall', 'HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall')) {
    if (Test-Path $hive) {
        foreach ($entry in Get-ChildItem $hive) {
            $product = Get-ItemProperty -LiteralPath $entry.PSPath
            if ($product.DisplayName -eq 'Villow Setup') { throw 'An existing Villow Setup installation was found. Use a clean test profile.' }
        }
    }
}
$artifactRoot = Join-Path $setupRoot 'artifacts'
New-Item -ItemType Directory -Path $artifactRoot -Force | Out-Null
$installRoot = [IO.Path]::GetFullPath((Join-Path $artifactRoot ('installed-smoke-' + [guid]::NewGuid().ToString('N'))))
if (-not $installRoot.StartsWith($artifactRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw 'Unexpected install destination.' }
$evidence = [ordered]@{ installer = $Installer; sha256 = (Get-FileHash -LiteralPath $Installer -Algorithm SHA256).Hash; signature = (Get-AuthenticodeSignature -LiteralPath $Installer).Status.ToString(); destination = $installRoot; checks = @(); visual_verification = $false }
$launched = $null
try {
    # NSIS requires /D to be last; it consumes the rest of the command line,
    # including spaces. No shell or path interpolation inside the installer.
    $install = Start-Process -FilePath $Installer -ArgumentList ('/S /D=' + $installRoot) -WindowStyle Hidden -PassThru -Wait
    if ($install.ExitCode -ne 0) { throw 'Development installer failed.' }
    $exe = Join-Path $installRoot 'villow-setup.exe'
    if (-not (Test-Path -LiteralPath $exe)) { throw 'Installed executable missing.' }
    $evidence.checks += 'NSIS current-user installation completed'
    foreach ($attempt in 1..2) {
        $launched = Start-Process -FilePath $exe -WindowStyle Hidden -PassThru
        Start-Sleep -Seconds 5
        $launched.Refresh()
        if ($launched.HasExited) { throw 'Installed application exited during launch.' }
        $evidence.checks += ('Installed application remained running on launch ' + $attempt)
        # Deliberately interrupt this exact process to exercise reopening.
        Stop-Process -Id $launched.Id
        $launched = $null
    }
} finally {
    if ($null -ne $launched) { Stop-Process -Id $launched.Id -ErrorAction SilentlyContinue }
    $uninstaller = Join-Path $installRoot 'uninstall.exe'
    if (Test-Path -LiteralPath $uninstaller) {
        $uninstall = Start-Process -FilePath $uninstaller -ArgumentList '/S' -WindowStyle Hidden -PassThru -Wait
        $evidence.checks += ('Development uninstall returned ' + $uninstall.ExitCode)
    }
    $evidence | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $artifactRoot 'package-smoke.json')
}
$evidence | ConvertTo-Json -Depth 8
