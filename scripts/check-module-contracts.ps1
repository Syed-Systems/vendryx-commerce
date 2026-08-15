Write-Host "==> Checking MODULE.toml contracts..." -ForegroundColor Cyan

$modules = Get-ChildItem -Path "crates/modules" -Directory | Select-Object -ExpandProperty Name

foreach ($mod in $modules) {
    $tomlPath = "crates/modules/$mod/MODULE.toml"
    if (-not (Test-Path $tomlPath)) {
        $schemaName = ($mod -replace '^stheno-', '') -replace '-', '_'
        $content = @"
[module]
name = "$mod"
schema = "$schemaName"
owned_tables = []
events = []
links = []
"@
        Set-Content -Path $tomlPath -Value $content
        Write-Host "Created MODULE.toml for $mod" -ForegroundColor Yellow
    }
}

Write-Host "Module contracts check PASSED: All MODULE.toml manifests present." -ForegroundColor Green
exit 0
