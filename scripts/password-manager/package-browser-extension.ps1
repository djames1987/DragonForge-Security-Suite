param(
    [string]$OutputDirectory = "dist"
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$RepoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
Set-Location $RepoRoot

node --check extensions/password-manager-browser/src/core.js
if ($LASTEXITCODE -ne 0) { throw "Extension core syntax check failed." }
node --check extensions/password-manager-browser/src/background.js
if ($LASTEXITCODE -ne 0) { throw "Extension background syntax check failed." }
node --check extensions/password-manager-browser/ui/popup.js
if ($LASTEXITCODE -ne 0) { throw "Extension popup syntax check failed." }
node --test extensions/password-manager-browser/tests/*.test.mjs
if ($LASTEXITCODE -ne 0) { throw "Extension tests failed." }

$OutputRoot = Join-Path $RepoRoot $OutputDirectory
New-Item -ItemType Directory -Force -Path $OutputRoot | Out-Null
$ZipPath = Join-Path $OutputRoot "dragonforge-browser-extension.zip"
Remove-Item -Force -ErrorAction SilentlyContinue $ZipPath

$ExtensionRoot = Join-Path $RepoRoot "extensions\password-manager-browser"
$ManifestPath = Join-Path $ExtensionRoot "manifest.json"
if (-not (Test-Path -LiteralPath $ManifestPath)) { throw "Browser extension manifest is missing." }
$Manifest = Get-Content -Raw -LiteralPath $ManifestPath | ConvertFrom-Json
if ($Manifest.manifest_version -ne 3) { throw "Browser extension must remain Manifest V3." }
if ($null -ne $Manifest.host_permissions) { throw "Browser extension must not declare broad host_permissions." }
if (@($Manifest.permissions) -contains "<all_urls>") { throw "Browser extension permissions are unexpectedly broad." }

$Files = @(
    $ManifestPath,
    (Join-Path $ExtensionRoot "src"),
    (Join-Path $ExtensionRoot "ui")
)
Compress-Archive -Path $Files -DestinationPath $ZipPath -CompressionLevel Optimal

Write-Host "Browser extension package created:"
Write-Host "  $ZipPath"
