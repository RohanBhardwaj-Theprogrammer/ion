@echo off
REM INSTANT CONTAINER INIT + FULL TOOLCHAIN
REM
REM Single target: "runtime" - Full dev environment ready instantly
REM
REM Strategy: Build everything ONCE in image → container starts instantly

setlocal enabledelayedexpansion

set DOCKER_BUILDKIT=1
set DOCKER_BUILDKIT_INLINE_CACHE=1

@REM echo  downloading the images to warm the cache...
@REM docker pull docker/dockerfile:1.7
@REM docker pull rust:1-bookworm@sha256:9676d0547a259997add8f5924eb6b959c589ed39055338e23b99aba7958d6d31

echo.
echo ============================================================
echo  INSTANT DEV ENVIRONMENT
echo ============================================================
echo.
echo Build workflow:
echo   1. Toolchain (mingw, rust, cargo) - cached forever
echo   2. Dependencies pre-compiled - cached until Cargo.toml
echo   3. Project copied + compiled
echo   4. Binary in PATH
echo.
echo Result: INSTANT container initialization
echo.
echo ============================================================
echo.

@REM echo Warming Docker caches (frontend + base image)...
@REM docker pull docker/dockerfile:1 >nul 2>&1
@REM docker pull rust:1-bookworm >nul 2>&1
@REM echo Caches warmed.
@REM echo.

set START_TIME=%time%

REM Build single "runtime" target - everything ready
docker build ^
    --target runtime ^
    -t ion:dev ^
    -f Dockerfile.win-gnu ^
    .

if errorlevel 1 (
    echo.
    echo ERROR: Docker build failed
    exit /b 1
)

set END_TIME=%time%

echo.
echo ============================================================
echo Build completed!
echo ============================================================
echo.

echo Image ready:
echo.
echo   ion:dev    - Full dev environment
echo              - Tools: mingw, rust, cargo, ion
echo              - Project: compiled + binary in PATH
echo              - Init: INSTANT
echo.

echo Usage:
echo.
echo   # Run container (instant startup)
echo   docker run -it ion:dev
echo.
echo   # Run ion commands
echo   docker run ion:dev ion build
echo   docker run ion:dev ion --help
echo.
echo   # Interactive shell
echo   docker run -it ion:dev /bin/bash
echo.

REM Compute elapsed time (milliseconds) between START_TIME and END_TIME
REM Robust parsing using substrings to avoid "Missing operator" warnings
set sthh=%START_TIME:~0,2%
set stmm=%START_TIME:~3,2%
set stss=%START_TIME:~6,2%
set stcs=%START_TIME:~9,2%
if "!sthh:~0,1!"==" " set sthh=0!sthh:~1,1!

set ehh=%END_TIME:~0,2%
set emm=%END_TIME:~3,2%
set ess=%END_TIME:~6,2%
set ecs=%END_TIME:~9,2%
if "!ehh:~0,1!"==" " set ehh=0!ehh:~1,1!

set /a start_cs=!sthh!*360000 + !stmm!*6000 + !stss!*100 + !stcs!
set /a end_cs=!ehh!*360000 + !emm!*6000 + !ess!*100 + !ecs!
set /a elapsed_cs=end_cs - start_cs
if !elapsed_cs! lss 0 set /a elapsed_cs+=24*360000
set /a elapsed_ms=elapsed_cs*10

echo ================== completed in ==========================================
echo                    from %START_TIME% to %END_TIME%
echo                               !elapsed_ms! ms
echo ==========================================================================




