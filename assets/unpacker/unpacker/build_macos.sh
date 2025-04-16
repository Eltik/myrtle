#!/bin/bash
# macOS build script for Ark-Unpacker

# Exit on error
set -e

echo "=== Ark-Unpacker macOS Build Script ==="
echo "This script will help you build Ark-Unpacker on macOS"

# Check if Python is installed
if ! command -v python3 &> /dev/null; then
    echo "Python 3 is not installed. Please install Python 3 first."
    exit 1
fi

# Check Python version
PYTHON_VERSION=$(python3 --version 2>&1 | awk '{print $2}')
echo "Python version: $PYTHON_VERSION"

# Check architecture
ARCH=$(uname -m)
echo "Architecture: $ARCH"

# Check if Poetry is installed
if ! command -v poetry &> /dev/null; then
    echo "Poetry is not installed. Installing Poetry..."
    curl -sSL https://install.python-poetry.org | python3 -
fi

# Install dependencies
echo "Installing dependencies..."
poetry install

# Create necessary directories if they don't exist
mkdir -p src/fbs/CN
mkdir -p build

# Run the build script
echo "Running build script..."
python3 Build.py

# Check if build was successful
if [ -f "./build/dist/ArkUnpacker-v4.0.0" ]; then
    echo "=== Build completed successfully ==="
    echo "The executable is in the build/dist directory"
    echo "You can run it with: ./build/dist/ArkUnpacker-v4.0.0"
else
    echo "=== Build failed ==="
    echo "Please check the error messages above for details."
    echo "Common issues:"
    echo "1. Missing dependencies - try running 'poetry install' again"
    echo "2. Architecture compatibility issues - make sure you're using the correct Python version for your architecture"
    echo "3. Permission issues - try running with sudo if needed"
    exit 1
fi 