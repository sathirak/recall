#!/bin/bash
set -e

echo "Uninstalling recall from zsh..."

# Remove binary
INSTALL_DIR="$HOME/.local/bin"
if [[ -f "$INSTALL_DIR/recall" ]]; then
    rm "$INSTALL_DIR/recall"
    echo "Removed $INSTALL_DIR/recall"
fi

# Remove zsh integration
if [[ -f "$HOME/.zshrc" ]]; then
    sed -i '/# recall command logger integration/,/^$/d' "$HOME/.zshrc"
    sed -i '/preexec.*recall.*log/d' "$HOME/.zshrc"
    echo "Removed zsh integration from ~/.zshrc"
fi

echo "Zsh uninstallation complete. Restart your terminal or run 'source ~/.zshrc'."
