# This file is executed after the 'cartesian' user is autologged in via systemd.

# --- BASIC LOCALIZATION ---
export LANG=en_US.UTF-8

# --- WAYLAND ENVIRONMENT VARIABLES ---
# Critical for Kitty and Hyprland to communicate
export USER_ID=$(id -u)
export XDG_RUNTIME_DIR=/run/user/$USER_ID
export XDG_SESSION_TYPE=wayland
export XDG_CURRENT_DESKTOP=Hyprland

# VM/VirtualBox SPECIFIC FIXES (Software Rendering Fallbacks)
export WLR_RENDERER_ALLOW_SOFTWARE=1
export WLR_NO_HARDWARE_CURSORS=1
export LIBGL_ALWAYS_SOFTWARE=1

# Ensure the runtime directory exists and is owned by cartesian
if [ ! -d "$XDG_RUNTIME_DIR" ]; then
    mkdir -p "$XDG_RUNTIME_DIR"
    chmod 700 "$XDG_RUNTIME_DIR"
fi

# --- AUTOSTART LOGIC ---
if [[ -z $DISPLAY_SKIPPED ]] && [[ $(tty) = /dev/tty1 ]]; then
    echo "🚀 Starting Project Cartesian Interface..."
    
    # Reverting to the non-exec version. 
    # This keeps the shell environment resident and prevents process tree collapse.
    if [ -z "$HYPRLAND_INSTANCE_SIGNATURE" ]; then
        dbus-run-session Hyprland > /tmp/hyprland.log 2>&1
    fi
fi