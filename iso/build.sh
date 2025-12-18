#!/bin/bash

# Exit on error
set -e

# Path Resolution
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(realpath "$SCRIPT_DIR/..")"
SRC_DIR="$PROJECT_ROOT/src/cartesian-core"
PKG_DIR="$PROJECT_ROOT/pkg"
REPO_DIR="$PROJECT_ROOT/repo/x86_64"
PROFILE_DIR="$SCRIPT_DIR/archiso_profile"
LOG_DIR="$PROJECT_ROOT/logs/build"
LOG_FILE="$LOG_DIR/latest_build.log"

# Internal mkarchiso temps
TEMP_WORK_DIR="/tmp/mkarchiso_work"
TEMP_OUT_DIR="/tmp/mkarchiso_out"

mkdir -p "$LOG_DIR"
echo "--- Build Started $(date) ---" > "$LOG_FILE"

function log {
    echo ">> $1" | tee -a "$LOG_FILE"
}

# --- PRE-FLIGHT ---
function check_dependencies {
    log "Checking build environment..."
    local deps=("cargo" "rustc" "makepkg" "mkarchiso" "dos2unix")
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &> /dev/null; then
            log "❌ ERROR: Required tool '$dep' not found."
            exit 1
        fi
    done
}

function sanitize_files {
    log "Sanitizing line endings..."
    # We do this QUIETLY to avoid log spam
    find "$PKG_DIR" "$SCRIPT_DIR" -type f \( -name "*.sh" -o -name "PKGBUILD" -o -name "*.conf" -o -name "profiledef.sh" \) -exec dos2unix -q {} +
}

# --- STEP 1: PREPARE ---
check_dependencies
sanitize_files

# --- STEP 2: COMPILATION ---
function check_rebuild_required {
    if [ ! -d "$REPO_DIR" ] || [ -z "$(ls -A "$REPO_DIR"/*.pkg.tar.zst 2>/dev/null)" ]; then
        return 0
    fi
    LATEST_PKG=$(ls -t "$REPO_DIR"/*.pkg.tar.zst | head -n 1)
    CHANGES=$(find "$SRC_DIR/src" -type f -newer "$LATEST_PKG" | wc -l)
    [ "$CHANGES" -gt 0 ] && return 0 || return 1
}

if check_rebuild_required; then
    log "🚀 Compiling & Packaging Cartesian Core..."
    
    cd "$SRC_DIR"
    cargo build --release >> "$LOG_FILE" 2>&1
    
    cd "$PKG_DIR"
    makepkg -f --noconfirm >> "$LOG_FILE" 2>&1
    
    mkdir -p "$REPO_DIR"
    cp "$PKG_DIR"/*.pkg.tar.zst "$REPO_DIR/"
    cd "$REPO_DIR"
    repo-add "cartesian.db.tar.gz" *.pkg.tar.zst >> "$LOG_FILE" 2>&1
    
    log "✅ Compilation/Packaging Complete."
fi

# --- STEP 3: ISO GENERATION ---
log "📀 Starting ISO Generation..."

rm -rf "$TEMP_WORK_DIR"
mkdir -p "$TEMP_OUT_DIR"
cd "$SCRIPT_DIR"

# Standard mkarchiso run with output strictly to log file
sudo mkarchiso -v -w "$TEMP_WORK_DIR" -o "$TEMP_OUT_DIR" "$PROFILE_DIR" >> "$LOG_FILE" 2>&1

# --- STEP 4: EXPORT ---
GENERATED_ISO=$(find "$TEMP_OUT_DIR" -maxdepth 1 -name "cartesian-*.iso" -print -quit)
if [ -f "$GENERATED_ISO" ]; then
    mkdir -p "$SCRIPT_DIR/out"
    sudo cp "$GENERATED_ISO" "$SCRIPT_DIR/out/"
    log "✅ SUCCESS: ISO available in iso/out/$(basename "$GENERATED_ISO")"
else
    log "❌ ERROR: ISO generation failed. Check $LOG_FILE"
    exit 1
fi