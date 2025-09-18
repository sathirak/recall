#!/bin/bash
set -e

echo "Uninstalling recall from bash..."

# Remove binary
INSTALL_DIR="$HOME/.local/bin"
if [[ -f "$INSTALL_DIR/recall" ]]; then
    rm "$INSTALL_DIR/recall"
    echo "Removed $INSTALL_DIR/recall"
fi

# Remove bash integration
if [[ -f "$HOME/.bashrc" ]]; then
    sed -i '/# recall command logger integration/,/^$/d' "$HOME/.bashrc"
    sed -i '/recall_log_last_command/d' "$HOME/.bashrc"
    sed -i '/PROMPT_COMMAND.*recall.*log/d' "$HOME/.bashrc"
    echo "Removed bash integration from ~/.bashrc"
fi

echo "Bash uninstallation complete. Restart your terminal or run 'source ~/.bashrc'."
