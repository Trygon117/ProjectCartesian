#!/bin/bash
set -e

echo ">> [Packager] Sanitizing Stage 2 Script..."
cp iso/build_stage2.sh /tmp/build_stage2.sh
dos2unix /tmp/build_stage2.sh
chmod +x /tmp/build_stage2.sh

echo ">> [Packager] Executing Stage 2..."
sudo -E /tmp/build_stage2.sh