#!/usr/bin/env bash
# =============================================================================
# Pulsar Contracts — Developer Setup Script
# =============================================================================
# Installs all prerequisites needed to build, test, and deploy the
# payment-processing smart contract on Soroban / Stellar.
#
# Idempotent: safe to run multiple times. Each step checks whether the
# tool is already present before installing.
#
# Supported platforms: Ubuntu 20.04+, macOS 12+
# =============================================================================

set -euo pipefail

# ── Colours ──────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Colour

info()    { echo -e "${CYAN}[INFO]${NC}  $*"; }
success() { echo -e "${GREEN}[OK]${NC}    $*"; }
warn()    { echo -e "${YELLOW}[WARN]${NC}  $*"; }
error()   { echo -e "${RED}[ERROR]${NC} $*" >&2; exit 1; }

# ── Platform detection ────────────────────────────────────────────────────────
OS="$(uname -s)"
case "$OS" in
  Linux*)  PLATFORM="linux" ;;
  Darwin*) PLATFORM="macos" ;;
  *)       error "Unsupported platform: $OS. This script supports Linux and macOS." ;;
esac
info "Detected platform: $PLATFORM"

# ── 1. Rust ───────────────────────────────────────────────────────────────────
install_rust() {
  if command -v rustc &>/dev/null; then
    RUST_VER="$(rustc --version)"
    success "Rust already installed: $RUST_VER"
  else
    info "Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
    # shellcheck source=/dev/null
    source "$HOME/.cargo/env"
    success "Rust installed: $(rustc --version)"
  fi

  # Ensure cargo is on PATH for the rest of this script
  export PATH="$HOME/.cargo/bin:$PATH"
}

# ── 2. WASM target ────────────────────────────────────────────────────────────
install_wasm_target() {
  if rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
    success "WASM target wasm32-unknown-unknown already installed"
  else
    info "Adding wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
    success "WASM target added"
  fi
}

# ── 3. Stellar CLI ────────────────────────────────────────────────────────────
install_stellar_cli() {
  if command -v stellar &>/dev/null; then
    STELLAR_VER="$(stellar --version 2>&1 | head -1)"
    success "Stellar CLI already installed: $STELLAR_VER"
    return
  fi

  info "Installing Stellar CLI..."

  if [ "$PLATFORM" = "linux" ]; then
    # Install via cargo (works on all Linux distros without root)
    info "Building Stellar CLI from crates.io (this may take a few minutes)..."
    cargo install --locked stellar-cli --features opt
  elif [ "$PLATFORM" = "macos" ]; then
    if command -v brew &>/dev/null; then
      brew install stellar-cli
    else
      info "Homebrew not found — building Stellar CLI from crates.io..."
      cargo install --locked stellar-cli --features opt
    fi
  fi

  success "Stellar CLI installed: $(stellar --version 2>&1 | head -1)"
}

# ── 4. Docker (optional — needed for local network only) ─────────────────────
check_docker() {
  if command -v docker &>/dev/null; then
    DOCKER_VER="$(docker --version)"
    success "Docker already installed: $DOCKER_VER"
  else
    warn "Docker not found. Docker is only required for running a local Stellar network."
    warn "Install Docker Desktop from https://www.docker.com/products/docker-desktop"
  fi
}

# ── 5. Verify minimum Rust version ───────────────────────────────────────────
check_rust_version() {
  REQUIRED="1.79.0"
  CURRENT="$(rustc --version | awk '{print $2}')"

  # Simple semver comparison using sort -V
  LOWEST="$(printf '%s\n%s' "$REQUIRED" "$CURRENT" | sort -V | head -1)"
  if [ "$LOWEST" = "$REQUIRED" ]; then
    success "Rust version $CURRENT meets minimum requirement ($REQUIRED)"
  else
    warn "Rust version $CURRENT is below the minimum required version $REQUIRED."
    info "Updating Rust toolchain..."
    rustup update stable
    success "Rust updated: $(rustc --version)"
  fi
}

# ── Main ──────────────────────────────────────────────────────────────────────
main() {
  echo ""
  echo "=============================================="
  echo "  Pulsar Contracts — Developer Setup"
  echo "=============================================="
  echo ""

  install_rust
  install_wasm_target
  check_rust_version
  install_stellar_cli
  check_docker

  echo ""
  echo "=============================================="
  success "Setup complete! Verify your environment:"
  echo "=============================================="
  echo ""
  echo "  rustc   --version  →  $(rustc --version 2>/dev/null || echo 'not found')"
  echo "  cargo   --version  →  $(cargo --version 2>/dev/null || echo 'not found')"
  echo "  stellar --version  →  $(stellar --version 2>&1 | head -1 || echo 'not found')"
  echo "  docker  --version  →  $(docker --version 2>/dev/null || echo 'not found (optional)')"
  echo ""
  info "Run 'cargo test' inside contracts/payment-processing-contract to confirm everything works."
  echo ""
}

main "$@"
