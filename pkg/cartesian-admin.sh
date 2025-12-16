#!/bin/bash
if [[ $EUID -ne 0 ]]; then
   echo "This script must be run as root (use pkexec)."
   exit 1
fi
COMMAND=$1
case "$COMMAND" in
    lobotomy)
        TARGET_PID=$2
        if [[ "$TARGET_PID" -lt 1000 ]]; then
            echo "Error: Safety Lock. Cannot lobotomize system process PID $TARGET_PID"
            exit 1
        fi
        kill -STOP "$TARGET_PID"
        ;;
    wakeup)
        TARGET_PID=$2
        kill -CONT "$TARGET_PID"
        ;;
    leash)
        TARGET_PID=$2
        renice -n 19 -p "$TARGET_PID"
        ;;
    *)
        echo "Unknown command: $COMMAND"
        exit 1
        ;;
esac
