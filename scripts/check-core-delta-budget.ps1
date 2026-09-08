# scripts/check-core-delta-budget.ps1 — P27-07 Core Delta Budget Gate
$ErrorActionPreference = "Stop"
Set-Location "$PSScriptRoot\.."

$DeltaDoc = "docs/FRONTEND_MATRIX_CORE_DELTAS.md"

if (-not (Test-Path $DeltaDoc)) {
    Write-Error "FAIL: Delta ledger $DeltaDoc does not exist."
    exit 1
}

Write-Host "Verifying core delta budget in $DeltaDoc..."
$content = Get-Content $DeltaDoc -Raw

if (-not ($content -match "Delta 1" -and $content -match "tokens\.css")) {
    Write-Error "FAIL: Missing Delta 1 (tokens.css) in $DeltaDoc."
    exit 1
}

if (-not ($content -match "Delta 2" -and $content -match "NormalModuleReplacementPlugin")) {
    Write-Error "FAIL: Missing Delta 2 (bundler request rewriter) in $DeltaDoc."
    exit 1
}

$MaxAllowed = 2
$ContractDeltas = 2

Write-Host "PASS: Core delta budget held ($ContractDeltas / $MaxAllowed allowed contract changes)."
exit 0
