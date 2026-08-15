Write-Host "==> Checking module isolation..." -ForegroundColor Cyan

$modules = Get-ChildItem -Path "crates/modules" -Directory | Select-Object -ExpandProperty Name
$failed = $false

foreach ($mod in $modules) {
    $cargoFile = "crates/modules/$mod/Cargo.toml"
    if (Test-Path $cargoFile) {
        $content = Get-Content $cargoFile
        foreach ($other in $modules) {
            if ($mod -ne $other) {
                if ($content -match "^\s*$other\s*=") {
                    Write-Host "ERROR: Module isolation violation! $mod depends on $other in $cargoFile" -ForegroundColor Red
                    $failed = $true
                }
            }
        }
    }
}

if ($failed) {
    Write-Host "Module isolation check FAILED." -ForegroundColor Red
    exit 1
}

Write-Host "Module isolation check PASSED: No cross-module dependencies detected." -ForegroundColor Green
exit 0
