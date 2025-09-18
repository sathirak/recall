#!/usr/bin/env fish

# Do not run as root
if test (id -u) -eq 0
    echo "Do not run this script as root."
    exit 1
end

# Check for Rust
if not command -v cargo > /dev/null
    echo "Rust is not installed. Please install Rust:"
    echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
end

echo "Building recall..."
cargo build --release

set INSTALL_DIR "$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"
cp target/release/recall "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/recall"
echo "Installed recall to $INSTALL_DIR/recall"

# Add to PATH if needed
mkdir -p "$HOME/.config/fish"
if not contains "$HOME/.local/bin" $PATH
    echo 'set -gx PATH $HOME/.local/bin $PATH' >> "$HOME/.config/fish/config.fish"
    echo "Added ~/.local/bin to PATH in fish config"
end

# Add fish integration
if not grep -q "recall command logger integration" "$HOME/.config/fish/config.fish" 2>/dev/null
    echo '
# recall command logger integration
function recall_log_command --on-event fish_preexec
    if not string match -q "recall*" -- "$argv"
        ~/.local/bin/recall log "$argv" 2>/dev/null &
    end
end' >> "$HOME/.config/fish/config.fish"
    echo "Added fish integration to ~/.config/fish/config.fish"
end

echo "Fish installation complete. Restart your terminal or run 'source ~/.config/fish/config.fish'."
