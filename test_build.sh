#!/bin/bash

# Simple build test script
echo "Testing RAX FTP Client compilation..."

cd "$(dirname "$0")"

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "Cargo not found. Please ensure Rust is installed."
    exit 1
fi

# Clean previous builds
echo "Cleaning previous builds..."
cargo clean

# Run cargo check to verify syntax
echo "Running cargo check..."
if cargo check; then
    echo "✅ Code compiles successfully!"
else
    echo "❌ Compilation failed!"
    exit 1
fi

# Run tests
echo "Running tests..."
if cargo test test_parse_pasv_response; then
    echo "✅ PASV parsing tests passed!"
else
    echo "❌ PASV parsing tests failed!"
    exit 1
fi

echo "✅ All checks passed! The LIST command fix should work correctly."
