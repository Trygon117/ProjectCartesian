@echo off
echo ==========================================
echo   Project Cartesian: Split-Brain Build
echo ==========================================

REM Check Docker
docker info >nul 2>&1
IF %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Docker is not running!
    pause
    exit /b
)

REM Ensure directories on Host
if not exist "logs\build" mkdir "logs\build"
if not exist "iso\out" mkdir "iso\out"
if not exist "pkg\libs" mkdir "pkg\libs"
if not exist "pkg\stage1_artifacts" mkdir "pkg\stage1_artifacts"

echo.
echo [1/3] Preparing Environments...

REM 1. Build the Compiler Image (Ubuntu + CUDA)
docker build -t cartesian-compiler -f iso/Dockerfile.compiler .
IF %ERRORLEVEL% NEQ 0 goto :error

REM 2. Build the Packager Image (Arch + mkarchiso)
docker build -t cartesian-packager -f iso/Dockerfile.packager .
IF %ERRORLEVEL% NEQ 0 goto :error

echo.
echo [2/3] Stage 1: Compiling Core (Ubuntu/CUDA)...
echo ---------------------------------------------
REM Runs the build, outputs binary to ./pkg/stage1_artifacts
REM We mount the target cache so we don't recompile from scratch every time.
docker run --rm ^
    -v "%cd%":/project_cartesian ^
    -v cartesian_registry_cache:/root/.cargo/registry ^
    -v cartesian_target_cache:/project_cartesian/src/cartesian-core/target ^
    cartesian-compiler
IF %ERRORLEVEL% NEQ 0 goto :error

echo.
echo [3/3] Stage 2: Packaging ISO (Arch Linux)...
echo ---------------------------------------------
REM Takes the artifacts from Stage 1 and builds the ISO
docker run --rm ^
    --privileged ^
    -v "%cd%":/project_cartesian ^
    cartesian-packager
IF %ERRORLEVEL% NEQ 0 goto :error

echo.
echo ==========================================
echo   SUCCESS: Check iso/out/
echo ==========================================
pause
exit /b

:error
echo.
echo [ERROR] Build Failed.
pause
exit /b