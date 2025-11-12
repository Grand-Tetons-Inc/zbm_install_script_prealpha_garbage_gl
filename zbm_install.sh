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
# shellcheck source=lib/validation.sh
source "${SCRIPT_DIR}/lib/validation.sh"
# shellcheck source=lib/disk.sh
source "${SCRIPT_DIR}/lib/disk.sh"
# shellcheck source=lib/zfs.sh
source "${SCRIPT_DIR}/lib/zfs.sh"
# shellcheck source=lib/bootloader.sh
source "${SCRIPT_DIR}/lib/bootloader.sh"
# shellcheck source=lib/system.sh
source "${SCRIPT_DIR}/lib/system.sh"
# shellcheck source=lib/device_tuning.sh
source "${SCRIPT_DIR}/lib/device_tuning.sh"
# shellcheck source=lib/network_zap.sh
source "${SCRIPT_DIR}/lib/network_zap.sh"

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
VERBOSE=false
SKIP_PREFLIGHT=false
ASHIFT=""  # Auto-detect if empty
COMPRESSION="zstd"  # zstd, lz4, lzjb, gzip, or off
HOSTNAME=""
BOOTLOADER="zbm"  # zbm (default), systemd-boot, or refind
BACKUP_CONFIG=true
SOURCE_ROOT="/"
EXCLUDE_PATHS=()
COPY_HOME=true
NVME_FORMAT_4K=false

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
    -s, --swap-size SIZE     Swap partition size (0 to disable, default: 8G)
    -a, --ashift VALUE       ZFS ashift value (9-16, auto-detect if not specified)
    -c, --compression TYPE   ZFS compression: zstd, lz4, lzjb, gzip, off (default: zstd)
    -H, --hostname NAME      Set hostname for new installation
    -b, --bootloader TYPE    Bootloader: zbm, systemd-boot, refind (default: zbm)
    --source-root PATH       Source root for existing mode (default: /)
    --exclude PATH           Additional paths to exclude (can be used multiple times)
    --no-copy-home           Don't copy home directories in existing mode
    -n, --dry-run            Show what would be done without making changes
    -f, --force              Skip confirmation prompts
    -v, --verbose            Enable verbose output
    -S, --skip-preflight     Skip pre-flight system checks (not recommended)
    -B, --no-backup          Don't backup existing configuration
    -l, --log-file PATH      Custom log file path
    --nvme-format-4k         Format NVMe drives to 4K sectors (DESTROYS DATA!)
    -h, --help               Display this help message

EXAMPLES:
    # Install on single drive (new installation)
    sudo $0 -m new -d sda

    # Install on mirrored drives
    sudo $0 -m new -d sda,sdb -r mirror

    # Copy existing system to new mirrored ZFS setup
    sudo $0 -m existing -d sda,sdb -r mirror

    # Copy existing system excluding specific paths
    sudo $0 -m existing -d nvme0n1 --exclude /home/user/Downloads --exclude /var/tmp

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
            -a|--ashift)
                ASHIFT="$2"
                shift 2
                ;;
            -c|--compression)
                COMPRESSION="$2"
                shift 2
                ;;
            -H|--hostname)
                HOSTNAME="$2"
                shift 2
                ;;
            -b|--bootloader)
                BOOTLOADER="$2"
                shift 2
                ;;
            --source-root)
                SOURCE_ROOT="$2"
                shift 2
                ;;
            --exclude)
                EXCLUDE_PATHS+=("$2")
                shift 2
                ;;
            --no-copy-home)
                COPY_HOME=false
                shift
                ;;
            -n|--dry-run)
                DRY_RUN=true
                shift
                ;;
            -f|--force)
                FORCE=true
                shift
                ;;
            -v|--verbose)
                VERBOSE=true
                shift
                ;;
            -S|--skip-preflight)
                SKIP_PREFLIGHT=true
                shift
                ;;
            -B|--no-backup)
                BACKUP_CONFIG=false
                shift
                ;;
            -l|--log-file)
                LOG_FILE="$2"
                shift 2
                ;;
            --nvme-format-4k)
                NVME_FORMAT_4K=true
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

    # Validate bootloader
    if [[ "$BOOTLOADER" != "zbm" && "$BOOTLOADER" != "systemd-boot" && "$BOOTLOADER" != "refind" ]]; then
        log_error "Invalid bootloader: $BOOTLOADER (must be 'zbm', 'systemd-boot', or 'refind')"
        exit 1
    fi

    # Check if requested bootloader exists (only for non-zbm)
    if [[ "$BOOTLOADER" == "systemd-boot" ]] && ! command_exists bootctl; then
        log_error "Bootloader 'systemd-boot' requested but bootctl command not found"
        log_error "Install systemd-boot or use --bootloader zbm"
        exit 1
    elif [[ "$BOOTLOADER" == "refind" ]] && ! command_exists refind-install; then
        log_error "Bootloader 'refind' requested but refind-install command not found"
        log_error "Install rEFInd or use --bootloader zbm"
        exit 1
    fi

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
Bootloader:         $BOOTLOADER
EFI Partition Size: $EFI_SIZE
Swap Size:          $SWAP_SIZE
Distribution:       $DISTRO $DISTRO_VERSION
Dry Run:            $DRY_RUN

EOF

    if [[ "$INSTALL_MODE" == "existing" ]]; then
        echo "System Copy Settings:"
        echo "  Source Root:    $SOURCE_ROOT"
        echo "  Copy Home:      $COPY_HOME"
        if [[ ${#EXCLUDE_PATHS[@]} -gt 0 ]]; then
            echo "  Custom Exclusions:"
            for exclude in "${EXCLUDE_PATHS[@]}"; do
                echo "    - $exclude"
            done
        fi
        echo ""
        log_warn "WARNING: Existing system will be copied to new ZFS installation"
    fi

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

    # Run pre-flight checks
    if [[ "$SKIP_PREFLIGHT" == "false" ]]; then
        if ! preflight_checks; then
            log_error "Pre-flight checks failed"
            if [[ "$FORCE" != "true" ]]; then
                exit 1
            else
                log_warn "Continuing anyway (--force specified)"
            fi
        fi
    else
        log_warn "Skipping pre-flight checks (--skip-preflight specified)"
    fi

    # Display summary and confirm
    display_summary

    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "DRY RUN MODE - No changes will be made"
    fi

    # Device fitness checks and tuning
    log_step "Checking device fitness and tuning"
    for drive in "${DRIVES[@]}"; do
        log_info "Checking device: $drive"

        # Check device fitness
        if ! check_device_fitness "$drive"; then
            log_error "Device fitness check failed for $drive"
            if [[ "$FORCE" != "true" ]]; then
                exit 1
            else
                log_warn "Continuing anyway (--force specified)"
            fi
        fi

        # Verify not source device
        if ! verify_not_source_device "$drive"; then
            log_error "Device $drive appears to be the source system device!"
            log_error "Refusing to proceed to protect running system"
            exit 1
        fi

        # Optimize device parameters
        optimize_device_parameters "$drive"

        # Format NVMe to 4K sectors if requested
        if [[ "$NVME_FORMAT_4K" == "true" ]] && [[ "$drive" =~ ^nvme ]]; then
            if nvme_supports_4k "$drive"; then
                local current_size
                current_size=$(get_nvme_sector_size "$drive")

                if [[ "$current_size" != "4096" ]]; then
                    log_warn "Formatting NVMe $drive to 4K sectors..."
                    if ! set_nvme_4k_sectors "$drive"; then
                        log_error "Failed to format NVMe device to 4K sectors"
                        if [[ "$FORCE" != "true" ]]; then
                            exit 1
                        fi
                    fi
                else
                    log_info "NVMe $drive already using 4K sectors"
                fi
            else
                log_warn "NVMe $drive does not support 4K sectors"
            fi
        fi
    done

    log_success "All devices passed fitness checks"

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

    # Step 4.5: Copy existing system if in existing mode
    if [[ "$INSTALL_MODE" == "existing" ]]; then
        log_step "Step 4.5: Copying existing system"

        # Add /home/* to exclusions if --no-copy-home specified
        if [[ "$COPY_HOME" == "false" ]]; then
            log_info "Excluding home directories from copy"
            EXCLUDE_PATHS+=("/home/*")
        fi

        # Display copy summary
        if [[ "$VERBOSE" == "true" ]] || [[ "$DRY_RUN" == "true" ]]; then
            display_copy_summary
        fi

        if ! copy_existing_system; then
            log_error "Failed to copy existing system"
            if [[ "$FORCE" != "true" ]]; then
                exit 1
            fi
        fi
    fi

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
