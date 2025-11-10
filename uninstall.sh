#!/bin/bash

# Autodns Uninstallation Script
# This script removes Autodns from the system

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

# Function to check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        print_error "This script must be run as root (use sudo)"
        exit 1
    fi
}

# Function to stop and disable service
stop_service() {
    print_info "Stopping Autodns service..."

    if systemctl is-active --quiet autodns; then
        systemctl stop autodns || {
            print_warn "Failed to stop service"
        }
        print_info "Service stopped"
    else
        print_info "Service is not running"
    fi

    if systemctl is-enabled --quiet autodns; then
        systemctl disable autodns || {
            print_warn "Failed to disable service"
        }
        print_info "Service disabled"
    else
        print_info "Service was not enabled"
    fi
}

# Function to remove systemd service
remove_service() {
    print_info "Removing systemd service..."

    if [ -f "$SERVICE_FILE" ]; then
        rm -f "$SERVICE_FILE" || {
            print_error "Failed to remove service file"
            exit 1
        }
        systemctl daemon-reload || {
            print_warn "Failed to reload systemd daemon"
        }
        print_info "Service file removed"
    else
        print_info "Service file not found"
    fi
}

# Function to remove binary
remove_binary() {
    print_info "Removing binary..."

    local binary_path="$INSTALL_DIR/$BINARY_NAME"

    if [ -f "$binary_path" ]; then
        rm -f "$binary_path" || {
            print_error "Failed to remove binary"
            exit 1
        }
        print_info "Binary removed: $binary_path"
    else
        print_info "Binary not found"
    fi
}

# Function to remove configuration
remove_config() {
    print_info "Handling configuration files..."

    if [ -d "$CONFIG_DIR" ]; then
        echo ""
        echo "Configuration directory found: $CONFIG_DIR"
        echo "This contains your DNS configuration."
        echo ""
        read -p "Do you want to remove configuration? (y/N): " -n 1 -r
        echo ""

        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -rf "$CONFIG_DIR" || {
                print_error "Failed to remove config directory"
                exit 1
            }
            print_info "Configuration removed"
        else
            print_info "Configuration kept at: $CONFIG_DIR"
            print_info "You can remove it manually later if needed"
        fi
    else
        print_info "Configuration directory not found"
    fi
}

# Function to restore resolv.conf backup
restore_resolv_conf() {
    print_info "Checking for resolv.conf backup..."

    if [ -f "/etc/resolv.conf.backup" ]; then
        echo ""
        read -p "Do you want to restore /etc/resolv.conf from backup? (y/N): " -n 1 -r
        echo ""

        if [[ $REPLY =~ ^[Yy]$ ]]; then
            cp /etc/resolv.conf.backup /etc/resolv.conf || {
                print_warn "Failed to restore resolv.conf backup"
            }
            print_info "Restored /etc/resolv.conf from backup"
        else
            print_info "Keeping current /etc/resolv.conf"
        fi
    else
        print_info "No resolv.conf backup found"
    fi
}

# Function to display post-uninstallation message
show_post_uninstall_message() {
    echo ""
    print_info "═══════════════════════════════════════════════════════════════"
    print_info "Autodns has been uninstalled"
    print_info "═══════════════════════════════════════════════════════════════"
    echo ""

    if [ -d "$CONFIG_DIR" ]; then
        echo "Note: Configuration files were kept at: $CONFIG_DIR"
        echo "To remove them manually:"
        echo "  sudo rm -rf $CONFIG_DIR"
        echo ""
    fi

    echo "Your DNS configuration in /etc/resolv.conf was not modified."
    echo "If you need to reconfigure DNS manually:"
    echo "  sudo nano /etc/resolv.conf"
    echo ""
    print_info "═══════════════════════════════════════════════════════════════"
}

# Main uninstallation flow
main() {
    echo ""
    print_info "═══════════════════════════════════════════════════════════════"
    print_info "Autodns Uninstallation Script"
    print_info "═══════════════════════════════════════════════════════════════"
    echo ""

    check_root

    # Confirm uninstallation
    echo "This will remove Autodns from your system."
    read -p "Are you sure you want to continue? (y/N): " -n 1 -r
    echo ""

    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Uninstallation cancelled"
        exit 0
    fi

    echo ""

    # Uninstall steps
    stop_service
    remove_service
    remove_binary
    remove_config
    restore_resolv_conf

    # Show completion message
    show_post_uninstall_message
}

# Run main function
main
