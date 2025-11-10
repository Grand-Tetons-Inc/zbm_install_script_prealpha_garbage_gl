#!/bin/bash
################################################################################
# ZFSBootMenu Installation Script
# 
# This script installs ZFSBootMenu on one or more drives with EFI partitions
# Supports: Fedora 42/43, Debian 13, MX Linux 25 RC1
# Based on: https://docs.zfsbootmenu.org/en/latest/guides
#
# Usage: sudo ./zbm_install.sh [OPTIONS]
#
# MIT License - Copyright (c) 2025 Grand-Tetons-Inc
################################################################################

set -e  # Exit on error
set -u  # Exit on undefined variable
set -o pipefail  # Exit on pipe failure

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source library functions
# shellcheck source=lib/common.sh
source "${SCRIPT_DIR}/lib/common.sh"
# shellcheck source=lib/disk.sh
source "${SCRIPT_DIR}/lib/disk.sh"
# shellcheck source=lib/zfs.sh
source "${SCRIPT_DIR}/lib/zfs.sh"
# shellcheck source=lib/bootloader.sh
source "${SCRIPT_DIR}/lib/bootloader.sh"

# Global configuration
INSTALL_MODE=""  # "new" or "existing"
POOL_NAME="zroot"
DRIVES=()
RAID_LEVEL="none"  # none, mirror, raidz1, raidz2, raidz3
EFI_SIZE="1G"
SWAP_SIZE="8G"
DISTRO=""
DISTRO_VERSION=""
DRY_RUN=false
FORCE=false

# Set log file based on permissions
if [[ -w /var/log ]]; then
    LOG_FILE="/var/log/zbm_install.log"
else
    LOG_FILE="/tmp/zbm_install.log"
fi

################################################################################
# Display usage information
################################################################################
usage() {
    cat << EOF
ZFSBootMenu Installation Script

Usage: sudo $0 [OPTIONS]

OPTIONS:
    -m, --mode MODE          Installation mode: 'new' or 'existing' (required)
    -d, --drives DRIVES      Comma-separated list of drives (e.g., sda,sdb)
    -p, --pool NAME          ZFS pool name (default: zroot)
    -r, --raid LEVEL         RAID level: none, mirror, raidz1, raidz2, raidz3
    -e, --efi-size SIZE      EFI partition size (default: 1G)
    -s, --swap-size SIZE     Swap partition size (default: 8G)
    -n, --dry-run            Show what would be done without making changes
    -f, --force              Skip confirmation prompts
    -h, --help               Display this help message

EXAMPLES:
    # Install on single drive (new installation)
    sudo $0 -m new -d sda

    # Install on mirrored drives
    sudo $0 -m new -d sda,sdb -r mirror

    # Install on existing system with RAIDZ1
    sudo $0 -m existing -d sda,sdb,sdc -r raidz1

EOF
    exit 0
}

################################################################################
# Parse command line arguments
################################################################################
parse_args() {
    if [[ $# -eq 0 ]]; then
        usage
    fi

    while [[ $# -gt 0 ]]; do
        case "$1" in
            -m|--mode)
                INSTALL_MODE="$2"
                shift 2
                ;;
            -d|--drives)
                IFS=',' read -ra DRIVES <<< "$2"
                shift 2
                ;;
            -p|--pool)
                POOL_NAME="$2"
                shift 2
                ;;
            -r|--raid)
                RAID_LEVEL="$2"
                shift 2
                ;;
            -e|--efi-size)
                EFI_SIZE="$2"
                shift 2
                ;;
            -s|--swap-size)
                SWAP_SIZE="$2"
                shift 2
                ;;
            -n|--dry-run)
                DRY_RUN=true
                shift
                ;;
            -f|--force)
                FORCE=true
                shift
                ;;
            -h|--help)
                usage
                ;;
            *)
                log_error "Unknown option: $1"
                usage
                ;;
        esac
    done
}

################################################################################
# Validate configuration
################################################################################
validate_config() {
    log_info "Validating configuration..."

    # Check if running as root
    if [[ $EUID -ne 0 ]]; then
        log_error "This script must be run as root"
        exit 1
    fi

    # Check install mode
    if [[ -z "$INSTALL_MODE" ]]; then
        log_error "Installation mode (-m) is required"
        exit 1
    fi

    if [[ "$INSTALL_MODE" != "new" && "$INSTALL_MODE" != "existing" ]]; then
        log_error "Invalid installation mode: $INSTALL_MODE (must be 'new' or 'existing')"
        exit 1
    fi

    # Check drives
    if [[ ${#DRIVES[@]} -eq 0 ]]; then
        log_error "At least one drive must be specified (-d)"
        exit 1
    fi

    # Validate RAID level
    validate_raid_level "$RAID_LEVEL" "${#DRIVES[@]}"

    # Validate drives exist
    for drive in "${DRIVES[@]}"; do
        if [[ ! -b "/dev/$drive" ]]; then
            log_error "Drive /dev/$drive does not exist"
            exit 1
        fi
    done

    log_success "Configuration validated successfully"
}

################################################################################
# Display configuration summary
################################################################################
display_summary() {
    cat << EOF

================================================================================
                    ZFSBootMenu Installation Summary
================================================================================

Installation Mode:  $INSTALL_MODE
Pool Name:          $POOL_NAME
Drives:             ${DRIVES[*]}
RAID Level:         $RAID_LEVEL
EFI Partition Size: $EFI_SIZE
Swap Size:          $SWAP_SIZE
Distribution:       $DISTRO $DISTRO_VERSION
Dry Run:            $DRY_RUN

EOF

    if [[ "$INSTALL_MODE" == "new" ]]; then
        log_warn "WARNING: All data on the following drives will be DESTROYED:"
        for drive in "${DRIVES[@]}"; do
            echo "  - /dev/$drive"
        done
    fi

    if [[ "$FORCE" == "false" ]]; then
        echo ""
        read -p "Do you want to proceed? (yes/no): " -r
        if [[ ! $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
            log_info "Installation cancelled by user"
            exit 0
        fi
    fi
}

################################################################################
# Main installation workflow
################################################################################
main() {
    log_info "Starting ZFSBootMenu installation..."
    log_info "Log file: $LOG_FILE"

    # Parse arguments
    parse_args "$@"

    # Detect distribution
    detect_distribution
    DISTRO="$DETECTED_DISTRO"
    DISTRO_VERSION="$DETECTED_VERSION"

    # Validate configuration
    validate_config

    # Display summary and confirm
    display_summary

    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "DRY RUN MODE - No changes will be made"
    fi

    # Step 1: Prepare disks
    log_step "Step 1: Preparing disks"
    prepare_disks "${DRIVES[@]}"

    # Step 2: Create partitions
    log_step "Step 2: Creating partitions"
    create_partitions "${DRIVES[@]}"

    # Step 3: Create ZFS pool
    log_step "Step 3: Creating ZFS pool"
    create_zfs_pool "$POOL_NAME" "$RAID_LEVEL" "${DRIVES[@]}"

    # Step 4: Create ZFS datasets
    log_step "Step 4: Creating ZFS datasets"
    create_zfs_datasets "$POOL_NAME"

    # Step 5: Install ZFSBootMenu
    log_step "Step 5: Installing ZFSBootMenu"
    install_zfsbootmenu "$DISTRO" "$POOL_NAME"

    # Step 6: Configure bootloader
    log_step "Step 6: Configuring bootloader"
    configure_bootloader "${DRIVES[@]}"

    # Step 7: Final configuration
    log_step "Step 7: Finalizing configuration"
    finalize_installation "$POOL_NAME"

    log_success "ZFSBootMenu installation completed successfully!"
    log_info "Please reboot your system to boot into ZFSBootMenu"
}

# Run main function
main "$@"
