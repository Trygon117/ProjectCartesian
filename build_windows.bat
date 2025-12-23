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

REM Ensure logs directory exists
if not exist "logs\build" mkdir "logs\build"

echo [1/2] Building the Builder Container...
docker build -t cartesian-builder -f iso/Dockerfile .

echo.
echo [2/2] Running Build Sequence...
docker run --rm -v "%cd%":/work alpine sh -c "apk add --no-cache dos2unix && find /work/iso/archiso_profile -type f -exec dos2unix {} +"
echo NOTE: Persistent volumes are used for Rust 'target' and Arch 'repo'.
echo.

REM SECURITY FIX (Audit 2.3): Replaced --privileged with granular capabilities.
REM --cap-add SYS_ADMIN: Required for 'mount' commands in mkarchiso.
REM --cap-add MKNOD: Required to create loop devices.
REM --security-opt apparmor:unconfined: Prevents permission errors on loopback mounting.
REM --device /dev/fuse: Often needed for user-space filesystems (optional but recommended).

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
echo   Check logs/build/ for build details.
echo ==========================================
pause