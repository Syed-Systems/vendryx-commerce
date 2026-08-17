# scripts/soak.ps1 — Sustained load soak and memory budget check (TASKS.md P10-14)
#
# This gate MEASURES. It drives real traffic at the configured rate against a running
# stheno-server and samples that process's real RSS. It reports what it observed.
param (
    [string]$duration = "1h",
    [int]$rps = 500
)

$ErrorActionPreference = "Stop"

$TargetUrl = if ($env:TARGET_URL) { $env:TARGET_URL } else { "http://127.0.0.1:9000" }
$RssBudgetMb = 400

$durationSec = switch -Regex ($duration) {
    '^(\d+)h$' { [int]$Matches[1] * 3600; break }
    '^(\d+)m$' { [int]$Matches[1] * 60;   break }
    '^(\d+)s$' { [int]$Matches[1];        break }
    default    { [int]$duration }
}

Write-Host "=========================================================="
Write-Host "    STHENO SOAK AND SUSTAINED LOAD TEST (P10-14)          "
Write-Host "=========================================================="
Write-Host "Target = $TargetUrl | Rate = $rps rps | Duration = $duration ($durationSec s)"

Write-Host "[1/4] Checking the target is up..."
try {
    Invoke-WebRequest -Uri "$TargetUrl/health" -TimeoutSec 5 -UseBasicParsing *>$null
} catch {
    Write-Error "FAIL: no server answering at $TargetUrl/health. Start it with: cargo run -p stheno-server -- --bind 127.0.0.1:9000"
    exit 1
}
Write-Host "  ok: target healthy."

Write-Host "[2/4] Locating the server process to sample RSS from..."
$proc = Get-Process stheno-server -ErrorAction SilentlyContinue | Select-Object -First 1
if ($null -eq $proc) {
    Write-Error "FAIL: could not find the stheno-server process, so RSS cannot be measured. P10-14 is UNVERIFIED."
    exit 1
}
Write-Host "  ok: sampling pid $($proc.Id)."

Write-Host "[3/4] Driving $rps rps for $durationSec s..."
$resultFile = Join-Path $env:TEMP "stheno-soak-result.json"
if (Get-Command k6 -ErrorAction SilentlyContinue) {
    $job = Start-Job -ScriptBlock {
        param($d) & k6 run --quiet --vus 50 --duration "${d}s" tests/load/all.js
        $LASTEXITCODE
    } -ArgumentList $durationSec
} else {
    Write-Host "  note: k6 not installed, using scripts/load-driver.js"
    $job = Start-Job -ScriptBlock {
        param($url, $r, $d, $out)
        & node scripts/load-driver.js --url $url --rps $r --duration $d | Out-File -FilePath $out -Encoding utf8
        $LASTEXITCODE
    } -ArgumentList $TargetUrl, $rps, $durationSec, $resultFile
}

$peakRssMb = 0
while ($job.State -eq 'Running') {
    $current = Get-Process -Id $proc.Id -ErrorAction SilentlyContinue
    if ($current) {
        $rss = [math]::Round($current.WorkingSet64 / 1MB)
        if ($rss -gt $peakRssMb) { $peakRssMb = $rss }
    }
    Start-Sleep -Seconds 5
}

$loadStatus = Receive-Job $job
Remove-Job $job

if (Test-Path $resultFile) {
    Write-Host "  driver result: $(Get-Content $resultFile -Raw)"
}
if ($loadStatus -ne 0) {
    Write-Error "FAIL: the load run reported errors (exit $loadStatus)."
    exit 1
}
Write-Host "  ok: load completed with no request errors."

Write-Host "[4/4] Auditing peak RSS (budget <= $RssBudgetMb MB)..."
Write-Host "  measured peak RSS: $peakRssMb MB"
if ($peakRssMb -le 0) {
    Write-Error "FAIL: no RSS samples were collected; the budget is unverified."
    exit 1
}
if ($peakRssMb -gt $RssBudgetMb) {
    Write-Error "FAIL: peak RSS $peakRssMb MB exceeds the $RssBudgetMb MB budget."
    exit 1
}

Write-Host "=========================================================="
Write-Host "PASS: peak RSS $peakRssMb MB over $durationSec s at $rps rps (measured)"
Write-Host "=========================================================="
