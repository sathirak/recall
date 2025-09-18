#!/usr/bin/env fish

echo "Uninstalling recall from fish..."

# Remove binary
set INSTALL_DIR "$HOME/.local/bin"
if test -f "$INSTALL_DIR/recall"
    rm "$INSTALL_DIR/recall"
    echo "Removed $INSTALL_DIR/recall"
end

# Remove fish integration
if test -f "$HOME/.config/fish/config.fish"
    sed -i '/# recall command logger integration/,/^end$/d' "$HOME/.config/fish/config.fish"
    sed -i '/recall_log_command.*fish_preexec/d' "$HOME/.config/fish/config.fish"
    echo "Removed fish integration from ~/.config/fish/config.fish"
end

echo "Fish uninstallation complete. Restart your terminal."
