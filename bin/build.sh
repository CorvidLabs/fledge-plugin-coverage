#!/usr/bin/env bash
set -e

if command -v cargo &>/dev/null; then
    echo "  Building fledge-plugin-coverage (Rust)..."
    cargo build --release --quiet
    cp target/release/fledge-plugin-coverage bin/fledge-coverage
    echo "  Build complete."
else
    echo "  Cargo not found — using pre-built binary if present."
fi
