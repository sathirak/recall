#!/bin/bash

# Agito Uninstallation Script
# This script removes Agito command logger from Linux systems

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

print_status "Starting Agito uninstallation..."

# Remove binary
INSTALL_DIR="$HOME/.local/bin"
if [[ -f "$INSTALL_DIR/agito" ]]; then
    rm "$INSTALL_DIR/agito"
    print_success "Removed binary from $INSTALL_DIR"
else
    print_warning "Binary not found in $INSTALL_DIR"
fi

# Remove shell integrations
print_status "Removing shell integrations..."

# Remove from .bashrc
if [[ -f "$HOME/.bashrc" ]]; then
    if grep -q "Agito command logger integration" "$HOME/.bashrc"; then
        # Create a backup
        cp "$HOME/.bashrc" "$HOME/.bashrc.agito.bak"
        # Remove agito lines
        sed -i '/# Agito command logger integration/,/^$/d' "$HOME/.bashrc"
        print_success "Removed integration from ~/.bashrc (backup saved as ~/.bashrc.agito.bak)"
    fi
fi

# Remove from .zshrc
if [[ -f "$HOME/.zshrc" ]]; then
    if grep -q "Agito command logger integration" "$HOME/.zshrc"; then
        # Create a backup
        cp "$HOME/.zshrc" "$HOME/.zshrc.agito.bak"
        # Remove agito lines
        sed -i '/# Agito command logger integration/,/^$/d' "$HOME/.zshrc"
        print_success "Removed integration from ~/.zshrc (backup saved as ~/.zshrc.agito.bak)"
    fi
fi

# Remove from fish config
FISH_CONFIG="$HOME/.config/fish/config.fish"
if [[ -f "$FISH_CONFIG" ]]; then
    if grep -q "Agito command logger integration" "$FISH_CONFIG"; then
        # Create a backup
        cp "$FISH_CONFIG" "$FISH_CONFIG.agito.bak"
        # Remove agito lines
        sed -i '/# Agito command logger integration/,/^end$/d' "$FISH_CONFIG"
        print_success "Removed integration from fish config (backup saved as config.fish.agito.bak)"
    fi
fi

# Remove systemd user service
SYSTEMD_DIR="$HOME/.config/systemd/user"
if [[ -f "$SYSTEMD_DIR/agito.service" ]]; then
    systemctl --user stop agito.service 2>/dev/null || true
    systemctl --user disable agito.service 2>/dev/null || true
    rm "$SYSTEMD_DIR/agito.service"
    systemctl --user daemon-reload
    print_success "Removed systemd user service"
fi

print_success "Agito uninstallation completed!"
print_status "Shell configuration backups are available with .agito.bak extension"
print_warning "Please restart your terminal or source your shell configuration files"
