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

echo [1/2] Building the Builder Container...
docker build -t cartesian-builder -f iso/Dockerfile .

echo.
echo [2/2] Running Build Sequence...
echo NOTE: This requires --privileged mode to mount loop devices for the ISO.
echo.

REM -v mounts the current folder (%cd%) to /project_cartesian inside the box
REM --privileged is required for mkarchiso to work
REM --rm deletes the container after it finishes (clean up)

docker run --privileged --rm -v "%cd%":/project_cartesian cartesian-builder

echo.
echo ==========================================
echo   Build Complete. Check iso/out/ folder.
echo ==========================================
pause
