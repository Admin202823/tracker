#!/bin/bash
# Build script for different optimization levels

set -e

case "$1" in
    "debug")
        echo "Building debug (optimized for development)..."
        cargo build
        ;;
    "debug-fast")
        echo "Building debug with minimal optimizations..."
        RUSTFLAGS="-C opt-level=0 -C debuginfo=1" cargo build
        ;;
    "debug-opt")
        echo "Building debug with basic optimizations..."
        RUSTFLAGS="-C opt-level=1 -C debuginfo=2" cargo build
        ;;
    "release")
        echo "Building release (production optimized)..."
        cargo build --release
        ;;
    "bench")
        echo "Building for benchmarking..."
        RUSTFLAGS="-C opt-level=3 -C debuginfo=1" cargo build --release
        ;;
    "size")
        echo "Building minimal size release..."
        RUSTFLAGS="-C opt-level=s -C panic=abort" cargo build --release
        ;;
    *)
        echo "Usage: $0 {debug|debug-fast|debug-opt|release|bench|size}"
        echo ""
        echo "Build targets:"
        echo "  debug      - Default debug build (optimized for development)"
        echo "  debug-fast - Fastest compile, minimal optimizations"
        echo "  debug-opt  - Debug with basic optimizations (recommended for testing)"
        echo "  release    - Production optimized build"
        echo "  bench      - Benchmarking build (opt-level=3 with debug info)"
        echo "  size       - Minimal binary size"
        exit 1
        ;;
esac

echo "Build complete!"