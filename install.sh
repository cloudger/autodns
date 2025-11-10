#!/bin/bash

# Autodns Installation Script
# This script compiles as normal user and installs as root

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
BINARY_NAME="autodns"
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="/etc/autodns"
SERVICE_FILE="/etc/systemd/system/autodns.service"
BUILD_TARGET="x86_64-unknown-linux-musl"

# Function to print colored messages
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check dependencies
check_dependencies() {
    print_info "Checking dependencies..."

    local missing_deps=()

    if ! command_exists cargo; then
        missing_deps+=("cargo (Rust toolchain)")
    fi

    if ! command_exists systemctl; then
        missing_deps+=("systemctl (systemd)")
    fi

    if [ ${#missing_deps[@]} -ne 0 ]; then
        print_error "Missing dependencies:"
        for dep in "${missing_deps[@]}"; do
            echo "  - $dep"
        done
        print_info "Install Rust from: https://rustup.rs/"
        exit 1
    fi

    print_info "All dependencies found"
}

# Function to build the binary (as normal user)
build_binary() {
    print_info "Building Autodns binary (release mode with musl target)..."

    # Check if musl target is installed
    if ! rustup target list --installed | grep -q "$BUILD_TARGET"; then
        print_info "Installing musl target..."
        rustup target add "$BUILD_TARGET" || {
            print_error "Failed to install musl target"
            exit 1
        }
    fi

    # Build the binary
    cargo build --release --target "$BUILD_TARGET" || {
        print_error "Failed to build binary"
        exit 1
    }

    print_info "Build successful"
}

# Function to check if binary exists
check_binary() {
    local binary_path="target/$BUILD_TARGET/release/$BINARY_NAME"

    if [ ! -f "$binary_path" ]; then
        print_error "Binary not found at $binary_path"
        print_error "Please run the build first"
        exit 1
    fi
}

# Function to install binary (requires root)
install_binary() {
    print_info "Installing binary to $INSTALL_DIR..."

    local binary_path="target/$BUILD_TARGET/release/$BINARY_NAME"

    sudo cp "$binary_path" "$INSTALL_DIR/$BINARY_NAME" || {
        print_error "Failed to copy binary"
        exit 1
    }

    sudo chmod +x "$INSTALL_DIR/$BINARY_NAME" || {
        print_error "Failed to set executable permission"
        exit 1
    }

    print_info "Binary installed successfully"
}

# Function to create config directory and copy config (requires root)
install_config() {
    print_info "Setting up configuration..."

    # Create config directory if it doesn't exist
    if [ ! -d "$CONFIG_DIR" ]; then
        sudo mkdir -p "$CONFIG_DIR" || {
            print_error "Failed to create config directory"
            exit 1
        }
        print_info "Created config directory: $CONFIG_DIR"
    fi

    # Copy config file if it doesn't exist
    if [ ! -f "$CONFIG_DIR/config.yaml" ]; then
        if [ -f "config.yaml" ]; then
            sudo cp config.yaml "$CONFIG_DIR/config.yaml" || {
                print_error "Failed to copy config file"
                exit 1
            }
            print_info "Configuration file installed: $CONFIG_DIR/config.yaml"
        else
            print_warn "config.yaml not found in current directory"
            print_warn "You'll need to create $CONFIG_DIR/config.yaml manually"
        fi
    else
        print_warn "Configuration file already exists: $CONFIG_DIR/config.yaml"
        print_warn "Keeping existing configuration (not overwriting)"
    fi
}

# Function to validate configuration
validate_config() {
    print_info "Validating configuration..."

    if [ ! -f "$CONFIG_DIR/config.yaml" ]; then
        print_error "Configuration file not found: $CONFIG_DIR/config.yaml"
        exit 1
    fi

    # Test configuration by running check command
    sudo "$INSTALL_DIR/$BINARY_NAME" --config "$CONFIG_DIR/config.yaml" check >/dev/null 2>&1 || {
        print_error "Configuration validation failed"
        print_error "Please check your configuration at: $CONFIG_DIR/config.yaml"
        exit 1
    }

    print_info "Configuration is valid"
}

# Function to install systemd service (requires root)
install_service() {
    print_info "Installing systemd service..."

    if [ ! -f "autodns.service" ]; then
        print_error "Service file not found: autodns.service"
        exit 1
    fi

    # Copy service file
    sudo cp autodns.service "$SERVICE_FILE" || {
        print_error "Failed to copy service file"
        exit 1
    }

    # Reload systemd daemon
    sudo systemctl daemon-reload || {
        print_error "Failed to reload systemd daemon"
        exit 1
    }

    print_info "Systemd service installed"
}

# Function to check /etc/resolv.conf permissions
check_resolv_conf_permissions() {
    print_info "Checking /etc/resolv.conf permissions..."

    if [ ! -w /etc/resolv.conf ]; then
        print_warn "/etc/resolv.conf is not writable by current user"
        print_info "Autodns will run as root via systemd service"

        # Check if it's a symlink to systemd-resolved
        if [ -L /etc/resolv.conf ]; then
            local target=$(readlink -f /etc/resolv.conf)
            print_warn "/etc/resolv.conf is a symlink to: $target"

            if [[ "$target" == *"systemd/resolve"* ]]; then
                print_warn "Your system uses systemd-resolved"
                print_warn "Consider configuring Autodns to use a different path or disable systemd-resolved"
            fi
        fi
    else
        print_info "/etc/resolv.conf is writable"
    fi
}

# Function to display post-installation instructions
show_post_install_instructions() {
    echo ""
    print_info "═══════════════════════════════════════════════════════════════"
    print_info "Autodns installation completed successfully!"
    print_info "═══════════════════════════════════════════════════════════════"
    echo ""
    echo "Next steps:"
    echo ""
    echo "1. Edit configuration (if needed):"
    echo "   sudo nano $CONFIG_DIR/config.yaml"
    echo ""
    echo "2. Test the configuration:"
    echo "   sudo $BINARY_NAME --config $CONFIG_DIR/config.yaml check"
    echo ""
    echo "3. Enable the service to start on boot:"
    echo "   sudo systemctl enable autodns"
    echo ""
    echo "4. Start the service:"
    echo "   sudo systemctl start autodns"
    echo ""
    echo "5. Check service status:"
    echo "   sudo systemctl status autodns"
    echo ""
    echo "6. View logs:"
    echo "   sudo journalctl -u autodns -f"
    echo ""
    print_info "═══════════════════════════════════════════════════════════════"
}

# Main installation flow
main() {
    echo ""
    print_info "═══════════════════════════════════════════════════════════════"
    print_info "Autodns Installation Script"
    print_info "═══════════════════════════════════════════════════════════════"
    echo ""

    # Phase 1: Build (as normal user)
    print_info "Phase 1: Building binary (as current user)"
    check_dependencies
    build_binary

    echo ""
    print_info "Phase 2: Installing to system (requires sudo)"
    echo ""

    # Check if binary was built
    check_binary

    # Phase 2: Install (with sudo)
    install_binary
    install_config

    # Validate configuration
    validate_config

    # Install service
    install_service

    # Post-installation checks
    check_resolv_conf_permissions

    # Show next steps
    show_post_install_instructions
}

# Run main function
main
