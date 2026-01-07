#!/bin/bash
set -e

# --- CONFIGURATION ---
MOUNT_ROOT="/project_cartesian"
WORK_ROOT="/tmp/fast_work"
ARTIFACT_DIR="$MOUNT_ROOT/pkg/stage1_artifacts"

echo "--- STAGE 1: COMPILATION (Ubuntu) ---"

# 1. I/O Optimization
# We copy source code to internal storage to speed up compilation
echo "🚀 Syncing source to internal drive..."
mkdir -p "$WORK_ROOT"
rsync -rtD --exclude 'target' --exclude '.git' "$MOUNT_ROOT/" "$WORK_ROOT/"

# 2. Sanitization
# Even though we are on Ubuntu, the files came from Windows.
echo "🧹 Sanitizing..."
find "$WORK_ROOT" -type f -not -path '*/.git/*' -exec grep -Iq . {} \; -and -print0 | xargs -0 dos2unix -q

# 3. Compilation
echo "🚀 Building Release Binary..."
cd "$WORK_ROOT/src/cartesian-core"

# Explicitly use the cached target directory mounted from host
export CARGO_TARGET_DIR="$MOUNT_ROOT/src/cartesian-core/target"
# Ensure the mount is writable
mkdir -p "$CARGO_TARGET_DIR"

# FORCE COMPILER SETTINGS
export CC=clang
export CXX=clang++

# GPU TARGET FIX:
# Since we are building on a laptop (no GPU), nvidia-smi fails to detect arch.
# We hardcode '86' (RTX 3080 / Ampere) so the build knows what to target.
export CUDA_COMPUTE_CAP=86

cargo build --release

# 4. Harvesting
echo "📦 Harvesting Artifacts..."
mkdir -p "$ARTIFACT_DIR"

# Copy Binary
cp "$CARGO_TARGET_DIR/release/cartesian-core" "$ARTIFACT_DIR/"

# Copy Libraries
# We grab the CUDA libs from the Ubuntu/Nvidia system so they travel with the app.
LIB_SOURCE="/usr/local/cuda/lib64"
cp "$LIB_SOURCE"/libcudart.so* "$ARTIFACT_DIR/"
cp "$LIB_SOURCE"/libcublas.so* "$ARTIFACT_DIR/"
cp "$LIB_SOURCE"/libcublasLt.so* "$ARTIFACT_DIR/"
cp "$LIB_SOURCE"/libcurand.so* "$ARTIFACT_DIR/"

echo "✅ Stage 1 Complete. Artifacts in $ARTIFACT_DIR"