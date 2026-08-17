# scripts/check-no-enterprise-paths.ps1 — Verify no enterprise paths remain in dashboard fork (TASKS.md P9-17)
$ErrorActionPreference = "Stop"

Write-Host "=========================================================="
Write-Host "    CHECK NO ENTERPRISE PATHS AUDIT (P9-17)               "
Write-Host "=========================================================="

Write-Host "[1/2] Scanning dashboard source code for proprietary enterprise paths..."
$forbiddenPatterns = @(
    "/admin/enterprise",
    "medusa-enterprise",
    "@medusajs/admin-enterprise"
)

if (Test-Path "apps/admin/src") {
    $files = Get-ChildItem -Path "apps/admin/src" -Recurse -File
    foreach ($file in $files) {
        $content = Get-Content $file.FullName -Raw
        foreach ($pattern in $forbiddenPatterns) {
            if ($content -match [regex]::Escape($pattern)) {
                Write-Error "Found forbidden enterprise pattern '$pattern' in $($file.FullName)"
                exit 1
            }
        }
    }
}
Write-Host "Zero enterprise paths detected in apps/admin/."

Write-Host "[2/2] Verifying clean-room implementations for RBAC and SSO..."
if (!(Test-Path "apps/admin/src/components/rbac.tsx") -or !(Test-Path "apps/admin/src/components/sso.tsx")) {
    Write-Error "Clean-room rbac.tsx or sso.tsx missing!"
    exit 1
}
Write-Host "Clean-room RBAC and SSO components verified."

Write-Host "=========================================================="
Write-Host "STATUS: NO ENTERPRISE PATHS AUDIT PASSED (Exit 0)"
Write-Host "=========================================================="
exit 0
