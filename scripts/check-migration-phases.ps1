Write-Host "==> Checking migration phases (AGENTS.md §2.13)..." -ForegroundColor Cyan

$migrationFiles = Get-ChildItem -Path "crates/modules" -Recurse -Filter "*.sql" -ErrorAction SilentlyContinue
$failed = $false

foreach ($file in $migrationFiles) {
    $content = Get-Content $file.FullName -Raw
    if ($content -match "(?i)(DROP\s+COLUMN|ALTER\s+COLUMN.*TYPE)") {
        if ($content -notmatch "--\s*phase:\s*contract") {
            Write-Host "ERROR: Destructive statement in $($file.FullName) without '-- phase: contract' marker" -ForegroundColor Red
            $failed = $true
        }
    }
}

if ($failed) {
    Write-Host "Migration phases check FAILED." -ForegroundColor Red
    exit 1
}

Write-Host "Migration phases check PASSED: All destructive statements are phased." -ForegroundColor Green
exit 0
