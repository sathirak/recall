# 📦 Recall

## Why

I usually need to search up older commands in the correct order they were executed. Using the classic `history` command this is tedious and we cant group by session or search through the commands easily. In certain automation tasks it really helps to have `recall` inbuilt.

Recall is a modern, extensible alternative to the classic `history` command. It provides advanced command history management, session tracking, and a terminal user interface for searching and exploring your shell command history.

## Features

- **Command Logging**: Automatically logs every command you run in your shell (supports Bash, Zsh, and Fish).
- **Session Tracking**: Groups commands by terminal session for better context.
- **TUI Viewer**: Browse, search, and filter your command history in a terminal user interface.
- **Shell Integration**: Easy setup for Bash, Zsh, and Fish shells.
- **Cross-platform**: Works on Linux and other Unix-like systems.

### View, Search all command history
<img width="1907" height="701" alt="image" src="https://github.com/user-attachments/assets/79783323-c7bb-4e20-9ffd-cb739cfe99bf" />

### View commands of a session
<img width="1907" height="701" alt="image" src="https://github.com/user-attachments/assets/5fde58a5-46e3-4a00-a1b3-af5557a53019" />


## Getting Started

### Installation

Currently, recall can only be build from source. Make sure you have Rust and Cargo installed.

```shell
git clone https://github.com/sathirak/recall.git
cd recall

# Install recall for bash
bash ./scripts/install-bash.sh

# Install recall for zsh
bash ./scripts/install-zsh.sh

# Install recall for fish
bash ./scripts/install-fish.sh

```
