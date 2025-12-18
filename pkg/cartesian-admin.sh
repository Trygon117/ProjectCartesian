#!/bin/bash

# ==============================================================================
# CartesianOS Administrative Bridge
# Hardened script for privileged AI-System interaction.
# ==============================================================================

if [[ $EUID -ne 0 ]]; then
   echo "Error: Cartesian-Admin must be run as root (use pkexec)."
   exit 1
fi

COMMAND=$1

# --- PID Resolution Logic ---
# Instead of trusting a PID from the AI (which could be injected),
# we resolve the PID of our core process internally.
resolve_core_pid() {
    local pid
    pid=$(pgrep -x "cartesian-core")
    if [[ -z "$pid" ]]; then
        echo "Error: cartesian-core process not found."
        exit 1
    fi
    echo "$pid"
}

case "$COMMAND" in
    telemetry)
        # Safe Tier: Read-only system state
        # In the future, this will extract GPU/Vision metrics.
        echo "LOG: Fetching system telemetry..."
        uptime -p
        free -h
        ;;

    lobotomy)
        # Admin Tier: Suspend AI
        TARGET_PID=$(resolve_core_pid)
        echo "LOG: Suspending Cartesian Core (PID: $TARGET_PID)"
        kill -STOP "$TARGET_PID"
        ;;

    wakeup)
        # Admin Tier: Resume AI
        TARGET_PID=$(resolve_core_pid)
        echo "LOG: Resuming Cartesian Core (PID: $TARGET_PID)"
        kill -CONT "$TARGET_PID"
        ;;

    leash)
        # Admin Tier: Throttle AI (Gaming Mode)
        TARGET_PID=$(resolve_core_pid)
        echo "LOG: Applying Leash (renice 19) to PID: $TARGET_PID"
        renice -n 19 -p "$TARGET_PID"
        ;;

    *)
        echo "Error: Unknown command '$COMMAND'"
        exit 1
        ;;
esac