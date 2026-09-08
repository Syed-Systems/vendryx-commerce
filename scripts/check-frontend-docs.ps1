# scripts/check-frontend-docs.ps1 — P27-10 Theme Authoring Documentation Gate
$ErrorActionPreference = "Stop"
Set-Location "$PSScriptRoot\.."

$Doc = "docs-site/docs/frontend-authoring.md"

if (-not (Test-Path $Doc)) {
    Write-Error "FAIL: Theme authoring guide $Doc does not exist."
    exit 1
}

Write-Host "Verifying theme authoring guide in $Doc..."
$content = Get-Content $Doc -Raw
$firstLines = (Get-Content $Doc -TotalCount 10) -join "`n"

if ($firstLines -notmatch "Checkout is core at every tier") {
    Write-Error "FAIL: Rule 0 ('Checkout is core at every tier') must be stated first in $Doc."
    exit 1
}

if ($content -notmatch "FrontendContract" -or $content -notmatch "45") {
    Write-Error "FAIL: $Doc must reference the 45-slot FrontendContract."
    exit 1
}

if ($content -notmatch "ADAPT" -or $content -notmatch "SKIN") {
    Write-Error "FAIL: $Doc must document ADAPT vs SKIN per AGENTS.md §12."
    exit 1
}

if ($content -notmatch "time-to-new-theme" -and $content -notmatch "Benchmark") {
    Write-Error "FAIL: $Doc must record time-to-new-theme benchmark measurements."
    exit 1
}

Write-Host "PASS: Theme authoring documentation verified (checkout rule first, contract slots, benchmarks)."
exit 0
