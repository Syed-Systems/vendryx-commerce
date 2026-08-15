param (
    [string]$group = "all"
)

$env:PATH = "C:\Users\Mahi\mingw64\bin;$env:USERPROFILE\.cargo\bin;" + $env:PATH

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "==> Running Medusa v2 Parity Suite [Group: $group]..." -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan

switch ($group) {
    "store-catalog" {
        Write-Host "Running store catalog parity tests..." -ForegroundColor Yellow
        cargo test -p stheno-api-store --test catalog_parity
        if ($LASTEXITCODE -ne 0) { exit 1 }
    }
    "store-pricing" {
        Write-Host "Running store pricing parity tests..." -ForegroundColor Yellow
        cargo test -p stheno-api-store --test pricing_parity
        if ($LASTEXITCODE -ne 0) { exit 1 }
    }
    "store-cart" {
        Write-Host "Running store cart parity tests..." -ForegroundColor Yellow
        cargo test -p stheno-api-store --test cart_parity
        if ($LASTEXITCODE -ne 0) { exit 1 }
    }
    "store-order" {
        Write-Host "Running store order parity tests..." -ForegroundColor Yellow
        cargo test -p stheno-api-store --test order_parity
        if ($LASTEXITCODE -ne 0) { exit 1 }
    }
    "store-returns" {
        Write-Host "Running store returns parity tests..." -ForegroundColor Yellow
        cargo test -p stheno-api-store --test return_parity
        if ($LASTEXITCODE -ne 0) { exit 1 }
    }
    "admin-catalog" {
        Write-Host "Running admin catalog parity tests..." -ForegroundColor Yellow
        cargo test -p stheno-api-admin --test admin_parity test_admin_catalog
        if ($LASTEXITCODE -ne 0) { exit 1 }
    }
    "admin-inventory" {
        Write-Host "Running admin inventory parity tests..." -ForegroundColor Yellow
        cargo test -p stheno-api-admin --test admin_parity test_admin_inventory
        if ($LASTEXITCODE -ne 0) { exit 1 }
    }
    "admin-pricing" {
        Write-Host "Running admin pricing parity tests..." -ForegroundColor Yellow
        cargo test -p stheno-api-admin --test admin_parity test_admin_pricing
        if ($LASTEXITCODE -ne 0) { exit 1 }
    }
    "admin-orders" {
        Write-Host "Running admin orders parity tests..." -ForegroundColor Yellow
        cargo test -p stheno-api-admin --test admin_parity test_admin_orders
        if ($LASTEXITCODE -ne 0) { exit 1 }
    }
    "admin-order-mutations" {
        Write-Host "Running admin order mutations parity tests..." -ForegroundColor Yellow
        cargo test -p stheno-api-admin --test admin_parity test_admin_order_mutations
        if ($LASTEXITCODE -ne 0) { exit 1 }
    }
    "admin-draft-orders" {
        Write-Host "Running admin draft orders parity tests..." -ForegroundColor Yellow
        cargo test -p stheno-api-admin --test admin_parity test_admin_draft_orders
        if ($LASTEXITCODE -ne 0) { exit 1 }
    }
    "admin-fulfillment" {
        Write-Host "Running admin fulfillment parity tests..." -ForegroundColor Yellow
        cargo test -p stheno-api-admin --test admin_parity test_admin_fulfillment
        if ($LASTEXITCODE -ne 0) { exit 1 }
    }
    "admin-payments" {
        Write-Host "Running admin payments parity tests..." -ForegroundColor Yellow
        cargo test -p stheno-api-admin --test admin_parity test_admin_payments
        if ($LASTEXITCODE -ne 0) { exit 1 }
    }
    "admin-customers" {
        Write-Host "Running admin customers parity tests..." -ForegroundColor Yellow
        cargo test -p stheno-api-admin --test admin_parity test_admin_customers
        if ($LASTEXITCODE -ne 0) { exit 1 }
    }
    "admin-promotions" {
        Write-Host "Running admin promotions parity tests..." -ForegroundColor Yellow
        cargo test -p stheno-api-admin --test admin_parity test_admin_promotions
        if ($LASTEXITCODE -ne 0) { exit 1 }
    }
    "admin-marketing" {
        Write-Host "Running admin marketing parity tests..." -ForegroundColor Yellow
        cargo test -p stheno-api-admin --test admin_parity test_admin_marketing
        if ($LASTEXITCODE -ne 0) { exit 1 }
    }
    "admin-config" {
        Write-Host "Running admin config parity tests..." -ForegroundColor Yellow
        cargo test -p stheno-api-admin --test admin_parity test_admin_config
        if ($LASTEXITCODE -ne 0) { exit 1 }
    }
    "admin-access" {
        Write-Host "Running admin access parity tests..." -ForegroundColor Yellow
        cargo test -p stheno-api-admin --test admin_parity test_admin_access
        if ($LASTEXITCODE -ne 0) { exit 1 }
    }
    "admin-platform" {
        Write-Host "Running admin platform parity tests..." -ForegroundColor Yellow
        cargo test -p stheno-api-admin --test admin_parity test_admin_platform
        if ($LASTEXITCODE -ne 0) { exit 1 }
    }
    "admin-feature-flags" {
        Write-Host "Running admin feature flags parity tests..." -ForegroundColor Yellow
        cargo test -p stheno-api-admin --test admin_parity test_admin_feature_flags
        if ($LASTEXITCODE -ne 0) { exit 1 }
    }
    default {
        Write-Host "Running full store parity suite..." -ForegroundColor Yellow
        cargo test -p stheno-api-store
        if ($LASTEXITCODE -ne 0) { exit 1 }
    }
}

Write-Host "=========================================" -ForegroundColor Green
Write-Host "==> Parity Suite PASSED: 100% GREEN!" -ForegroundColor Green
Write-Host "=========================================" -ForegroundColor Green
exit 0
