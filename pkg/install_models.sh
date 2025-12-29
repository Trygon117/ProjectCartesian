#!/bin/bash

# ==============================================================================
# CartesianOS: First Boot Model Installer
# Downloads the AI "Brains" (Gemma + BERT) if they are missing.
# ==============================================================================

# --- CONFIGURATION ---
MODEL_DIR="/usr/share/cartesian/models"
INSTALL_MARKER="$MODEL_DIR/.installed"

# 1. The Manager/Sidekick (Gemma 2B Quantized) - ~1.7GB
GEMMA_URL="https://huggingface.co/TheBloke/Gemma-2B-IT-GGUF/resolve/main/gemma-2b-it.Q4_K_M.gguf"
GEMMA_FILE="gemma-2b-it.gguf"

# 2. The Hippocampus (BERT Embedding) - ~90MB
BERT_URL="https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/model.safetensors"
BERT_FILE="all-MiniLM-L6-v2.safetensors"

# --- LOGIC ---

echo ":: CartesianOS Model Installer"
echo ":: Target Directory: $MODEL_DIR"

# Check if already installed
if [ -f "$INSTALL_MARKER" ]; then
    echo ":: Models already installed. Skipping."
    exit 0
fi

# Ensure we have root privileges to write to /usr/share
if [[ $EUID -ne 0 ]]; then
   echo ":: Requesting System Privileges for installation..."
   # Check if pkexec exists
   if command -v pkexec >/dev/null 2>&1; then
       exec pkexec "$0" "$@"
   else
       echo ":: [ERROR] pkexec not found. Cannot escalate privileges."
       echo ":: Please run this script as root: sudo $0"
       exit 1
   fi
   exit 1
fi

# Create directory if missing
mkdir -p "$MODEL_DIR"

# Function to download if missing
download_model() {
    local url=$1
    local filename=$2
    local filepath="$MODEL_DIR/$filename"

    if [ -f "$filepath" ]; then
        echo ":: [SKIP] $filename already exists."
    else
        echo ":: [DOWNLOADING] $filename..."
        # Check if curl exists
        if ! command -v curl >/dev/null 2>&1; then
             echo ":: [ERROR] curl is missing. Installing..."
             pacman -S --noconfirm curl
        fi

        if curl -L -# "$url" -o "$filepath"; then
            echo ":: [SUCCESS] $filename downloaded."
        else
            echo ":: [ERROR] Failed to download $filename."
            rm -f "$filepath"
            exit 1
        fi
    fi
}

# Execute Downloads
echo "--------------------------------------------------------"
download_model "$GEMMA_URL" "$GEMMA_FILE"
download_model "$BERT_URL" "$BERT_FILE"
echo "--------------------------------------------------------"

# Fix Permissions
echo ":: Setting permissions..."
chmod 644 "$MODEL_DIR"/*
chmod 755 "$MODEL_DIR"

# Create marker file
touch "$INSTALL_MARKER"

echo ":: Installation Complete."
echo ":: Please restart Cartesian Core to load the new models."
sleep 2