@echo off
REM ============================================================================
REM  Vendryx - local dev launcher (Windows)
REM
REM  Thin wrapper. All the real work is in scripts/dev-tenants.sh, which is the
REM  same script CI runs (.github/workflows/ci.yml). This file only supplies the
REM  three things that are specific to one Windows machine, and starts the four
REM  frontends, which that script does not cover.
REM
REM    1. Port numbers. Windows reserves TCP 5429-5528 (contains Postgres 5432)
REM       and TCP 5141-5240 (contains vite's 5173/5174/5175). Both fail with
REM       EACCES / "access to the socket is forbidden".
REM    2. Git Bash, for the script itself.
REM    3. The frontends.
REM
REM  Needs: Docker Desktop, Rust 1.80+, Node 20-24, Git for Windows.
REM  Node 25+ breaks the storefront SSR (see NODE_OPTIONS below).
REM ============================================================================

setlocal
cd /d "%~dp0"

REM Set before the stop check - "stop" needs the port numbers too.
REM    1. Port numbers. Windows reserves TCP 5429-5528 (contains Postgres 5432)
REM       and TCP 5141-5240 (contains vite's 5173/5174/5175). Both fail with
REM       EACCES / "access to the socket is forbidden".
REM
REM       The admin port is 6100, not 6000. Chromium refuses to navigate to 6000
REM       at all: it is on the browser's restricted-port list (X11 territory), so
REM       the dashboard died with ERR_UNSAFE_PORT and the server never received
REM       the request. Verified with Playwright against every candidate. 6100 is
REM       not restricted and is outside both Windows-reserved ranges.
set "ADMIN_PORT=6100"
set "PLAT_PORT=6001"
set "MKT_PORT=6002"
set "STORE_PORT=8000"

REM Node 25 exposes a global localStorage that throws on access without
REM --localstorage-file, so every SSR render 500s on
REM "localStorage.getItem is not a function". CI pins Node 20. Unset this once
REM you are back on a supported Node.
set "NODE_OPTIONS=--no-experimental-webstorage"

set "BASH=C:\Program Files\Git\bin\bash.exe"
if not exist "%BASH%" (
  echo   MISSING: Git Bash at %BASH%
  echo   It runs scripts/dev-tenants.sh, which CI also uses. Install Git for Windows.
  goto :fail
)

if /i "%~1"=="stop" goto :stop

echo.
echo === Prerequisites ===
where /q docker || (echo   MISSING: docker ^(Docker Desktop^) & goto :fail)
where /q cargo  || (echo   MISSING: cargo ^(Rust 1.80+^)   & goto :fail)
where /q node   || (echo   MISSING: node  ^(Node 20-24^)  & goto :fail)
docker info >nul 2>&1 || (echo   MISSING: docker daemon is not running. Start Docker Desktop. & goto :fail)
echo   docker, cargo, node found.

echo.
echo === Postgres + Redis ===
REM The 5900 publish comes from docker-compose.override.yml, not from here.
docker compose up -d postgres redis
if errorlevel 1 (
  echo   FAILED: docker compose could not start the containers.
  goto :fail
)

echo.
echo === Building engine ^(first run is slow^) ===
REM The script starts an already-built binary, so build before calling it.
call :build
if errorlevel 1 goto :fail

REM Feed dev-tenants.sh the two things it cannot work out on its own here.
REM   STHENO_BIN - it resolves its own path from cargo metadata. That now agrees
REM                with this build (.cargo/config.toml no longer pins an absolute
REM                path), but the binary is looked up once up front rather than
REM                rediscovered per call.
REM   PGPORT     - it defaults to 5432, which this Windows reserves.
REM   STHENO_CREDENTIAL_KEY - without it gateway credential writes fail silently.
set "STHENO_BIN=%TARGET_DIR%/debug/stheno-server.exe"
set "PGPORT=5900"
set "PGUSER=stheno"
set "PGPASSWORD=stheno"
if not defined STHENO_CREDENTIAL_KEY set "STHENO_CREDENTIAL_KEY=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"

echo.
echo === Engine, migrations and tenants ^(scripts/dev-tenants.sh^) ===
REM That script owns: --migrate, per-tenant create + seed, owner registration,
REM the 127.0.0.1 domain alias, start, wait for health, stop.
"%BASH%" scripts/dev-tenants.sh up
if errorlevel 1 goto :fail

echo.
echo === Frontend env files ===
if not exist "apps\storefront\.env.local" copy /y "apps\storefront\.env.template" "apps\storefront\.env.local" >nul
if not exist "apps\admin\.env"            copy /y "apps\admin\.env.example"      "apps\admin\.env" >nul
if not exist "apps\marketing\.env"        copy /y "apps\marketing\.env.example"  "apps\marketing\.env" >nul

REM Point the storefront's server-side fetches at 127.0.0.1. Browsers resolve
REM *.localhost internally but Node does not, and the engine resolves its tenant
REM from the Host header, so only the loopback address is a known merchant.
powershell -NoProfile -Command "(Get-Content 'apps\storefront\.env.local') -replace '^STHENO_BACKEND_URL=.*$','STHENO_BACKEND_URL=http://127.0.0.1:9000' | Set-Content 'apps\storefront\.env.local'"

REM The placeholder pk_test_... in .env.example is truthy, so it would override
REM the working dev key that clerk-auth-provider.tsx falls back to.
powershell -NoProfile -Command "(Get-Content 'apps\admin\.env') -replace '^VITE_CLERK_PUBLISHABLE_KEY=.*$','# left unset: the pk_test_... placeholder overrides the built-in dev key' | Set-Content 'apps\admin\.env'"

echo.
echo === npm install ^(first run only^) ===
call :deps storefront
call :deps admin
call :deps marketing
call :deps platform

echo.
echo === Starting frontends ===
start "Vendryx Storefront :%STORE_PORT%" /D "%CD%\apps\storefront" cmd /k npm run dev
start "Vendryx Admin :%ADMIN_PORT%"      /D "%CD%\apps\admin"      cmd /k npm run dev -- --port %ADMIN_PORT% --strictPort
start "Vendryx Marketing :%MKT_PORT%"    /D "%CD%\apps\marketing"  cmd /k npm run dev -- --port %MKT_PORT% --strictPort
start "Vendryx Platform :%PLAT_PORT%"    /D "%CD%\apps\platform"   cmd /k npm run dev -- --port %PLAT_PORT% --strictPort

REM `start` returns as soon as the window is created, not when the app is ready,
REM so the banner below used to print "Up" unconditionally -- including when a
REM frontend died on boot and bound nothing. Poll each port until it accepts a
REM connection or the timeout expires, then name the ones that never came up.
REM First run pays a full Vite cold build, hence the generous budget.
REM
REM Deliberately one PowerShell process for all four ports rather than a batch
REM loop: `setlocal` here has no EnableDelayedExpansion, so `!errorlevel!` would
REM not expand, and adding it would change how every other %VAR% in this file
REM resolves. A single script block sidesteps that entirely.
powershell -NoProfile -Command ^
  "$ports = @(%STORE_PORT%,%ADMIN_PORT%,%MKT_PORT%,%PLAT_PORT%);" ^
  "  $deadline = (Get-Date).AddSeconds(180);" ^
  "  $up = @{};" ^
  "  foreach ($p in $ports) {" ^
  "    while ((Get-Date) -lt $deadline -and -not $up[$p]) {" ^
  "      if (Test-NetConnection -ComputerName 127.0.0.1 -Port $p -InformationLevel Quiet -WarningAction SilentlyContinue) { $up[$p] = $true }" ^
  "      elseif ((Get-Date) -lt $deadline) { Start-Sleep -Seconds 2 }" ^
  "    }" ^
  "  };" ^
  "  $dead = $ports | Where-Object { -not $up[$_] };" ^
  "  if ($dead) { Write-Host ''; Write-Host ('   WARNING: nothing is listening on: ' + ($dead -join ', ')); Write-Host '   Check that window''s output. A Vite app fails here when its' ; Write-Host '   port is taken or when NODE_OPTIONS carries a flag its toolchain rejects.' }"

echo.
echo ============================================================
echo  Up.
echo.
echo   Storefront    http://localhost:%STORE_PORT%
echo   Admin         http://127.0.0.1:%ADMIN_PORT%
echo   Platform      http://127.0.0.1:%PLAT_PORT%
echo   Marketing     http://127.0.0.1:%MKT_PORT%
echo.
echo   NOTE: the admin port is 6100 deliberately. Chromium blocks 6000 as an
echo   unsafe port (ERR_UNSAFE_PORT), so the dashboard will not load there.
echo.
echo   Engine        http://127.0.0.1:9000/health
echo   Merchants are told apart by the Host header:
echo     pro    http://pro.localhost:9000    owner@pro.localhost
echo     free   http://free.localhost:9000   owner@free.localhost
echo     trial  http://trial.localhost:9000  owner@trial.localhost
echo   password for all three: Sup3rSecret!
echo.
echo   Shut down with:  run-dev.bat stop
echo ============================================================
echo.
pause
exit /b 0


REM call :build
:build
REM Ask cargo where it actually builds. .cargo/config.toml no longer pins an
REM absolute path, so this is normally the repo-local target/ - but respecting it
REM means this launcher keeps working if someone sets CARGO_TARGET_DIR, and it
REM fails loudly rather than silently building a second copy somewhere.
set "TARGET_DIR="
for /f "delims=" %%i in ('powershell -NoProfile -Command "((ConvertFrom-Json -InputObject (cargo metadata --format-version 1 --no-deps)).target_directory)"') do set "TARGET_DIR=%%i"
if not defined TARGET_DIR set "TARGET_DIR=%CD%\target"
set "TARGET_DIR=%TARGET_DIR:\=/%"
mkdir "%TARGET_DIR%" 2>nul
if not errorlevel 1 goto :build_ok
set "TARGET_DIR=%CD%\target"
echo   configured target-dir was not writable - using %TARGET_DIR%
mkdir "%TARGET_DIR%" 2>nul
:build_ok
set "CARGO_TARGET_DIR=%TARGET_DIR%"
cargo build -p stheno-server
if errorlevel 1 (
  echo   FAILED: cargo build
  exit /b 1
)
exit /b 0


REM call :deps <app>
:deps
if exist "apps\%~1\node_modules" exit /b 0
echo   installing apps\%~1 ...
pushd "apps\%~1"
call npm install --no-fund --no-audit
popd
exit /b 0


:stop
echo.
echo === Stopping frontends ===
for %%P in (%ADMIN_PORT% %PLAT_PORT% %MKT_PORT% %STORE_PORT%) do for /f "tokens=5" %%i in ('netstat -ano ^| findstr /c:":%%P " ^| findstr /c:"LISTENING"') do taskkill /f /pid %%i >nul 2>&1
echo === Stopping engine and tenants ^(scripts/dev-tenants.sh^) ===
"%BASH%" scripts/dev-tenants.sh down
echo === Docker ===
docker compose stop
echo.
echo Done. Containers stopped, data kept. Wipe the database with: docker compose down -v
echo.
pause
exit /b 0


:fail
echo.
echo === FAILED - see the message above ===
echo.
pause
exit /b 1