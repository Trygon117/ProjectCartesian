#!/bin/bash
set -e

# --- CONFIGURATION ---
MOUNT_ROOT="/project_cartesian"
WORK_ROOT="/home/builder/fast_work"
ARTIFACT_SOURCE="$MOUNT_ROOT/pkg/stage1_artifacts"
# REMOVED LOG_FILE usage so we see output

echo "--- STAGE 2: PACKAGING (Arch) ---"

# 1. I/O Optimization
echo "🚀 Syncing source..."
sudo mkdir -p "$WORK_ROOT"
rsync -rtD --exclude 'target' --exclude '.git' "$MOUNT_ROOT/" "$WORK_ROOT/"
echo "🔧 Fixing permissions..."
sudo chown -R builder:builder "$WORK_ROOT"
sudo chmod -R u+rw "$WORK_ROOT"

# --- SANITIZATION ---
echo "🧹 Sanitizing project files..."
find "$WORK_ROOT" -type f -not -path '*/.git/*' -exec grep -Iq . {} \; -and -print0 | xargs -0 dos2unix -q
find "$WORK_ROOT" -name "*.sh" -exec chmod +x {} +

# 2. Mocking the Build Environment
echo "🎭 Mocking Target Directory..."
MOCK_TARGET="$WORK_ROOT/src/cartesian-core/target/release"
sudo -u builder mkdir -p "$MOCK_TARGET"

sudo cp "$ARTIFACT_SOURCE/cartesian-core" "$MOCK_TARGET/"
sudo chmod +x "$MOCK_TARGET/cartesian-core"
sudo chown builder:builder "$MOCK_TARGET/cartesian-core"

sudo -u builder mkdir -p "$WORK_ROOT/pkg/libs"
sudo cp "$ARTIFACT_SOURCE/"*.so* "$WORK_ROOT/pkg/libs/"
sudo chown builder:builder "$WORK_ROOT/pkg/libs/"*.so*

# 3. Packaging (makepkg)
echo "📦 Creating Arch Package..."
cd "$WORK_ROOT/pkg"

# GPG Setup (Root Context)
echo "🔐 Initializing GPG for Signing..."
export GNUPGHOME="$(mktemp -d)"

# Initialize Pacman Keyring
pacman-key --init
pacman-key --populate archlinux

# Generate Key (As Root)
cat > "$GNUPGHOME/params" <<EOF
%echo Generating builder key...
Key-Type: RSA
Key-Length: 2048
Name-Real: Cartesian Builder
Name-Email: builder@local
Expire-Date: 0
%no-protection
%commit
EOF
gpg --batch --generate-key "$GNUPGHOME/params"
GPG_KEY=$(gpg --list-secret-keys --with-colons | grep sec | cut -d: -f5)
echo "🔑 Generated Key ID: $GPG_KEY"

# Export Key Pair
gpg --armor --export "$GPG_KEY" > "$GNUPGHOME/builder.pub"
gpg --armor --export-secret-keys "$GPG_KEY" > "$GNUPGHOME/builder.sec"

# Trust in Pacman
pacman-key --add "$GNUPGHOME/builder.pub"
pacman-key --lsign-key "$GPG_KEY"

# Ensure builder owns the pkg directory
sudo chown -R builder:builder "$WORK_ROOT/pkg"

# BUILD: Run makepkg (Unsigned)
echo "📦 Building Package (Unsigned)..."
sudo -u builder makepkg -f --noconfirm

# SIGN: Manually sign the artifacts using Root's GPG context
echo "✍️  Manually Signing Packages..."
for pkg in *.pkg.tar.zst; do
    echo "   Signing $pkg..."
    gpg --detach-sign --no-armor --default-key "$GPG_KEY" "$pkg"
done

# 4. Repo Setup
echo "🔗 Setting up Local Repo..."
REPO_DIR="$WORK_ROOT/repo/x86_64"
mkdir -p "$REPO_DIR"
cp *.pkg.tar.zst "$REPO_DIR/"
cp *.pkg.tar.zst.sig "$REPO_DIR/"
cd "$REPO_DIR"
rm -f cartesian.db* cartesian.files*

# Sign the repo DB
repo-add --sign --key "$GPG_KEY" "cartesian.db.tar.gz" *.pkg.tar.zst

# FIX: Sync the repo back to the mount point NOW so mkarchiso can find it
echo "🔄 Syncing Repo to Host for visibility..."
mkdir -p "$MOUNT_ROOT/repo/x86_64"
cp -r "$REPO_DIR/"* "$MOUNT_ROOT/repo/x86_64/"

# 5. ISO Generation
echo "📀 Generating ISO..."
cd "$WORK_ROOT/iso"

# Run mkarchiso (Output visible)
sudo mkarchiso -v -w /tmp/archiso_work -o /tmp/archiso_out "$WORK_ROOT/iso/archiso_profile"

# 6. Artifact Sync
GENERATED_ISO=$(find /tmp/archiso_out -maxdepth 1 -name "cartesian-*.iso" -print -quit)
if [ -f "$GENERATED_ISO" ]; then
    echo "💾 Saving ISO..."
    cp "$GENERATED_ISO" "$MOUNT_ROOT/iso/out/"
    # Repo is already synced above, no need to do it again
    echo "✅ SUCCESS."
else
    echo "❌ ISO Generation Failed."
    exit 1
fi