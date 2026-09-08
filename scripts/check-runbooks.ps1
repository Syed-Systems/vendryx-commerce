# scripts/check-runbooks.ps1
# P28-05: Operational incident runbooks verification gate (PowerShell).

$ErrorActionPreference = "Stop"
$RepoRoot = Resolve-Path "$PSScriptRoot\.."
Set-Location $RepoRoot

Write-Host "==> Verifying operational incident runbooks (P28-05)..." -ForegroundColor Cyan

$Runbooks = @(
    "docs/runbooks/reconciliation-discrepancy.md",
    "docs/runbooks/stuck-consignment.md",
    "docs/runbooks/gateway-outage.md"
)

$RequiredSections = @(
    "Incident Definition",
    "Detection",
    "Investigation",
    "Mitigation",
    "Post-Incident"
)

foreach ($runbook in $Runbooks) {
    if (-not (Test-Path $runbook)) {
        Write-Error "Missing required runbook: $runbook"
        exit 1
    }
    Write-Host "✓ Found $runbook" -ForegroundColor Green

    $content = Get-Content $runbook -Raw
    foreach ($section in $RequiredSections) {
        if ($content -notmatch [regex]::Escape($section)) {
            Write-Error "Runbook $runbook missing required section: $section"
            exit 1
        }
    }
    Write-Host "  ✓ All required sections present in $runbook" -ForegroundColor DarkGreen
}

Write-Host "==> All operational runbooks verified successfully (P28-05 PASSED)." -ForegroundColor Green
exit 0
