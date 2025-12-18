#!/bin/bash

# Project Cartesian: Linux Native Build Wrapper
# This script assumes you are running on an Arch Linux host (or similar)
# with the necessary build tools installed.

# Exit on error
set -e

echo "=========================================="
echo "   Project Cartesian: Linux Build Wrapper"
echo "=========================================="

# 1. Ensure logs directory exists
mkdir -p logs/build

# 2. Check for root (mkarchiso requires it)
if [ "$EUID" -ne 0 ]; then
  echo "[ERROR] Please run this script with sudo."
  exit 1
fi

# 3. Make internal build script executable
chmod +x iso/build.sh

# 4. Execute the build process
# We don't need Docker here because we are already in Linux.
# We call build.sh directly.
./iso/build.sh

echo ""
echo "=========================================="
echo "   Build Complete. Check iso/out/ folder."
echo "=========================================="