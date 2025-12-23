#!/bin/bash
# ==============================================================================
# CartesianOS Build Orchestrator (Hardened Version)
# Refactored for Docker-on-Windows Stability & Full Pipeline Logic
# ==============================================================================

set -e

# --- ERROR HANDLING & LOGGING ---
# AUDIT FIX: Trap errors and show the log tail so the user isn't blind.
function failure_handler {
    echo -e "\n\e[31m❌ BUILD FAILED: The script encountered an error on line $1.\e[0m"
    echo -e "\e[33m--- LAST 20 LINES OF LOG ($LOG_FILE) ---\e[0m"
    if [ -f "$LOG_FILE" ]; then
        tail -n 20 "$LOG_FILE"
    else
        echo "Log file not found."
    fi
    echo -e "\e[31m--------------------------------------------\e[0m"
    exit 1
}
# Trap passes the line number ($LINENO) to the handler
trap 'failure_handler $LINENO' ERR

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

# --- BANNER ---
echo "--- CartesianOS Build System v3.0 (Hardened) ---"

# --- PRE-FLIGHT ---
function check_dependencies {
    log "Checking build environment..."
    local deps=("cargo" "rustc" "makepkg" "mkarchiso" "dos2unix" "gpg")
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &> /dev/null; then
            log "❌ ERROR: Required tool '$dep' not found."
            exit 1
        fi
    done
}

function sanitize_files {
    log "Sanitizing line endings..."
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

# --- SECURITY: GPG SETUP ---
function setup_gpg {
    log "🔐 Initializing Ephemeral GPG Key for Signing..."
    export GNUPGHOME="$(mktemp -d)"
    
    # FIX: ALWAYS Force initialization.
    # We removed the 'smart check' because Docker keyrings are often ghosted/broken.
    log "Initializing system keyring (Wait ~10s)..."
    sudo rm -rf /etc/pacman.d/gnupg
    sudo pacman-key --init >> "$LOG_FILE" 2>&1
    sudo pacman-key --populate archlinux >> "$LOG_FILE" 2>&1

    # Generate a batch key for "Cartesian Builder"
    cat > "$GNUPGHOME/params" <<EOF
    %echo Generating builder key...
    Key-Type: RSA
    Key-Length: 2048
    Subkey-Type: RSA
    Subkey-Length: 2048
    Name-Real: Cartesian Builder
    Name-Email: builder@cartesian.local
    Expire-Date: 0
    %no-protection
    %commit
    %echo Done
EOF
    gpg --batch --generate-key "$GNUPGHOME/params" >> "$LOG_FILE" 2>&1
    
    # Extract Key ID
    GPG_KEY=$(gpg --list-secret-keys --with-colons | grep sec | cut -d: -f5)
    
    if [ -z "$GPG_KEY" ]; then
         log "❌ ERROR: GPG Key generation failed."
         exit 1
    fi
    log "🔑 Key Generated: $GPG_KEY"

    # TRUST THE KEY
    gpg --armor --export "$GPG_KEY" > "$GNUPGHOME/builder.asc"
    sudo pacman-key --add "$GNUPGHOME/builder.asc" >> "$LOG_FILE" 2>&1
    sudo pacman-key --lsign-key "$GPG_KEY" >> "$LOG_FILE" 2>&1
    log "✅ Key trusted by local pacman."
}

check_dependencies
sanitize_files
check_resources
setup_gpg

# --- STEP 2: COMPILATION & SIGNING ---
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
    makepkg -f --sign --key "$GPG_KEY" --noconfirm >> "$LOG_FILE" 2>&1
    
    mkdir -p "$REPO_DIR"
    
    log "🔗 Syncing package to local repository..."
    for pkg in "$PKG_DIR"/*.pkg.tar.zst; do
        ln -sf "$pkg" "$REPO_DIR/$(basename "$pkg")"
    done
    # Sync sigs
    for sig in "$PKG_DIR"/*.pkg.tar.zst.sig; do
        ln -sf "$sig" "$REPO_DIR/$(basename "$sig")"
    done

    cd "$REPO_DIR"
    repo-add --sign --key "$GPG_KEY" "cartesian.db.tar.gz" *.pkg.tar.zst >> "$LOG_FILE" 2>&1
    log "✅ Compilation/Packaging Complete."

else
    log "⏭️ Source unchanged. Skipping compilation."
    log "✍️ Re-signing existing artifacts with new key..."
    
    # AUDIT FIX: We MUST resign the old packages because the GPG Key is new.
    # Otherwise mkarchiso will reject them as 'invalid signature'.
    cd "$PKG_DIR"
    for pkg in *.pkg.tar.zst; do
        if [ -f "$pkg" ]; then
            # Detached signature
            rm -f "${pkg}.sig"
            gpg --batch --yes --detach-sign --no-armor --local-user "$GPG_KEY" --output "${pkg}.sig" "$pkg" >> "$LOG_FILE" 2>&1
        fi
    done

    # Sync new sigs to Repo
    mkdir -p "$REPO_DIR"
    for sig in "$PKG_DIR"/*.pkg.tar.zst.sig; do
        ln -sf "$sig" "$REPO_DIR/$(basename "$sig")"
    done

    # Update Repo DB Signature
    cd "$REPO_DIR"
    rm -f cartesian.db cartesian.db.sig cartesian.files cartesian.files.sig
    repo-add --sign --key "$GPG_KEY" "cartesian.db.tar.gz" *.pkg.tar.zst >> "$LOG_FILE" 2>&1
    log "✅ Re-signing Complete."
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
    
    echo -e "\n======================================================="
    echo -e "✅ \e[32mSUCCESS: ISO Generated Successfully!\e[0m"
    echo -e "📂 Location: iso/out/$(basename "$GENERATED_ISO")"
    echo -e "=======================================================\n"
else
    log "❌ ERROR: ISO generation failed (File not found). Check $LOG_FILE"
    exit 1
fi

# Final Cleanup
sudo rm -rf "$TEMP_WORK_DIR"
rm -rf "$GNUPGHOME"