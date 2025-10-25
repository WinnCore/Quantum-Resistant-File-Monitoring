#!/usr/bin/env bash
#
# WinnCore AV Installation Script
# Usage: curl -fsSL https://raw.githubusercontent.com/WinnCore/Quantum-Resistant-File-Monitoring/main/install.sh | sh
#
# This script downloads and installs the latest WinnCore AV release for ARM64 Linux

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
REPO="WinnCore/Quantum-Resistant-File-Monitoring"
PACKAGE_NAME="charmedwoa-av"
VERSION="0.2.0"

# Logging functions
info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Check if running as root
if [ "$EUID" -eq 0 ]; then
    error "Do not run this script as root. It will prompt for sudo when needed."
fi

# Detect OS and architecture
detect_system() {
    info "Detecting system configuration..."

    # Detect OS
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        OS=$ID
        OS_VERSION=$VERSION_ID
    else
        error "Cannot detect OS. /etc/os-release not found."
    fi

    # Detect architecture
    ARCH=$(uname -m)

    info "Detected: $OS $OS_VERSION on $ARCH"

    # Validate ARM64
    if [ "$ARCH" != "aarch64" ] && [ "$ARCH" != "arm64" ]; then
        error "WinnCore currently only supports ARM64. Detected: $ARCH"
    fi

    # Validate supported OS
    case "$OS" in
        ubuntu|debian|raspbian)
            info "Supported OS detected: $OS"
            ;;
        *)
            warn "Untested OS: $OS. Installation may fail."
            read -p "Continue anyway? (y/N) " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                exit 1
            fi
            ;;
    esac
}

# Check prerequisites
check_prerequisites() {
    info "Checking prerequisites..."

    # Check for required commands
    for cmd in wget dpkg systemctl; do
        if ! command -v $cmd &> /dev/null; then
            error "Required command not found: $cmd"
        fi
    done

    info "All prerequisites satisfied"
}

# Download latest release
download_package() {
    info "Downloading WinnCore AV v${VERSION}..."

    DEB_URL="https://github.com/${REPO}/releases/download/v${VERSION}/${PACKAGE_NAME}_${VERSION}_aarch64.deb"
    TMP_DIR=$(mktemp -d)
    DEB_FILE="${TMP_DIR}/${PACKAGE_NAME}_${VERSION}_aarch64.deb"

    if wget -q --show-progress "$DEB_URL" -O "$DEB_FILE"; then
        info "Downloaded successfully"
    else
        error "Failed to download package from $DEB_URL"
    fi
}

# Install package
install_package() {
    info "Installing WinnCore AV..."

    if sudo dpkg -i "$DEB_FILE"; then
        info "Package installed successfully"
    else
        warn "Installation had errors. Attempting to fix dependencies..."
        sudo apt-get install -f -y
    fi

    # Cleanup
    rm -rf "$TMP_DIR"
}

# Configure service
configure_service() {
    info "Configuring systemd service..."

    # Enable but don't start (user choice)
    sudo systemctl enable av-daemon.service

    info "Service configured (not started yet)"
}

# Post-install instructions
post_install() {
    echo
    echo -e "${GREEN}┌────────────────────────────────────────────┐${NC}"
    echo -e "${GREEN}│  WinnCore AV installed successfully!       │${NC}"
    echo -e "${GREEN}└────────────────────────────────────────────┘${NC}"
    echo
    info "Quick Start:"
    echo
    echo "  # Scan a directory"
    echo "  av-cli scan ~/Downloads"
    echo
    echo "  # Start real-time protection"
    echo "  sudo systemctl start av-daemon"
    echo
    echo "  # Check service status"
    echo "  systemctl status av-daemon"
    echo
    echo "  # View logs"
    echo "  journalctl -u av-daemon -f"
    echo
    info "Documentation: https://github.com/${REPO}"
    info "Support: zw@winncore.com"
    echo
}

# Main installation flow
main() {
    echo
    echo "╔══════════════════════════════════════════════════════════╗"
    echo "║          WinnCore AV - ARM64 Antivirus Suite            ║"
    echo "║              Installation Script v${VERSION}                  ║"
    echo "╚══════════════════════════════════════════════════════════╝"
    echo

    detect_system
    check_prerequisites
    download_package
    install_package
    configure_service
    post_install
}

# Run main function
main
