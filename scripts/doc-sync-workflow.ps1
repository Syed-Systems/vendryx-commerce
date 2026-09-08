# scripts/doc-sync-workflow.ps1 — Documentation sync and version logging wrapper
[CmdletBinding()]
param (
    [Parameter(Position = 0)]
    [ValidateSet("check", "log")]
    [string]$Command = "check",

    [string]$Version,
    [string]$Summary,
    [string]$Surfaces,
    [string]$Gates
)

$repoRoot = Split-Path -Parent $PSScriptRoot

if ($Command -eq "check") {
    python "$repoRoot/scripts/doc-sync-workflow.py" check
    exit $LASTEXITCODE
}
elseif ($Command -eq "log") {
    if (-not $Version -or -not $Summary) {
        Write-Error "Both -Version and -Summary are required for logging."
        exit 1
    }
    python "$repoRoot/scripts/doc-sync-workflow.py" log --version $Version --summary $Summary --surfaces "$Surfaces" --gates "$Gates"
    exit $LASTEXITCODE
}
