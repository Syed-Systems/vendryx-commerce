# P11-21, P14-22: a module declared migrated may not hold in-memory domain state.

param(
    [switch]$All
)

$ErrorActionPreference = "Stop"

# All modules migrated in P11, P12, and P14
$Migrated = @(
    "stheno-region",
    "stheno-currency",
    "stheno-product",
    "stheno-pricing",
    "stheno-inventory",
    "stheno-stock-location",
    "stheno-sales-channel",
    "stheno-tax",
    "stheno-store",
    "stheno-customer",
    "stheno-auth",
    "stheno-user",
    "stheno-api-key",
    "stheno-rbac",
    "stheno-file",
    "stheno-notification",
    "stheno-search",
    "stheno-index",
    "stheno-translation",
    "stheno-settings",
    "stheno-audit",
    "stheno-analytics"
)

if ($All) {
    $Migrated = Get-ChildItem -Directory "crates/modules" | Select-Object -ExpandProperty Name
}

Write-Host "==> Checking for in-memory stores in migrated modules..."

$Failed = $false
$Checked = 0
$Pattern = 'RwLock<\s*(HashMap|Vec)|Mutex<\s*(HashMap|Vec)'

foreach ($module in $Migrated) {
    # Exclude non-domain / infrastructure crates from domain store scan
    if ($module -eq "stheno-caching" -or $module -eq "stheno-locking" -or $module -eq "stheno-demo") {
        continue
    }

    $src = "crates/modules/$module/src"
    if (-not (Test-Path $src)) {
        Write-Host "FAIL: $module is declared migrated but has no source at $src" -ForegroundColor Red
        $Failed = $true
        continue
    }

    $Checked++
    $files = Get-ChildItem -Path $src -Filter "*.rs" -Recurse
    foreach ($file in $files) {
        $lines = Get-Content $file.FullName
        for ($i = 0; $i -lt $lines.Count; $i++) {
            $line = $lines[$i]
            if ($line -match '^\s*(//|/\*|\*)') {
                continue
            }
            if ($line -match $Pattern) {
                Write-Host "FAIL: $module ($($file.Name):$($i+1)) still holds in-memory state: $line" -ForegroundColor Red
                $Failed = $true
            }
        }
    }
}

if ($Checked -eq 0) {
    Write-Host "FAIL: no modules were checked." -ForegroundColor Red
    exit 1
}

if ($Failed) {
    Write-Host "In-memory store check FAILED." -ForegroundColor Red
    exit 1
}

Write-Host "In-memory store check PASSED: $Checked migrated module(s) hold no in-process state." -ForegroundColor Green
exit 0
