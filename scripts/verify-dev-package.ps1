$ErrorActionPreference = 'Stop'
$taskRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$taskVersion = (Get-Content -LiteralPath (Join-Path $taskRoot 'package.json') -Raw | ConvertFrom-Json).version
if ($taskVersion -notmatch '^\d+\.\d+\.\d+$') { throw 'Unexpected package version' }
$taskInstaller = Join-Path $taskRoot ("src-tauri\target\release\bundle\nsis\Villow Setup_" + $taskVersion + '_x64-setup.exe')
$taskArtifactRoot = (Resolve-Path (Join-Path $taskRoot 'artifacts')).Path
$taskDestination = [IO.Path]::GetFullPath((Join-Path $taskArtifactRoot ('installed-integration-' + [guid]::NewGuid().ToString('N'))))
if (-not $taskDestination.StartsWith($taskArtifactRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw 'Test destination escaped artifact root' }
if (Test-Path -LiteralPath $taskDestination) { throw 'Test destination already exists' }
if (Get-Process -Name villow-setup -ErrorAction SilentlyContinue) { throw 'Close the existing Villow Setup process before this isolated test' }
$taskExisting = Get-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*' -ErrorAction SilentlyContinue | Where-Object { $_.DisplayName -eq 'Villow Setup' }
if ($taskExisting) { throw 'An existing Villow Setup installation is registered; refusing to replace it for a test' }
$taskChecks = @()
try {
  $taskInstall = Start-Process -FilePath $taskInstaller -ArgumentList "/S /D=$taskDestination" -WindowStyle Hidden -PassThru
  if (-not $taskInstall.WaitForExit(60000)) { throw 'Development installation timed out' }
  if ($taskInstall.ExitCode -ne 0) { throw "Installation failed: $($taskInstall.ExitCode)" }
  $taskExecutable = Join-Path $taskDestination 'villow-setup.exe'
  if (-not (Test-Path -LiteralPath $taskExecutable)) { throw 'Installed executable missing' }
  foreach ($taskNotice in @('LICENSE.txt', 'privacy.md', 'code-signing-policy.md')) {
    if (-not (Test-Path -LiteralPath (Join-Path $taskDestination $taskNotice))) { throw "Installed notice missing: $taskNotice" }
  }
  if ((Get-FileHash -LiteralPath (Join-Path $taskDestination 'LICENSE.txt')).Hash -ne (Get-FileHash -LiteralPath (Join-Path $taskRoot 'LICENSE')).Hash) { throw 'Installed license differs from the project notice' }
  $taskChecks += 'MIT license, privacy notice and development signing policy installed'
  $taskChecks += 'NSIS current-user installation completed'
  foreach ($taskLaunch in 1..2) {
    $taskProcess = Start-Process -FilePath $taskExecutable -WindowStyle Hidden -PassThru
    try {
      Start-Sleep -Seconds 5
      $taskProcess.Refresh()
      if ($taskProcess.HasExited) { throw "Installed application exited during launch $taskLaunch" }
      $taskChecks += "Installed application remained running on launch $taskLaunch"
    } finally {
      if (-not $taskProcess.HasExited) { Stop-Process -Id $taskProcess.Id; $taskProcess.WaitForExit(10000) | Out-Null }
    }
  }
} finally {
  $taskUninstaller = Join-Path $taskDestination 'uninstall.exe'
  if (Test-Path -LiteralPath $taskUninstaller) {
    $taskUninstall = Start-Process -FilePath $taskUninstaller -ArgumentList '/S' -WindowStyle Hidden -PassThru
    if (-not $taskUninstall.WaitForExit(60000) -or $taskUninstall.ExitCode -ne 0) { throw 'Development test uninstall did not complete' }
    $taskChecks += 'Development uninstall returned 0'
    # NSIS may return after launching its temporary uninstaller copy. Removing
    # the installation folder and uninstall registration are separate steps.
    $taskRemovalDeadline = [DateTime]::UtcNow.AddSeconds(20)
    do {
      $taskRemaining = Get-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*' -ErrorAction SilentlyContinue | Where-Object { $_.DisplayName -eq 'Villow Setup' }
      if (-not (Test-Path -LiteralPath $taskDestination) -and -not $taskRemaining) { break }
      Start-Sleep -Milliseconds 250
    } while ([DateTime]::UtcNow -lt $taskRemovalDeadline)
    if (Test-Path -LiteralPath $taskDestination) { throw 'Development uninstall left the test installation directory behind' }
    if ($taskRemaining) { throw 'Development uninstall left its registration behind' }
    $taskChecks += 'Test installation directory and current-user uninstall registration removed'
  }
}
@{
  installer = $taskInstaller
  sha256 = (Get-FileHash -LiteralPath $taskInstaller -Algorithm SHA256).Hash
  signature = (Get-AuthenticodeSignature -LiteralPath $taskInstaller).Status.ToString()
  destination = $taskDestination
  checks = $taskChecks
  visual_verification = $false
} | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $taskArtifactRoot 'integration-package-smoke.json')
Write-Output 'Unsigned development package installed, launched twice and uninstalled. Visual review is not claimed.'
