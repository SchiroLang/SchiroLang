#!/bin/bash
# Build and run SchiroLang full pipeline
# Usage: ./scripts/run.sh [example|test]
set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$ROOT_DIR"

export LLVM_SYS_181_PREFIX=/tmp/llvm-wrap
export PATH="/tmp/llvm-wrap/bin:$PATH"

case "${1:-example}" in
    example)
        echo "=== SchiroLang Full Pipeline ==="
        cargo run -p schiro-codegen --example compile 2>&1
        echo ""
        echo "=== Running compiled executable ==="
        /tmp/schiro_test_output
        echo "Exit code: $?"
        ;;
    test)
        cargo test 2>&1
        ;;
    clean)
        cargo clean -p llvm-sys -p inkwell -p schiro-codegen 2>&1
        echo "Cleaned"
        ;;
    *)
        echo "Usage: $0 [example|test|clean]"
        ;;
esac
