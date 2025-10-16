#!/bin/bash
# install-pdfium.sh
# Script to install Pdfium library for PDF processing support

set -e

# Configuration
PDFIUM_VERSION="chromium/7469"
INSTALL_DIR="/usr/local/lib"
TEMP_DIR="/tmp/pdfium-install"

# Detect platform
ARCH=$(uname -m)
OS=$(uname -s)

case "${OS}" in
    Linux)
        if [ "${ARCH}" = "x86_64" ]; then
            PLATFORM="linux-x64"
            LIB_NAME="libpdfium.so"
        elif [ "${ARCH}" = "aarch64" ]; then
            PLATFORM="linux-arm64"
            LIB_NAME="libpdfium.so"
        else
            echo "Unsupported architecture: ${ARCH}"
            exit 1
        fi
        ;;
    Darwin)
        if [ "${ARCH}" = "x86_64" ]; then
            PLATFORM="darwin-x64"
            LIB_NAME="libpdfium.dylib"
        elif [ "${ARCH}" = "arm64" ]; then
            PLATFORM="darwin-arm64"
            LIB_NAME="libpdfium.dylib"
        else
            echo "Unsupported architecture: ${ARCH}"
            exit 1
        fi
        ;;
    *)
        echo "Unsupported operating system: ${OS}"
        exit 1
        ;;
esac

echo "======================================"
echo "Pdfium Library Installation Script"
echo "======================================"
echo "Platform: ${PLATFORM}"
echo "Version: ${PDFIUM_VERSION}"
echo "Install Directory: ${INSTALL_DIR}"
echo ""

# Check if already installed
if ldconfig -p 2>/dev/null | grep -q pdfium || [ -f "${INSTALL_DIR}/${LIB_NAME}" ]; then
    echo "⚠️  Pdfium library appears to be already installed."
    read -p "Do you want to reinstall? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Installation cancelled."
        exit 0
    fi
fi

# Create temporary directory
echo "Creating temporary directory..."
mkdir -p "${TEMP_DIR}"
cd "${TEMP_DIR}"

# Download
echo "Downloading pdfium-${PLATFORM}.tgz..."
DOWNLOAD_URL="https://github.com/bblanchon/pdfium-binaries/releases/download/${PDFIUM_VERSION}/pdfium-${PLATFORM}.tgz"
echo "URL: ${DOWNLOAD_URL}"

if ! curl -L -f "${DOWNLOAD_URL}" -o pdfium.tgz; then
    echo "❌ Download failed. Please check your internet connection and try again."
    exit 1
fi

# Extract
echo "Extracting archive..."
tar -xzf pdfium.tgz

# Verify library file exists
if [ ! -f "lib/${LIB_NAME}" ]; then
    echo "❌ Library file not found in archive: lib/${LIB_NAME}"
    exit 1
fi

# Install
echo "Installing to ${INSTALL_DIR}..."
if [ ! -w "${INSTALL_DIR}" ]; then
    echo "Installing with sudo (requires root permissions)..."
    sudo mkdir -p "${INSTALL_DIR}"
    sudo cp "lib/${LIB_NAME}" "${INSTALL_DIR}/"
    sudo chmod 755 "${INSTALL_DIR}/${LIB_NAME}"
else
    mkdir -p "${INSTALL_DIR}"
    cp "lib/${LIB_NAME}" "${INSTALL_DIR}/"
    chmod 755 "${INSTALL_DIR}/${LIB_NAME}"
fi

# Update library cache (Linux only)
if [ "${OS}" = "Linux" ]; then
    echo "Updating library cache..."
    if command -v ldconfig &> /dev/null; then
        sudo ldconfig
    fi
fi

# Verify installation
echo ""
echo "Verifying installation..."
if [ -f "${INSTALL_DIR}/${LIB_NAME}" ]; then
    echo "✓ Library file installed: ${INSTALL_DIR}/${LIB_NAME}"
    ls -lh "${INSTALL_DIR}/${LIB_NAME}"
else
    echo "❌ Installation verification failed - library not found"
    exit 1
fi

# Check library cache (Linux only)
if [ "${OS}" = "Linux" ] && command -v ldconfig &> /dev/null; then
    if ldconfig -p | grep -q pdfium; then
        echo "✓ Library registered in system cache:"
        ldconfig -p | grep pdfium
    else
        echo "⚠️  Library not found in ldconfig cache"
        echo "   You may need to add ${INSTALL_DIR} to /etc/ld.so.conf.d/"
    fi
fi

# Cleanup
echo ""
echo "Cleaning up temporary files..."
cd /
rm -rf "${TEMP_DIR}"

# Success message
echo ""
echo "======================================"
echo "✓ Installation Complete!"
echo "======================================"
echo ""
echo "The Pdfium library has been installed successfully."
echo ""
echo "Next steps:"
echo "1. For the current session, run:"
if [ "${OS}" = "Linux" ]; then
    echo "   export LD_LIBRARY_PATH=${INSTALL_DIR}:\${LD_LIBRARY_PATH}"
else
    echo "   export DYLD_LIBRARY_PATH=${INSTALL_DIR}:\${DYLD_LIBRARY_PATH}"
fi
echo ""
echo "2. To make it permanent, add the above line to your shell profile:"
echo "   ~/.bashrc, ~/.zshrc, or ~/.profile"
echo ""
echo "3. Test PDF processing:"
echo "   cargo test --package riptide-pdf"
echo ""
