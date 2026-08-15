Write-Host "==> Checking DTO / Entity separation..." -ForegroundColor Cyan

$modelFiles = Get-ChildItem -Path "crates/modules" -Recurse -Filter "models.rs" -ErrorAction SilentlyContinue
$failed = $false

foreach ($file in $modelFiles) {
    $matches = Select-String -Path $file.FullName -Pattern "derive.*Deserialize"
    if ($matches) {
        Write-Host "ERROR: Entity models must never derive Deserialize (AGENTS.md §2.6) in $($file.FullName)" -ForegroundColor Red
        $failed = $true
    }
}

if ($failed) {
    Write-Host "DTO separation check FAILED." -ForegroundColor Red
    exit 1
}

Write-Host "DTO separation check PASSED: No entity models derive Deserialize." -ForegroundColor Green
exit 0
