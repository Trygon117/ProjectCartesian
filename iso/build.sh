#!/bin/bash

# Exit on error immediately
set -e

# Get directory of this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Resolve paths
PROJECT_ROOT="$(realpath "$SCRIPT_DIR/..")"
REPO_DIR="$(realpath "$PROJECT_ROOT/repo")"
PKG_DIR="$(realpath "$PROJECT_ROOT/pkg")"
PROFILE_DIR="$(realpath "$SCRIPT_DIR/archiso_profile")"

# Local directory for the build repo
BUILD_REPO_DIR="$SCRIPT_DIR/local_repo"
HTTP_PORT="8050"

# --- NEW: Internal Linux Temporary Paths for Performance/Stability (War Story Fix) ---
TEMP_WORK_DIR="/tmp/mkarchiso_work"
TEMP_OUT_DIR="/tmp/mkarchiso_out"

echo "=== PROJECT CARTESIAN ISO BUILDER ==="

# --- NEW: PRE-FLIGHT CLEANUP ---
# This addresses the user's issue where an interrupted build leaves 
# lock files or mounted loop devices that prevent new builds.
function pre_build_cleanup {
    echo "🔍 Running pre-flight cleanup..."
    
    # 1. Clear internal temp directories if they exist from a crashed run
    if [ -d "$TEMP_WORK_DIR" ]; then
        echo "   -> Removing stale work directory: $TEMP_WORK_DIR"
        rm -rf "$TEMP_WORK_DIR" || (echo "      ⚠️ Failed to remove work dir. Check permissions." && exit 1)
    fi
    
    # 2. Check for port 8050 usage
    P_PID=$(lsof -t -i:$HTTP_PORT || true)
    if [ -n "$P_PID" ]; then
        echo "   -> Port $HTTP_PORT is occupied by PID $P_PID. Terminating..."
        kill -9 "$P_PID" || true
    fi

    # 3. Restore pacman.conf if a backup exists from a crashed run
    if [ -f "$PROFILE_DIR/pacman.conf.bak" ]; then
        echo "   -> Restoring pacman.conf from backup..."
        mv "$PROFILE_DIR/pacman.conf.bak" "$PROFILE_DIR/pacman.conf"
    fi
}

# Run cleanup before starting
pre_build_cleanup

# --- CRITICAL PROTOCOL: CRLF Line Ending Fix ---
echo "✅ Line ending sanitization handled by Docker entrypoint."


# --- STEP 1: Repository Generation ---
if [ -d "$REPO_DIR/x86_64" ]; then
    echo "✅ Source repository found at: $REPO_DIR/x86_64"
    SOURCE_PATH="$REPO_DIR/x86_64"
else
    echo "⚠️  Repository not found. Attempting to generate from 'pkg'..."
    PKG_FOUND=$(find "$PKG_DIR" -maxdepth 1 -name "*.pkg.tar.zst" -print -quit)
    if [ -n "$PKG_FOUND" ]; then
        mkdir -p "$REPO_DIR/x86_64"
        cp "$PKG_DIR"/*.pkg.tar.zst "$REPO_DIR/x86_64/"
        echo "Generating Pacman Database..."
        cd "$REPO_DIR/x86_64"
        repo-add "cartesian.db.tar.gz" *.pkg.tar.zst
        cd "$SCRIPT_DIR"
        SOURCE_PATH="$REPO_DIR/x86_64"
        echo "✅ Repository generated."
    else
        echo "❌ ERROR: No compiled packages found in $PKG_DIR"
        exit 1
    fi
fi

# --- STEP 2: Safe Build Environment & HTTP Server ---
echo "Setting up local build repo at: $BUILD_REPO_DIR"
if [ -d "$BUILD_REPO_DIR" ]; then rm -rf "$BUILD_REPO_DIR"; fi
mkdir -p "$BUILD_REPO_DIR"
cp -r -L "$SOURCE_PATH/." "$BUILD_REPO_DIR/"

echo "🚀 Starting temporary HTTP repo server on port $HTTP_PORT..."
python3 -m http.server "$HTTP_PORT" --directory "$BUILD_REPO_DIR" --bind 0.0.0.0 > /dev/null 2>&1 &
SERVER_PID=$!
sleep 2

if kill -0 $SERVER_PID 2>/dev/null; then
    echo "Server running with PID: $SERVER_PID"
else
    echo "❌ Server failed to start!"
    exit 1
fi

# --- STEP 3: Config Injection & Execution ---
cp "$PROFILE_DIR/pacman.conf" "$PROFILE_DIR/pacman.conf.bak"

function cleanup {
    echo ""
    echo "🧹 Cleaning up..."
    if kill -0 $SERVER_PID 2>/dev/null; then
        echo "Stopping HTTP server (PID $SERVER_PID)..."
        kill $SERVER_PID
    fi
    echo "Restoring original pacman.conf..."
    if [ -f "$PROFILE_DIR/pacman.conf.bak" ]; then
        mv "$PROFILE_DIR/pacman.conf.bak" "$PROFILE_DIR/pacman.conf"
    fi
    echo "Cleaning up internal mkarchiso work directory: $TEMP_WORK_DIR"
    rm -rf "$TEMP_WORK_DIR"
}
trap cleanup EXIT

sed -i '/^\[cartesian\]/,$d' "$PROFILE_DIR/pacman.conf.bak"
cat >> "$PROFILE_DIR/pacman.conf.bak" <<EOF

# --- INJECTED BY BUILD.SH ---
[cartesian]
SigLevel = Optional TrustAll
Server = http://127.0.0.1:$HTTP_PORT
EOF

mv "$PROFILE_DIR/pacman.conf.bak" "$PROFILE_DIR/pacman.conf"
echo "Starting mkarchiso..."
cd "$SCRIPT_DIR"
mkdir -p "$TEMP_OUT_DIR"
mkarchiso -v -w "$TEMP_WORK_DIR" -o "$TEMP_OUT_DIR" "$PROFILE_DIR"

# --- STEP 4: Copy Result Back ---
echo ""
echo "=== Build Complete ==="
GENERATED_ISO_TEMP_PATH=$(find "$TEMP_OUT_DIR" -maxdepth 1 -name "cartesian-*.iso" -print -quit)
if [ -f "$GENERATED_ISO_TEMP_PATH" ]; then
    FINAL_ISO_OUT_DIR="$SCRIPT_DIR/out"
    mkdir -p "$FINAL_ISO_OUT_DIR"
    cp "$GENERATED_ISO_TEMP_PATH" "$FINAL_ISO_OUT_DIR/"
    FINAL_ISO_PATH="$FINAL_ISO_OUT_DIR/$(basename "$GENERATED_ISO_TEMP_PATH")"
    echo "✅ SUCCESS: ISO generated successfully."
else
    echo "⚠️  Build finished, but ISO was not found."
fi