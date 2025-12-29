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

# 3. Standard Path
export PATH="$PATH:$HOME/.local/bin:$HOME/.cargo/bin"

# --- SMART VM DETECTION  ---
# Logic: Check if we are in a VM. If so, enable compatibility flags.
# If Native, do NOTHING (allow full hardware speed).

if systemd-detect-virt -q || hostnamectl status | grep -q "Chassis: vm"; then
    echo " [SYSTEM] Virtual Environment Detected."
    echo " [SYSTEM] Engaging Compatibility Layers..."
    
    # 1. Fix Invisible Cursor (Hyprland)
    export WLR_NO_HARDWARE_CURSORS=1
    
    # 2. Fix Desktop Rendering (Hyprland)
    export WLR_RENDERER_ALLOW_SOFTWARE=1
    
    # 3. FIX FOR KITTY / LAG
    # Force applications (like Kitty) to use CPU rendering if GPU is flaky.
    # This prevents the "Kitty won't launch" OpenGL error.
    export LIBGL_ALWAYS_SOFTWARE=1
else
    echo " [SYSTEM] Native Hardware Detected."
    echo " [SYSTEM] Running in High-Performance Mode."
fi

# 4. Clean Session Launch
if [[ -z "$DISPLAY" ]] && [[ "$(tty)" == "/dev/tty1" ]]; then
    echo "Starting CartesianOS Interface..."
    exec Hyprland
fi