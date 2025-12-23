# ==============================================================================
# CartesianOS User Environment Setup
# Path: iso/archiso_profile/airootfs/home/cartesian/.bash_profile
# ==============================================================================

# 1. Localization
export LANG=en_US.UTF-8

# 2. XDG & Wayland Protocol
export USER_ID=$(id -u)
export XDG_RUNTIME_DIR=/run/user/$USER_ID

export XDG_SESSION_TYPE=wayland
export XDG_CURRENT_DESKTOP=Hyprland

# 3. VM/VirtualBox SPECIFIC FIXES (Software Rendering Fallbacks)
export WLR_RENDERER_ALLOW_SOFTWARE=1
export WLR_NO_HARDWARE_CURSORS=1
export LIBGL_ALWAYS_SOFTWARE=1

# 4. Standard Path
export PATH="$PATH:$HOME/.local/bin:$HOME/.cargo/bin"

# 5. Clean Session Launch
if [[ -z "$DISPLAY" ]] && [[ "$(tty)" == "/dev/tty1" ]]; then
    echo "Starting CartesianOS Interface..."
    exec Hyprland
fi
