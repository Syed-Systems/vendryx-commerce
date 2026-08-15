Write-Host "==> Checking schema drift..." -ForegroundColor Cyan

$status = git status --porcelain
if ($status -match "schema\.rs") {
    Write-Host "ERROR: Uncommitted schema.rs changes detected." -ForegroundColor Red
    git diff -- "crates/modules/*/src/schema.rs"
    exit 1
}

Write-Host "Schema drift check PASSED: No schema drift detected." -ForegroundColor Green
exit 0
