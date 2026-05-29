#!/bin/bash
# -----------------------------------------------------------------------------
# LightQOS - Quantum Operating System
# install.sh — Automated installation script for all platforms
#
# Copyright (c) 2021 - 2026 Márcio Coutinho
# Date: 16-05-2022
# All rights reserved.
# -----------------------------------------------------------------------------

set -e  # Exit on error

echo "╔══════════════════════════════════════════════════════════╗"
echo "║            LightQOS Installation Script                  ║"
echo "║                      v0.2.0                              ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Functions
print_step() {
    echo -e "${GREEN}==>${NC} $1"
}

print_error() {
    echo -e "${RED}ERROR:${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}WARNING:${NC} $1"
}

# Check OS
print_step "Detecting operating system..."
OS="$(uname -s)"
case "${OS}" in
    Linux*)     MACHINE=Linux;;
    Darwin*)    MACHINE=Mac;;
    CYGWIN*)    MACHINE=Cygwin;;
    MINGW*)     MACHINE=MinGw;;
    *)          MACHINE="UNKNOWN:${OS}"
esac
echo "   Detected: ${MACHINE}"

# Check Python
print_step "Checking Python installation..."
if ! command -v python3 &> /dev/null; then
    print_error "Python 3 not found. Please install Python 3.10+"
    exit 1
fi

PYTHON_VERSION=$(python3 --version | cut -d' ' -f2)
echo "   Python version: ${PYTHON_VERSION}"

# Check if Python >= 3.10
PYTHON_MAJOR=$(echo $PYTHON_VERSION | cut -d'.' -f1)
PYTHON_MINOR=$(echo $PYTHON_VERSION | cut -d'.' -f2)

if [ "$PYTHON_MAJOR" -lt 3 ] || ([ "$PYTHON_MAJOR" -eq 3 ] && [ "$PYTHON_MINOR" -lt 10 ]); then
    print_error "Python 3.10+ required. Found: ${PYTHON_VERSION}"
    exit 1
fi

# Check Rust
print_step "Checking Rust installation..."
if ! command -v rustc &> /dev/null; then
    print_warning "Rust not found. Installing via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

RUST_VERSION=$(rustc --version | cut -d' ' -f2)
echo "   Rust version: ${RUST_VERSION}"

# Create virtual environment
print_step "Creating Python virtual environment..."
python3 -m venv venv
source venv/bin/activate

# Upgrade pip
print_step "Upgrading pip..."
pip install --upgrade pip

# Install Python dependencies
print_step "Installing Python dependencies..."
pip install -r requirements.txt

# Build Rust components
print_step "Building Rust kernel..."
cd kernel
cargo build --release
cd ..

print_step "Building simulators..."
cd simulators
cargo build --release
cd ..

print_step "Building protocols..."
cd protocols
cargo build --release
cd ..

# Build PyO3 bindings
print_step "Building PyO3 bindings..."
cd pyo3_bindings
maturin develop --release
cd ..

# Install LightQOS package
print_step "Installing LightQOS package..."
pip install -e .

# Run tests
print_step "Running tests..."
pytest tests/ -v

# Success
echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║          ✅ LightQOS installed successfully!             ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""
echo "To activate the environment:"
echo "  source venv/bin/activate"
echo ""
echo "To run examples:"
echo "  python examples/basic_usage.py"
echo ""
echo "To run tests:"
echo "  pytest tests/"
echo ""
echo "Documentation: docs/README.md"
echo ""
