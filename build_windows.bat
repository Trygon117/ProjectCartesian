@echo off
echo ==========================================
echo   Project Cartesian: Windows Build Wrapper
echo ==========================================

REM Check if Docker is running
docker info >nul 2>&1
IF %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Docker is not running!
    echo Please install Docker Desktop and start it.
    pause
    exit /b
)

REM --- SMART VERSION CHECK ---
echo [0/2] Checking Package Version State...

REM Extract version from PKGBUILD using simple string parsing
set "PKG_VER="
set "PKG_REL="

if not exist "pkg\PKGBUILD" (
    echo [WARN] PKGBUILD not found in pkg/. Assuming fresh build.
    set NEED_CLEAN=y
    goto :DoClean
)

for /f "tokens=2 delims==" %%a in ('findstr "pkgver=" pkg\PKGBUILD') do set PKG_VER=%%a
for /f "tokens=2 delims==" %%a in ('findstr "pkgrel=" pkg\PKGBUILD') do set PKG_REL=%%a

REM Trim whitespace (Batch is finicky)
set PKG_VER=%PKG_VER: =%
set PKG_REL=%PKG_REL: =%

set "EXPECTED_PKG=cartesian-core-%PKG_VER%-%PKG_REL%-x86_64.pkg.tar.zst"
echo     - Target: %EXPECTED_PKG%

if exist "repo\x86_64\%EXPECTED_PKG%" (
    echo     - [OK] Package already exists in repo. Using Incremental Build.
    set NEED_CLEAN=n
) else (
    echo     - [UPDATE DETECTED] Target package missing from repo.
    echo     - Forcing Clean Build to ensure version update.
    set NEED_CLEAN=y
)

:DoClean
if "%NEED_CLEAN%"=="y" (
    echo.
    echo [AUTO-CLEAN] Wiping stale artifacts...
    
    if exist "repo\x86_64" (
        echo     - Deleting old repo database...
        rmdir /s /q "repo\x86_64"
    )
    if exist "iso\out" (
        echo     - Deleting old ISOs...
        rmdir /s /q "iso\out"
    )
    
    REM Clean the pkg directory of old artifacts but keep source files
    if exist "pkg\*.pkg.tar.zst" del /q "pkg\*.pkg.tar.zst"
    
    echo     - Clean complete.
)

REM Ensure directories exist
if not exist "logs\build" mkdir "logs\build"
if not exist "repo\x86_64" mkdir "repo\x86_64"
if not exist "iso\out" mkdir "iso\out"

echo [1/2] Building the Builder Container...
docker build -t cartesian-builder -f iso/Dockerfile .

echo.
echo [2/2] Running Build Sequence...

REM --- CONSOLIDATED SANITIZATION STEP ---
docker run --rm -v "%cd%":/work alpine sh -c "apk add --no-cache dos2unix && dos2unix /work/iso/build.sh /work/pkg/*.sh && find /work/iso/archiso_profile -type f -exec dos2unix {} +"

echo NOTE: Persistent volumes are used for Rust 'target'.
echo.

REM SECURITY FIX: Capabilities instead of --privileged
docker run --rm ^
    --cap-add SYS_ADMIN ^
    --cap-add MKNOD ^
    --security-opt apparmor:unconfined ^
    --device /dev/fuse ^
    -v "%cd%":/project_cartesian ^
    -v "%cd%\src\cartesian-core\target":/project_cartesian/src/cartesian-core/target ^
    cartesian-builder

echo.
echo ==========================================
echo   Build Sequence Finished.
echo   Check iso/out/ for the ISO.
echo ==========================================
pause