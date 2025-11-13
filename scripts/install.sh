#!/usr/bin/env bash
# RipTide Installation Script
# Detects OS/architecture and downloads the appropriate binary

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO="your-org/riptidecrawler"  # Update with actual repo
INSTALL_DIR="${INSTALL_DIR:-$HOME/.riptide}"
BIN_DIR="${BIN_DIR:-$HOME/.local/bin}"
VERSION="${VERSION:-latest}"
VARIANT="${VARIANT:-native}"  # native or wasm

# Helper functions
info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

success() {
    echo -e "${GREEN}✓${NC} $1"
}

warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1"
    exit 1
}

# Detect OS and architecture
detect_platform() {
    local os arch

    # Detect OS
    case "$(uname -s)" in
        Linux*)     os="linux" ;;
        Darwin*)    os="macos" ;;
        MINGW*|MSYS*|CYGWIN*) os="windows" ;;
        *)          error "Unsupported operating system: $(uname -s)" ;;
    esac

    # Detect architecture
    case "$(uname -m)" in
        x86_64|amd64)   arch="x64" ;;
        aarch64|arm64)  arch="arm64" ;;
        *)              error "Unsupported architecture: $(uname -m)" ;;
    esac

    echo "${os}-${arch}"
}

# Get latest version from GitHub
get_latest_version() {
    if command -v curl &> /dev/null; then
        curl -sSL "https://api.github.com/repos/${REPO}/releases/latest" | \
            grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
    elif command -v wget &> /dev/null; then
        wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" | \
            grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
    else
        error "Neither curl nor wget is available. Please install one of them."
    fi
}

# Download and extract binary
download_binary() {
    local platform=$1
    local version=$2
    local variant=$3

    # Construct asset name
    local asset_name="riptide-${platform}-${variant}"
    local archive_ext=".tar.gz"

    if [[ "$platform" == *"windows"* ]]; then
        archive_ext=".zip"
    fi

    local download_url="https://github.com/${REPO}/releases/download/${version}/${asset_name}${archive_ext}"

    info "Downloading RipTide ${version} (${variant}) for ${platform}..."
    info "URL: ${download_url}"

    # Create temporary directory
    local tmp_dir=$(mktemp -d)
    cd "$tmp_dir"

    # Download with progress
    if command -v curl &> /dev/null; then
        curl -L --progress-bar -o "riptide${archive_ext}" "$download_url" || \
            error "Download failed. Please check your internet connection and try again."
    elif command -v wget &> /dev/null; then
        wget --show-progress -O "riptide${archive_ext}" "$download_url" || \
            error "Download failed. Please check your internet connection and try again."
    fi

    # Verify checksum if available
    if command -v curl &> /dev/null; then
        if curl -sSL "${download_url}.sha256" -o "checksum.sha256" 2>/dev/null; then
            info "Verifying checksum..."
            if command -v sha256sum &> /dev/null; then
                sha256sum -c checksum.sha256 || error "Checksum verification failed!"
            elif command -v shasum &> /dev/null; then
                shasum -a 256 -c checksum.sha256 || error "Checksum verification failed!"
            fi
            success "Checksum verified"
        fi
    fi

    # Extract archive
    info "Extracting archive..."
    if [[ "$archive_ext" == ".zip" ]]; then
        unzip -q "riptide${archive_ext}" || error "Extraction failed"
    else
        tar xzf "riptide${archive_ext}" || error "Extraction failed"
    fi

    # Install binaries
    info "Installing to ${INSTALL_DIR}..."
    mkdir -p "$INSTALL_DIR/bin"
    mkdir -p "$INSTALL_DIR/config"

    # Move files
    cp -r "${asset_name}"/* "$INSTALL_DIR/"

    # Make binaries executable
    chmod +x "$INSTALL_DIR"/riptide-*

    # Create symlinks in bin directory
    mkdir -p "$BIN_DIR"
    ln -sf "$INSTALL_DIR/riptide-api" "$BIN_DIR/riptide-api"
    ln -sf "$INSTALL_DIR/riptide-cli" "$BIN_DIR/riptide"
    ln -sf "$INSTALL_DIR/riptide-headless" "$BIN_DIR/riptide-headless"
    ln -sf "$INSTALL_DIR/riptide-workers" "$BIN_DIR/riptide-workers"

    # Cleanup
    cd - > /dev/null
    rm -rf "$tmp_dir"

    success "Installation complete!"
}

# Setup configuration
setup_config() {
    info "Setting up configuration..."

    # Copy .env.example to .env if it doesn't exist
    if [[ ! -f "$INSTALL_DIR/.env" ]] && [[ -f "$INSTALL_DIR/.env.example" ]]; then
        cp "$INSTALL_DIR/.env.example" "$INSTALL_DIR/.env"
        success "Created .env from template"
        warning "Please edit $INSTALL_DIR/.env to configure your installation"
    fi

    # Create data directories
    mkdir -p "$INSTALL_DIR/data"
    mkdir -p "$INSTALL_DIR/logs"
    mkdir -p "$INSTALL_DIR/cache"

    success "Configuration setup complete"
}

# Add to PATH
setup_path() {
    local shell_rc=""

    # Detect shell
    if [[ -n "$BASH_VERSION" ]]; then
        shell_rc="$HOME/.bashrc"
    elif [[ -n "$ZSH_VERSION" ]]; then
        shell_rc="$HOME/.zshrc"
    else
        shell_rc="$HOME/.profile"
    fi

    # Check if already in PATH
    if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
        info "Adding $BIN_DIR to PATH in $shell_rc..."
        echo "" >> "$shell_rc"
        echo "# RipTide" >> "$shell_rc"
        echo "export PATH=\"\$PATH:$BIN_DIR\"" >> "$shell_rc"
        success "Added to PATH. Run 'source $shell_rc' or restart your terminal."
    else
        success "$BIN_DIR already in PATH"
    fi
}

# Print usage information
print_usage() {
    cat << EOF
RipTide Installation Script

Usage: $0 [OPTIONS]

Options:
    -h, --help          Show this help message
    -v, --version VER   Install specific version (default: latest)
    -w, --wasm          Install WASM variant (default: native)
    -d, --dir DIR       Installation directory (default: ~/.riptide)
    -b, --bin DIR       Binary directory (default: ~/.local/bin)
    --no-path           Don't modify shell PATH
    --uninstall         Uninstall RipTide

Environment Variables:
    INSTALL_DIR         Installation directory
    BIN_DIR             Binary directory
    VERSION             Version to install
    VARIANT             Variant to install (native or wasm)

Examples:
    # Install latest native variant
    $0

    # Install specific version with WASM
    $0 --version v0.9.0 --wasm

    # Custom installation directory
    $0 --dir /opt/riptide --bin /usr/local/bin

EOF
}

# Uninstall RipTide
uninstall() {
    warning "Uninstalling RipTide..."

    # Remove binaries
    rm -f "$BIN_DIR/riptide"*

    # Remove installation directory
    if [[ -d "$INSTALL_DIR" ]]; then
        read -p "Remove installation directory $INSTALL_DIR? [y/N] " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -rf "$INSTALL_DIR"
            success "Removed $INSTALL_DIR"
        fi
    fi

    success "Uninstall complete"
    info "You may want to remove the PATH entry from your shell configuration"
    exit 0
}

# Main installation flow
main() {
    local modify_path=true

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                print_usage
                exit 0
                ;;
            -v|--version)
                VERSION="$2"
                shift 2
                ;;
            -w|--wasm)
                VARIANT="wasm"
                shift
                ;;
            -d|--dir)
                INSTALL_DIR="$2"
                shift 2
                ;;
            -b|--bin)
                BIN_DIR="$2"
                shift 2
                ;;
            --no-path)
                modify_path=false
                shift
                ;;
            --uninstall)
                uninstall
                ;;
            *)
                error "Unknown option: $1. Use --help for usage information."
                ;;
        esac
    done

    echo -e "${BLUE}"
    cat << "EOF"
    ____  _      _______ _     _
   |  _ \(_)_ __|_   _(_) | __| | ___
   | |_) | | '_ \ | | | | |/ _` |/ _ \
   |  _ <| | |_) || | | | | (_| |  __/
   |_| \_\_| .__/ |_| |_|_|\__,_|\___|
           |_|
EOF
    echo -e "${NC}"

    info "RipTide Installer"
    echo

    # Detect platform
    local platform=$(detect_platform)
    info "Detected platform: $platform"

    # Get version
    if [[ "$VERSION" == "latest" ]]; then
        VERSION=$(get_latest_version)
        if [[ -z "$VERSION" ]]; then
            error "Failed to determine latest version"
        fi
    fi
    info "Installing version: $VERSION"
    info "Variant: $VARIANT"
    echo

    # Download and install
    download_binary "$platform" "$VERSION" "$VARIANT"

    # Setup
    setup_config

    if [[ "$modify_path" == true ]]; then
        setup_path
    fi

    echo
    success "RipTide installed successfully!"
    echo
    info "Installation location: $INSTALL_DIR"
    info "Binaries available at: $BIN_DIR"
    echo
    info "Quick start:"
    echo "  1. Configure: edit $INSTALL_DIR/.env"
    echo "  2. Start API:  riptide-api"
    echo "  3. Or use CLI: riptide --help"
    echo
    info "Documentation: https://github.com/${REPO}"

    # Version check
    if command -v riptide &> /dev/null; then
        echo
        riptide --version 2>/dev/null || true
    fi
}

# Run main function
main "$@"
