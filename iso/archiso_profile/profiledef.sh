#!/usr/bin/env bash
# Project Cartesian Profile Definition

iso_name="cartesian"
# Using a fixed, simple label to avoid macro expansion issues in the bootloader
iso_label="CARTESIAN_LIVE"
iso_publisher="Project Cartesian <https://github.com/project-cartesian>"
iso_application="Project Cartesian Live Environment"
iso_version="$(date +%Y.%m.%d)"
install_dir="arch"
buildmodes=('iso')

# Modern simplified GRUB UEFI bootmodes
bootmodes=('bios.syslinux.mbr' 'bios.syslinux.eltorito' 'uefi.grub')

# Kernel Command Line: 
# 1. We hardcode the label check to match iso_label exactly.
kernel_options="archisobasedir=%INSTALL_DIR% archisolabel=CARTESIAN_LIVE cow_label=CARTESIAN_LIVE systemd.unified_cgroup_hierarchy=1 copytoram"

arch="x86_64"
pacman_conf="pacman.conf"
airootfs_image_type="squashfs"
airootfs_image_tool_options=('-comp' 'zstd')

# --- PERMISSIONS PROTOCOL ---
file_permissions=(
  ["/etc/shadow"]="0:0:400"
  ["/etc/passwd"]="0:0:644"
  ["/root"]="0:0:750"
  ["/home/cartesian"]="1000:1000:750"
  ["/home/cartesian/.cargo"]="1000:1000:750"
  ["/home/cartesian/.bashrc"]="1000:1000:644"
)