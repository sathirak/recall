#!/bin/bash

# recall Installation Script
# This script builds and installs the recall command logger for Linux systems

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running as root
if [[ $EUID -eq 0 ]]; then
    print_error "This script should not be run as root. Run as a regular user."
    exit 1
fi

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    print_error "Rust is not installed. Please install Rust first:"
    echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

print_status "Starting recall installation..."

# Build the project
print_status "Building recall..."
if ! cargo build --release; then
    print_error "Failed to build recall"
    exit 1
fi

print_success "Build completed successfully"

# Create installation directory
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

# Copy binary to installation directory
print_status "Installing binary to $INSTALL_DIR..."
cp target/release/recall "$INSTALL_DIR/"

# Make sure the binary is executable
chmod +x "$INSTALL_DIR/recall"

print_success "Binary installed to $INSTALL_DIR/recall"

# Check if ~/.local/bin is in PATH
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    print_warning "~/.local/bin is not in your PATH"
    print_status "Adding ~/.local/bin to PATH in shell configuration files..."
    
    # Add to .bashrc if it exists
    if [[ -f "$HOME/.bashrc" ]]; then
        if ! grep -q "export PATH=.*\.local/bin" "$HOME/.bashrc"; then
            echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
            print_status "Added to ~/.bashrc"
        fi
    fi
    
    # Add to .zshrc if it exists
    if [[ -f "$HOME/.zshrc" ]]; then
        if ! grep -q "export PATH=.*\.local/bin" "$HOME/.zshrc"; then
            echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.zshrc"
            print_status "Added to ~/.zshrc"
        fi
    fi
    
    # Add to fish config if it exists
    if [[ -d "$HOME/.config/fish" ]]; then
        FISH_CONFIG="$HOME/.config/fish/config.fish"
        if ! grep -q "set -gx PATH.*\.local/bin" "$FISH_CONFIG" 2>/dev/null; then
            mkdir -p "$HOME/.config/fish"
            echo 'set -gx PATH $HOME/.local/bin $PATH' >> "$FISH_CONFIG"
            print_status "Added to fish config"
        fi
    fi
    
    print_warning "Please restart your terminal or run 'source ~/.bashrc' (or equivalent for your shell)"
fi

# Detect current shell and offer to install integration
CURRENT_SHELL=$(basename "$SHELL")
print_status "Detected shell: $CURRENT_SHELL"

read -p "Would you like to install shell integration for automatic command logging? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    print_status "Installing shell integration for $CURRENT_SHELL..."
    if "$INSTALL_DIR/recall" install --shell "$CURRENT_SHELL"; then
        print_success "Shell integration installed successfully"
        print_warning "Please restart your terminal or source your shell configuration file"
    else
        print_error "Failed to install shell integration"
    fi
else
    print_status "Skipping shell integration. You can install it later with:"
    echo "recall install --shell $CURRENT_SHELL"
fi


print_success "recall installation completed!"
print_status "Available commands:"
echo "  recall log <command>     - Log a command manually"
echo "  recall history           - Show recent command history"
echo "  recall history -l 50     - Show last 50 commands"
echo "  recall clear             - Clear command history"
echo "  recall install --shell <shell> - Install shell integration"
echo ""

if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    print_warning "Remember to restart your terminal or run: source ~/.bashrc"
fi
