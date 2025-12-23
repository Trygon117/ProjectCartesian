#!/bin/bash
# ==============================================================================
# CartesianOS Build Orchestrator (Hardened Version)
# Refactored for Docker-on-Windows Stability & Full Pipeline Logic
# ==============================================================================

set -e

# --- PATH RESOLUTION ---
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
    echo -e "\e[34m>>\e[0m $1" | tee -a "$LOG_FILE"
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
    # Exclude build artifacts and logs from the scan to save time
    find "$PROFILE_DIR" "$PKG_DIR" "$SCRIPT_DIR" \
        -path "$SRC_DIR/target" -prune -o \
        -path "$SCRIPT_DIR/out" -prune -o \
        -path "$LOG_DIR" -prune -o \
        -type f \( -name "*.sh" -o -name "PKGBUILD" -o -name "*.conf" -o -name "profiledef.sh" -o -name "shadow" -o -name "passwd" \) \
        -exec dos2unix -q {} +
}

function check_resources {
    MEM_LIMIT=$(free -m | awk '/Mem:/ { print $2 }')
    if [ "$MEM_LIMIT" -lt 4000 ]; then
        log "\e[33m[WARNING]\e[0m Low memory detected ($MEM_LIMIT MB). ISO generation may fail with 'unexpected EOF'."
    fi
}

check_dependencies
sanitize_files
check_resources

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
    
    log "🔗 Syncing package to local repository..."
    for pkg in "$PKG_DIR"/*.pkg.tar.zst; do
        ln -sf "$pkg" "$REPO_DIR/$(basename "$pkg")"
    done
    
    cd "$REPO_DIR"
    repo-add "cartesian.db.tar.gz" *.pkg.tar.zst >> "$LOG_FILE" 2>&1
    log "✅ Compilation/Packaging Complete."
else
    log "⏭️ Source unchanged. Skipping compilation."
fi

# --- STEP 3: ISO GENERATION ---
log "📀 Starting ISO Generation"

# Purge internal temp dirs
sudo rm -rf "$TEMP_WORK_DIR" "$TEMP_OUT_DIR"
mkdir -p "$TEMP_OUT_DIR"

cd "$SCRIPT_DIR"

# Run mkarchiso using the fast internal /tmp storage
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

# Final Cleanup of container-internal work dir
sudo rm -rf "$TEMP_WORK_DIR"