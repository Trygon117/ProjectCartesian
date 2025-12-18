FROM archlinux:latest

# Update system and install build AND runtime dependencies
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
    polkit

# --- SECURITY & PACKAGING PROTOCOL ---
RUN useradd -m builder && \
    echo "builder ALL=(ALL) NOPASSWD: ALL" >> /etc/sudoers

WORKDIR /project_cartesian

ENTRYPOINT ["/bin/bash", "-c", "chown -R builder:builder /project_cartesian && dos2unix /project_cartesian/iso/build.sh && sudo -u builder /bin/bash /project_cartesian/iso/build.sh"]