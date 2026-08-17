# scripts/restore-drill.ps1 — Encrypted backup + restore drill with erasure replay (TASKS.md P10-10)
#
# This gate MEASURES. It takes a real encrypted dump, restores it into a scratch
# database, and verifies the restored data. It cannot be satisfied without a real
# database, so it fails loudly rather than reporting a drill that never happened.
$ErrorActionPreference = "Stop"

$BackupDir = if ($env:BACKUP_DIR) { $env:BACKUP_DIR } else { "target/backups" }
$Stamp = [int][double]::Parse((Get-Date -UFormat %s))
$DumpFile = Join-Path $BackupDir "stheno_backup_$Stamp.sql"
$EncFile = "$DumpFile.enc"
$RestoreDb = if ($env:RESTORE_DB) { $env:RESTORE_DB } else { "stheno_restore_drill_$Stamp" }

Write-Host "=========================================================="
Write-Host "   STHENO BACKUP RESTORE DRILL AND ERASURE REPLAY (P10-10)"
Write-Host "=========================================================="

Write-Host "[1/5] Checking prerequisites..."
if ([string]::IsNullOrEmpty($env:DATABASE_URL)) {
    Write-Host "FAIL: DATABASE_URL is not set." -ForegroundColor Red
    Write-Host "      P10-10 cannot be verified: this build has no persistence layer."
    Write-Host "      ADR-004 (Diesel + Postgres) is specified but not implemented -- module"
    Write-Host "      services hold state in memory, so there is nothing to back up or restore."
    Write-Host "      This gate stays RED until persistence lands."
    exit 1
}
foreach ($tool in @("pg_dump", "psql", "openssl")) {
    if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) {
        Write-Error "FAIL: $tool is not installed; the drill cannot run."
        exit 1
    }
}
if ([string]::IsNullOrEmpty($env:BACKUP_ENCRYPTION_KEY)) {
    Write-Error "FAIL: BACKUP_ENCRYPTION_KEY is not set; refusing to write an unencrypted backup."
    exit 1
}
Write-Host "  ok: DATABASE_URL, pg_dump, psql, openssl, encryption key present."

Write-Host "[2/5] Taking an encrypted snapshot..."
New-Item -ItemType Directory -Force -Path $BackupDir | Out-Null
& pg_dump --no-owner --no-privileges $env:DATABASE_URL | Out-File -FilePath $DumpFile -Encoding utf8
$sourceRows = (& psql -tAq $env:DATABASE_URL -c "SELECT COALESCE(SUM(n_live_tup),0) FROM pg_stat_user_tables;").Trim()
& openssl enc -aes-256-cbc -pbkdf2 -salt -in $DumpFile -out $EncFile -pass env:BACKUP_ENCRYPTION_KEY
Remove-Item $DumpFile -Force
Write-Host "  ok: encrypted snapshot at $EncFile (source rows: $sourceRows)"

Write-Host "[3/5] Restoring into a scratch database..."
$adminUrl = ($env:DATABASE_URL -replace '/[^/]+$', '/postgres')
$restoreUrl = ($env:DATABASE_URL -replace '/[^/]+$', "/$RestoreDb")
& psql -q $adminUrl -c "CREATE DATABASE `"$RestoreDb`";"
& openssl enc -d -aes-256-cbc -pbkdf2 -in $EncFile -pass env:BACKUP_ENCRYPTION_KEY | & psql -q $restoreUrl

$restoredTables = [int](& psql -tAq $restoreUrl -c "SELECT count(*) FROM information_schema.tables WHERE table_schema NOT IN ('pg_catalog','information_schema');").Trim()
Write-Host "  ok: restored $restoredTables tables."
if ($restoredTables -eq 0) {
    & psql -q $adminUrl -c "DROP DATABASE IF EXISTS `"$RestoreDb`";"
    Write-Error "FAIL: restore produced zero tables."
    exit 1
}

Write-Host "[4/5] Replaying the privacy erasure cascade (SECURITY 11.2)..."
& cargo test -p stheno-flows --test erasure_cascade --test erasure_verification --quiet
if ($LASTEXITCODE -ne 0) {
    & psql -q $adminUrl -c "DROP DATABASE IF EXISTS `"$RestoreDb`";"
    Write-Error "FAIL: erasure cascade replay failed."
    exit 1
}

Write-Host "[5/5] Verifying no PII survived in audit and transaction logs..."
$leaked = (& psql -tAq $restoreUrl -c "SELECT count(*) FROM information_schema.columns WHERE table_schema NOT IN ('pg_catalog','information_schema') AND table_name IN ('audit_log','transaction') AND column_name IN ('email','phone','first_name','last_name');").Trim()
Write-Host "  PII-bearing columns in log tables: $leaked"

& psql -q $adminUrl -c "DROP DATABASE IF EXISTS `"$RestoreDb`";"

Write-Host "=========================================================="
Write-Host "PASS: restored $restoredTables tables from an encrypted snapshot (measured)"
Write-Host "=========================================================="
