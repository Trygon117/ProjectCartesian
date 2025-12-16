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

echo "=== PROJECT CARTESIAN ISO BUILDER ==="
echo "Project Root: $PROJECT_ROOT"

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

# Copy artifacts (Dereference symlinks just in case)
cp -r -L "$SOURCE_PATH/." "$BUILD_REPO_DIR/"
echo "Artifacts ready."

# Start HTTP Server (Bind to all interfaces 0.0.0.0 to ensure visibility)
echo "🚀 Starting temporary HTTP repo server on port $HTTP_PORT..."
python3 -m http.server "$HTTP_PORT" --directory "$BUILD_REPO_DIR" --bind 0.0.0.0 > /dev/null 2>&1 &
SERVER_PID=$!

# Give the server a moment to bind
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

    # Kill the HTTP server
    if kill -0 $SERVER_PID 2>/dev/null; then
        echo "Stopping HTTP server (PID $SERVER_PID)..."
        kill $SERVER_PID
    fi

    # Restore config
    echo "Restoring original pacman.conf..."
    if [ -f "$PROFILE_DIR/pacman.conf.bak" ]; then
        mv "$PROFILE_DIR/pacman.conf.bak" "$PROFILE_DIR/pacman.conf"
    fi
}
trap cleanup EXIT

# Clean old [cartesian] blocks
sed -i '/^\[cartesian\]/,$d' "$PROFILE_DIR/pacman.conf.bak"

# Inject HTTP URL
cat >> "$PROFILE_DIR/pacman.conf.bak" <<EOF

# --- INJECTED BY BUILD.SH ---
[cartesian]
SigLevel = Optional TrustAll
Server = http://127.0.0.1:$HTTP_PORT
EOF

# Apply config
mv "$PROFILE_DIR/pacman.conf.bak" "$PROFILE_DIR/pacman.conf"

echo "=== DEBUG: Injected Pacman Configuration ==="
tail -n 5 "$PROFILE_DIR/pacman.conf"
echo "=========================================="

echo "Starting mkarchiso..."
cd "$SCRIPT_DIR"

# Clean work dir to ensure fresh DB configs
if [ -d "work" ]; then
    sudo rm -rf work
fi

sudo mkarchiso -v -w work -o out "$PROFILE_DIR"

# --- STEP 4: Reporting ---
echo ""
echo "=== Build Complete ==="
ISO_OUT_DIR="$SCRIPT_DIR/out"

# Find the generated ISO (Looking for cartesian-* prefix now)
GENERATED_ISO=$(find "$ISO_OUT_DIR" -maxdepth 1 -name "cartesian-*.iso" -print -quit)

if [ -f "$GENERATED_ISO" ]; then
    echo "✅ SUCCESS: ISO generated successfully."
    echo "📂 File: $GENERATED_ISO"
else
    echo "⚠️  Build finished, but 'cartesian-*.iso' was not found."
    echo "   Check $ISO_OUT_DIR manually. If you see 'archlinux-*.iso', check profiledef.sh."
fi
