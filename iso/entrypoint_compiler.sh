#!/bin/bash
set -e

echo ">> [Compiler] Sanitizing Stage 1 Script..."
cp iso/build_stage1.sh /tmp/build_stage1.sh
dos2unix /tmp/build_stage1.sh
chmod +x /tmp/build_stage1.sh

echo ">> [Compiler] Executing Stage 1..."
/tmp/build_stage1.sh