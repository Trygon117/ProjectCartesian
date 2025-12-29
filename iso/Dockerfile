FROM archlinux:latest

# Update system and install build AND runtime dependencies
# AUDIT FIX: Added 'gnupg' for package signing (Audit 2.4 dependency)
RUN pacman -Syu --noconfirm \
    base-devel \
    rust \
    cargo \
    archiso \
    dos2unix \
    grub \
    efibootmgr \
    git \
    sudo \
    wayland \
    libxkbcommon \
    vulkan-icd-loader \
    polkit \
    gnupg \
    fontconfig \
    clang \
    cmake \
    pipewire

# --- SECURITY & PACKAGING PROTOCOL ---
RUN useradd -m builder && \
    echo "builder ALL=(ALL) NOPASSWD: ALL" >> /etc/sudoers

WORKDIR /project_cartesian

# AUDIT 4.1 FIX: Centralized Entrypoint
# We ensure build.sh has correct line endings before executing.
ENTRYPOINT ["/bin/bash", "-c", "dos2unix -q /project_cartesian/iso/build.sh && chown -R builder:builder /project_cartesian && sudo -u builder /bin/bash /project_cartesian/iso/build.sh"]