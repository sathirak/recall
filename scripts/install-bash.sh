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
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
    echo "Added ~/.local/bin to PATH in .bashrc"
fi

# Add bash integration
if ! grep -q "recall command logger integration" "$HOME/.bashrc" 2>/dev/null; then
    cat >> "$HOME/.bashrc" << 'EOF'

# recall command logger integration
recall_log_last_command() {
    local last_cmd=$(history 1 | sed 's/^[ ]*[0-9]*[ ]*//')
    if [[ "$last_cmd" != recall* ]] && [[ -n "$last_cmd" ]]; then
        ~/.local/bin/recall log "$last_cmd" 2>/dev/null
    fi
}
export PROMPT_COMMAND="${PROMPT_COMMAND:+$PROMPT_COMMAND$'\n'}history -a; recall_log_last_command"
EOF
    echo "Added bash integration to ~/.bashrc"
fi

echo "Bash installation complete. Run 'source ~/.bashrc' or restart your terminal."
