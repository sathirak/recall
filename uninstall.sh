#!/bin/bash

# recall Uninstallation Script
# This script removes recall command logger from Linux systems

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

print_status "Starting recall uninstallation..."

# Remove binary
INSTALL_DIR="$HOME/.local/bin"
if [[ -f "$INSTALL_DIR/recall" ]]; then
    rm "$INSTALL_DIR/recall"
    print_success "Removed binary from $INSTALL_DIR"
else
    print_warning "Binary not found in $INSTALL_DIR"
fi

# Remove shell integrations
print_status "Removing shell integrations..."

# Remove from .bashrc
if [[ -f "$HOME/.bashrc" ]]; then
    if grep -q "recall command logger integration" "$HOME/.bashrc"; then
        # Create a backup
        cp "$HOME/.bashrc" "$HOME/.bashrc.recall.bak"
        # Remove recall lines
        sed -i '/# recall command logger integration/,/^$/d' "$HOME/.bashrc"
        print_success "Removed integration from ~/.bashrc (backup saved as ~/.bashrc.recall.bak)"
    fi
fi

# Remove from .zshrc
if [[ -f "$HOME/.zshrc" ]]; then
    if grep -q "recall command logger integration" "$HOME/.zshrc"; then
        # Create a backup
        cp "$HOME/.zshrc" "$HOME/.zshrc.recall.bak"
        # Remove recall lines
        sed -i '/# recall command logger integration/,/^$/d' "$HOME/.zshrc"
        print_success "Removed integration from ~/.zshrc (backup saved as ~/.zshrc.recall.bak)"
    fi
fi

# Remove from fish config
FISH_CONFIG="$HOME/.config/fish/config.fish"
if [[ -f "$FISH_CONFIG" ]]; then
    if grep -q "recall command logger integration" "$FISH_CONFIG"; then
        # Create a backup
        cp "$FISH_CONFIG" "$FISH_CONFIG.recall.bak"
        # Remove recall lines
        sed -i '/# recall command logger integration/,/^end$/d' "$FISH_CONFIG"
        print_success "Removed integration from fish config (backup saved as config.fish.recall.bak)"
    fi
fi

# Remove systemd user service
SYSTEMD_DIR="$HOME/.config/systemd/user"
if [[ -f "$SYSTEMD_DIR/recall.service" ]]; then
    systemctl --user stop recall.service 2>/dev/null || true
    systemctl --user disable recall.service 2>/dev/null || true
    rm "$SYSTEMD_DIR/recall.service"
    systemctl --user daemon-reload
    print_success "Removed systemd user service"
fi

print_success "recall uninstallation completed!"
print_status "Shell configuration backups are available with .recall.bak extension"
print_warning "Please restart your terminal or source your shell configuration files"
