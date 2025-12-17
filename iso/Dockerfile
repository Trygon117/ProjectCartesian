# Use the official Arch Linux base image
FROM archlinux:latest

# Added lsof to the dependencies
RUN pacman -Sy --noconfirm && \
    pacman -S --noconfirm archiso git python3 make dos2unix grub efibootmgr lsof

WORKDIR /project_cartesian

CMD ["/bin/bash", "-c", "echo 'Running mandatory CRLF sanitization...'; dos2unix ./iso/build.sh; dos2unix ./iso/archiso_profile/profiledef.sh; dos2unix ./iso/archiso_profile/pacman.conf; find ./iso/archiso_profile -type f -print0 | xargs -0 dos2unix; echo 'Sanitization complete. Starting build.'; ./iso/build.sh"]