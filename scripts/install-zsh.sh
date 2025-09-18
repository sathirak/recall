#!/bin/bash
set -e

# Do not run as root
if [[ $EUID -eq 0 ]]; then
    echo "Do not run this script as root."
    exit 1
fi

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo "Rust is not installed. Please install Rust:"
    echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "Building recall..."
cargo build --release

INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"
cp target/release/recall "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/recall"
echo "Installed recall to $INSTALL_DIR/recall"

# Add to PATH if needed
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    if [[ -f "$HOME/.zshrc" ]]; then
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.zshrc"
        echo "Added ~/.local/bin to PATH in .zshrc"
    fi
fi

# Add zsh integration
if [[ -f "$HOME/.zshrc" ]]; then
    if ! grep -q "recall command logger integration" "$HOME/.zshrc" 2>/dev/null; then
        cat >> "$HOME/.zshrc" << 'EOF'

# recall command logger integration
preexec() {
    [[ "$1" != recall* ]] && ~/.local/bin/recall log "$1" 2>/dev/null
}
EOF
        echo "Added zsh integration to ~/.zshrc"
    fi
else
    echo "~/.zshrc not found. Please create it first or switch to zsh."
    exit 1
fi

echo "Zsh installation complete. Run 'source ~/.zshrc' or restart your terminal."
