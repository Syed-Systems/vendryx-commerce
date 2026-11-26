[CmdletBinding()]
param(
    [string] $SpaceRepositoryRoot = (Join-Path (Split-Path -Parent (Split-Path -Parent $PSScriptRoot)) 'Space'),
    [ValidateSet('check', 'rust-ci', 'rust-package', 'web-admin', 'web-static', 'web-test')]
    [string] $Mode = 'check',
    [string[]] $CargoPackage,
    [switch] $CargoClippy,
    [string] $CargoTestTarget,
    [switch] $CargoTestServices
)

$ErrorActionPreference = 'Stop'

$storeRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$spaceRoot = (Resolve-Path -LiteralPath $SpaceRepositoryRoot).Path
$builder = Join-Path $spaceRoot 'scripts/build-on-space.ps1'

if (-not (Test-Path -LiteralPath $builder -PathType Leaf)) {
    throw "Space build entrypoint was not found: $builder"
}

# Keep build-container, image, transfer, and cache policy in Space's shared builder.
$builderArgs = @{
    Project = 'store'
    Mode = $Mode
    RepositoryRoot = $storeRoot
    HostAlias = 'space'
    RemoteUser = 'syed-build'
}
if ($CargoPackage) {
    $builderArgs.CargoPackage = $CargoPackage
}
if ($CargoClippy) {
    $builderArgs.CargoClippy = $true
}
if ($CargoTestTarget) {
    $builderArgs.CargoTestTarget = $CargoTestTarget
}
if ($CargoTestServices) {
    $builderArgs.CargoTestServices = $true
}
& $builder @builderArgs
