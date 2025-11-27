#!/usr/bin/env bash
# Bulldog developer bootstrap script
# Verifies VS Code + WSL extension, ensures Rust nightly, and sets aliases (with -Z bindeps)

set -euo pipefail

RED="\033[0;31m"
YELLOW="\033[0;33m"
GREEN="\033[0;32m"
NC="\033[0m"

echo -e "${GREEN}==> Bulldog developer bootstrap starting...${NC}"

# 1) Check VS Code CLI
if ! command -v code >/dev/null 2>&1; then
  echo -e "${RED}✖ VS Code CLI ('code') not found.${NC}"
  echo -e "${YELLOW}• Install VS Code and enable 'code' in PATH.${NC}"
  echo -e "${YELLOW}• In VS Code: Command Palette → 'Shell Command: Install 'code' command in PATH' (macOS), or install VS Code Server for WSL (Windows).${NC}"
  exit 1
fi
echo -e "${GREEN}✔ VS Code CLI found.${NC}"

# 2) Check WSL extension in VS Code
if ! code --list-extensions | grep -qi '^ms-vscode-remote.remote-wsl$'; then
  echo -e "${RED}✖ VS Code WSL extension not found (ms-vscode-remote.remote-wsl).${NC}"

  echo -e "${YELLOW}• Installing WSL extension...${NC}"
  code --install-extension ms-vscode-remote.remote-wsl || {
    echo -e "${RED}Failed to install WSL extension automatically.${NC}"
    echo -e "${YELLOW}Try installing manually from the Extensions panel: 'Remote - WSL'.${NC}"
    exit 1
  }
fi
echo -e "${GREEN}✔ WSL extension present.${NC}"

# 2b) Check and install essential VS Code extensions
install_extension () {
  local ext="$1"
  if ! code --list-extensions | grep -qi "^${ext}$"; then
    echo -e "${YELLOW}• Installing VS Code extension: ${ext}${NC}"
    code --install-extension "${ext}" || {
      echo -e "${RED}Failed to install ${ext}.${NC}"
    }
  else
    echo -e "${GREEN}✔ Extension ${ext} already installed.${NC}"
  fi
}

install_extension rust-lang.rust-analyzer
install_extension GitHub.copilot
install_extension GitHub.copilot-chat
install_extension tamasfe.even-better-toml
install_extension fill-labs.dependi


# 3) Check Rustup and nightly toolchain
if ! command -v rustup >/dev/null 2>&1; then
  echo -e "${RED}✖ rustup not found.${NC}"
  echo -e "${YELLOW}• Install Rustup: https://rustup.rs${NC}"
  exit 1
fi
echo -e "${GREEN}✔ rustup found.${NC}"

if ! rustup toolchain list | grep -q '^nightly'; then
  echo -e "${YELLOW}• Installing Rust nightly toolchain...${NC}"
  rustup install nightly
fi
echo -e "${GREEN}✔ Rust nightly available.${NC}"

# 4) Ensure cargo/bin on PATH and source cargo env
if [ -f "$HOME/.cargo/env" ]; then
  # Only source if not already in current shell
  if ! echo "$PATH" | grep -q "$HOME/.cargo/bin"; then
    . "$HOME/.cargo/env"
    echo -e "${GREEN}✔ Sourced ~/.cargo/env.${NC}"
  else
    echo -e "${GREEN}✔ ~/.cargo/bin already on PATH.${NC}"
  fi
else
  echo -e "${YELLOW}• ~/.cargo/env not found. Rust tools may not be on PATH for new shells.${NC}"
fi

# 5) Write aliases to ~/.bash_aliases
ALIASES_FILE="$HOME/.bash_aliases"
touch "$ALIASES_FILE"

add_alias () {
  local name="$1"; shift
  local body="$*"
  if grep -q "^alias ${name}=" "$ALIASES_FILE"; then
    echo -e "${YELLOW}• Alias '${name}' already present; leaving as-is.${NC}"
  else
    echo "alias ${name}='${body}'" >> "$ALIASES_FILE"
    echo -e "${GREEN}✔ Added alias '${name}'.${NC}"
  fi
}

# VS Code launcher (reuse window)
add_alias bulldog "cd ~/bulldog && code . --reuse-window"

# Cargo build/run with nightly and -Z bindeps
add_alias bulldog-build "cd ~/bulldog && cargo +nightly build -Z bindeps"
add_alias bulldog-run "cd ~/bulldog && cargo +nightly run -Z bindeps"

# Optional: quick clean
add_alias bulldog-clean "cd ~/bulldog && cargo +nightly clean"

# 6) Reload aliases for current shell
. "$ALIASES_FILE"

echo -e "${GREEN}==> Bootstrap complete.${NC}"
echo "Use:"
echo "  - bulldog        → open Bulldog in VS Code (WSL)"
echo "  - bulldog-build  → cargo +nightly build -Z bindeps"
echo "  - bulldog-run    → cargo +nightly run -Z bindeps"
echo "  - bulldog-clean  → cargo +nightly clean"
# Reload .bashrc so aliases are immediately available
source ~/.bashrc
echo -e "${GREEN}✔ Aliases reloaded and ready to use: bulldog, bulldog-build, bulldog-run, bulldog-clean${NC}"

