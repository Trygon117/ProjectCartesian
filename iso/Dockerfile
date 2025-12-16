# Use the official Arch Linux base image
FROM archlinux:latest

# Initialize pacman keyring and update system
# We install:
# - archiso: The tool to build the ISO
# - git: Often needed for dependencies
# - python3: For our local HTTP server in build.sh
RUN pacman -Sy --noconfirm && \
    pacman -S --noconfirm archiso git python3 make

# Set the working directory inside the container
WORKDIR /project_cartesian

# By default, when this container runs, execute the build script
CMD ["/bin/bash", "./iso/build.sh"]
