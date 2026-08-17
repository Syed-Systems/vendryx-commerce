# scripts/check-image.ps1 — Distroless image and cold-boot budget checks (TASKS.md P10-11)
#
# This gate MEASURES. It builds the image, reads its real size, boots a container and
# times it to first healthy response. It does not assert numbers it did not observe.
$ErrorActionPreference = "Stop"

$ImageTag = if ($env:IMAGE_TAG) { $env:IMAGE_TAG } else { "stheno-server:gate-p10-11" }
$SizeBudgetMb = 60
$ColdStartBudgetMs = 1000
$ContainerName = "stheno-p10-11-$PID"

Write-Host "=========================================================="
Write-Host "      STHENO CONTAINER IMAGE AND SECURITY AUDIT (P10-11)  "
Write-Host "=========================================================="

try {
    Write-Host "[1/4] Checking Dockerfile presence and base image..."
    if (!(Test-Path "Dockerfile")) { Write-Error "FAIL: Dockerfile not found."; exit 1 }
    $content = Get-Content "Dockerfile" -Raw
    if ($content -notmatch "distroless/static") {
        Write-Error "FAIL: Dockerfile must use a distroless static base."; exit 1
    }
    Write-Host "  ok: distroless static base declared."

    Write-Host "[2/4] Verifying non-root execution policy..."
    if ($content -notmatch "USER 65532:65532") {
        Write-Error "FAIL: Dockerfile must run as non-root user 65532:65532."; exit 1
    }
    Write-Host "  ok: non-root user declared."

    if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
        Write-Error "FAIL: docker is not installed, so size and cold start cannot be measured. P10-11 is UNVERIFIED."
        exit 1
    }
    docker info *>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Error "FAIL: the Docker daemon is not reachable. P10-11 is UNVERIFIED."
        exit 1
    }

    Write-Host "[3/4] Building image and measuring size (budget <= $SizeBudgetMb MB)..."
    docker build -t $ImageTag . *>$null
    if ($LASTEXITCODE -ne 0) { Write-Error "FAIL: docker build failed."; exit 1 }

    $sizeBytes = [int64](docker image inspect $ImageTag --format '{{.Size}}')
    $sizeMb = [math]::Floor($sizeBytes / 1000000)
    Write-Host "  measured: $sizeMb MB"
    if ($sizeMb -gt $SizeBudgetMb) {
        Write-Error "FAIL: image is $sizeMb MB, over the $SizeBudgetMb MB budget."; exit 1
    }
    Write-Host "  ok: within budget."

    Write-Host "[4/4] Measuring cold start (budget <= $ColdStartBudgetMs ms)..."
    $hostPort = Get-Random -Minimum 19000 -Maximum 20000
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    docker run -d --name $ContainerName -p "${hostPort}:9000" $ImageTag --bind 0.0.0.0:9000 *>$null
    if ($LASTEXITCODE -ne 0) { Write-Error "FAIL: docker run failed."; exit 1 }

    $ready = $false
    for ($i = 0; $i -lt 600; $i++) {
        try {
            Invoke-WebRequest -Uri "http://127.0.0.1:$hostPort/health" -TimeoutSec 1 -UseBasicParsing *>$null
            $ready = $true
            break
        } catch {
            $running = docker ps -q --filter "name=$ContainerName"
            if (-not $running) {
                docker logs $ContainerName
                Write-Error "FAIL: container exited before becoming healthy."; exit 1
            }
            Start-Sleep -Milliseconds 50
        }
    }
    $sw.Stop()

    if (-not $ready) {
        docker logs $ContainerName
        Write-Error "FAIL: container never answered /health."; exit 1
    }

    $coldStartMs = [math]::Round($sw.Elapsed.TotalMilliseconds)
    Write-Host "  measured: $coldStartMs ms"
    if ($coldStartMs -gt $ColdStartBudgetMs) {
        Write-Error "FAIL: cold start $coldStartMs ms exceeds the $ColdStartBudgetMs ms budget."; exit 1
    }
    Write-Host "  ok: within budget."

    Write-Host "=========================================================="
    Write-Host "PASS: image $sizeMb MB, cold start $coldStartMs ms (measured)"
    Write-Host "=========================================================="
}
finally {
    docker rm -f $ContainerName *>$null
}
