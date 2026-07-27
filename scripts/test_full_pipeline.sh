#!/bin/bash
# Quick test: source → executable → run
set -e
cd "$(dirname "$0")/.."

export LLVM_SYS_181_PREFIX=/tmp/llvm-wrap
export PATH="/tmp/llvm-wrap/bin:$PATH"

echo "➜ Compiling + running full pipeline..."
cargo run -p schiro-codegen --example compile 2>&1

echo ""
echo "➜ Running compiled binary..."
if [ -f /tmp/schiro_test_output ]; then
    chmod +x /tmp/schiro_test_output
    /tmp/schiro_test_output
    echo "Exit code: $?"
fi
