param([Parameter(Mandatory=$true)][string]$Path,[Parameter(Mandatory=$true)][string]$ExpectedThumbprint,[Parameter(Mandatory=$true)][string]$ExpectedSubject)
$ErrorActionPreference='Stop'
$signature=Get-AuthenticodeSignature -LiteralPath $Path
if($signature.Status -ne 'Valid'){throw 'Authenticode verification failed'}
if($signature.SignerCertificate.Thumbprint -ne $ExpectedThumbprint){throw 'Unexpected signing certificate'}
if($signature.SignerCertificate.Subject -ne $ExpectedSubject){throw 'Unexpected legal publisher subject'}
if($null -eq $signature.TimeStamperCertificate){throw 'Release must be timestamped'}
Write-Output 'Valid expected Windows publisher signature and timestamp.'
