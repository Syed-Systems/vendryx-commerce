Write-Host "==> Checking SQL boundaries..." -ForegroundColor Cyan

$modules = Get-ChildItem -Path "crates/modules" -Directory | Select-Object -ExpandProperty Name
$failed = $false

foreach ($mod in $modules) {
    $srcPath = "crates/modules/$mod/src"
    if (Test-Path $srcPath) {
        foreach ($other in $modules) {
            if ($mod -ne $other) {
                $otherSchema = ($other -replace '^stheno-', '') -replace '-', '_'
                $matches = Select-String -Path "$srcPath\*.rs" -Pattern "sql_query.*$otherSchema\." -ErrorAction SilentlyContinue
                if ($matches) {
                    Write-Host "ERROR: Cross-schema SQL violation in $mod referencing $otherSchema" -ForegroundColor Red
                    $failed = $true
                }
            }
        }
    }
}

if ($failed) {
    Write-Host "SQL boundary check FAILED." -ForegroundColor Red
    exit 1
}

Write-Host "SQL boundary check PASSED: No cross-schema SQL detected." -ForegroundColor Green
exit 0
