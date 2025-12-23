#!/usr/bin/env bash
# Project Cartesian Profile Definition

iso_name="cartesian"
iso_label="CARTESIAN_LIVE"
iso_publisher="Project Cartesian <https://github.com/project-cartesian>"
iso_application="Project Cartesian Live Environment"
iso_version="0.1.0-alpha" 

install_dir="arch"
buildmodes=('iso')
bootmodes=('bios.syslinux.mbr' 'bios.syslinux.eltorito' 'uefi.grub')

# Kernel Command Line: 
kernel_options="archisobasedir=%INSTALL_DIR% archisolabel=CARTESIAN_LIVE systemd.unified_cgroup_hierarchy=1 copytoram rootdelay=10"

arch="x86_64"
pacman_conf="pacman.conf"
airootfs_image_type="squashfs"
airootfs_image_tool_options=('-comp' 'zstd')

# --- PERMISSIONS PROTOCOL ---
file_permissions=(
  ["/etc/shadow"]="0:0:400"
  ["/etc/passwd"]="0:0:644"
  ["/etc/sudoers"]="0:0:440"
  ["/etc/group"]="0:0:644"
  ["/etc/locale.gen"]="0:0:644"
  ["/etc/locale.conf"]="0:0:644"
  ["/root"]="0:0:750"
  ["/home/cartesian"]="1000:1000:750"
  ["/home/cartesian/.cargo"]="1000:1000:750"
  ["/home/cartesian/.bashrc"]="1000:1000:644"
  ["/home/cartesian/.bash_profile"]="1000:1000:644"
)