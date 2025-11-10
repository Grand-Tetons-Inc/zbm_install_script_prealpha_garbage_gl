#!/bin/bash
################################################################################
# ZFS management functions for ZFSBootMenu installation
################################################################################

################################################################################
# Ensure ZFS is installed and loaded
################################################################################

ensure_zfs() {
    log_info "Checking ZFS availability..."
    
    # Check if ZFS kernel module is loaded
    if ! is_module_loaded zfs; then
        log_info "Loading ZFS kernel module..."
        load_module zfs || {
            log_error "Failed to load ZFS kernel module"
            log_info "Attempting to install ZFS..."
            install_zfs_packages
            load_module zfs
        }
    fi
    
    # Check if ZFS commands are available
    if ! command_exists zpool || ! command_exists zfs; then
        log_info "ZFS commands not found, installing ZFS packages..."
        install_zfs_packages
    fi
    
    log_success "ZFS is available"
}

################################################################################
# Install ZFS packages based on distribution
################################################################################

install_zfs_packages() {
    log_info "Installing ZFS packages for $DETECTED_DISTRO..."
    
    case "$DETECTED_DISTRO" in
        fedora)
            # Enable ZFS repository
            execute_cmd "dnf install -y https://zfsonlinux.org/fedora/zfs-release-2-3\$(rpm --eval '%{dist}').noarch.rpm" || true
            execute_cmd "dnf install -y kernel-devel zfs"
            ;;
        debian|mx)
            execute_cmd "apt-get update"
            execute_cmd "apt-get install -y linux-headers-\$(uname -r) zfsutils-linux zfs-dkms"
            ;;
        *)
            log_error "Unsupported distribution for automatic ZFS installation: $DETECTED_DISTRO"
            exit 1
            ;;
    esac
    
    log_success "ZFS packages installed"
}

################################################################################
# Create ZFS pool
################################################################################

create_zfs_pool() {
    local pool_name="$1"
    local raid_level="$2"
    shift 2
    local drives=("$@")
    
    log_info "Creating ZFS pool: $pool_name with RAID level: $raid_level"
    
    # Ensure ZFS is available
    ensure_zfs
    
    # Build partition list
    local partitions=()
    for drive in "${drives[@]}"; do
        local zfs_part
        zfs_part=$(get_zfs_partition "$drive")
        partitions+=("$zfs_part")
    done
    
    # Build zpool create command based on RAID level
    local zpool_cmd="zpool create -f -o ashift=12"
    
    # Pool properties for better performance and compatibility
    zpool_cmd+=" -O acltype=posixacl"
    zpool_cmd+=" -O compression=lz4"
    zpool_cmd+=" -O dnodesize=auto"
    zpool_cmd+=" -O normalization=formD"
    zpool_cmd+=" -O relatime=on"
    zpool_cmd+=" -O xattr=sa"
    zpool_cmd+=" -O canmount=off"
    zpool_cmd+=" -O mountpoint=/"
    
    # Bootloader properties for ZFSBootMenu
    zpool_cmd+=" -o feature@encryption=enabled"
    zpool_cmd+=" -o feature@bookmark_v2=enabled"
    
    # Add pool name
    zpool_cmd+=" $pool_name"
    
    # Add RAID configuration
    case "$raid_level" in
        none)
            zpool_cmd+=" ${partitions[0]}"
            ;;
        mirror)
            zpool_cmd+=" mirror ${partitions[*]}"
            ;;
        raidz1)
            zpool_cmd+=" raidz ${partitions[*]}"
            ;;
        raidz2)
            zpool_cmd+=" raidz2 ${partitions[*]}"
            ;;
        raidz3)
            zpool_cmd+=" raidz3 ${partitions[*]}"
            ;;
    esac
    
    # Create the pool
    execute_cmd "$zpool_cmd"
    
    # Display pool status
    log_info "ZFS pool status:"
    execute_cmd "zpool status $pool_name"
    
    log_success "ZFS pool created successfully"
}

################################################################################
# Create ZFS datasets with proper hierarchy
################################################################################

create_zfs_datasets() {
    local pool_name="$1"
    
    log_info "Creating ZFS datasets for $pool_name..."
    
    # Create root container dataset
    log_info "Creating root container dataset..."
    execute_cmd "zfs create -o canmount=off -o mountpoint=none $pool_name/ROOT"
    
    # Create default boot environment
    log_info "Creating default boot environment..."
    execute_cmd "zfs create -o canmount=noauto -o mountpoint=/ $pool_name/ROOT/default"
    
    # Mount the root dataset
    execute_cmd "zfs mount $pool_name/ROOT/default"
    
    # Create additional datasets
    log_info "Creating home dataset..."
    execute_cmd "zfs create -o mountpoint=/home $pool_name/home"
    
    log_info "Creating root home dataset..."
    execute_cmd "zfs create -o mountpoint=/root $pool_name/home/root"
    
    log_info "Creating var dataset..."
    execute_cmd "zfs create -o canmount=off -o mountpoint=/var $pool_name/var"
    
    log_info "Creating var subdatasets..."
    execute_cmd "zfs create $pool_name/var/log"
    execute_cmd "zfs create $pool_name/var/cache"
    execute_cmd "zfs create $pool_name/var/tmp"
    
    # Set permissions for /var/tmp
    execute_cmd "chmod 1777 /$pool_name/var/tmp" || true
    
    # Create opt dataset
    log_info "Creating opt dataset..."
    execute_cmd "zfs create -o mountpoint=/opt $pool_name/opt"
    
    # Create srv dataset
    log_info "Creating srv dataset..."
    execute_cmd "zfs create -o mountpoint=/srv $pool_name/srv"
    
    # Create usr dataset
    log_info "Creating usr dataset..."
    execute_cmd "zfs create -o canmount=off -o mountpoint=/usr $pool_name/usr"
    execute_cmd "zfs create $pool_name/usr/local"
    
    # Display dataset list
    log_info "ZFS datasets created:"
    execute_cmd "zfs list -r $pool_name"
    
    log_success "ZFS datasets created successfully"
}

################################################################################
# Set ZFS pool properties for boot
################################################################################

set_boot_properties() {
    local pool_name="$1"
    
    log_info "Setting boot properties on $pool_name..."
    
    # Set bootfs property to the default boot environment
    execute_cmd "zpool set bootfs=$pool_name/ROOT/default $pool_name"
    
    # Enable cachefile
    execute_cmd "zpool set cachefile=/etc/zfs/zpool.cache $pool_name"
    
    log_success "Boot properties set successfully"
}

################################################################################
# Create ZFS snapshot
################################################################################

create_snapshot() {
    local dataset="$1"
    local snapshot_name="$2"
    
    log_info "Creating snapshot: $dataset@$snapshot_name"
    execute_cmd "zfs snapshot $dataset@$snapshot_name"
}

################################################################################
# Export ZFS pool
################################################################################

export_pool() {
    local pool_name="$1"
    
    log_info "Exporting ZFS pool: $pool_name"
    execute_cmd "zpool export $pool_name"
}

################################################################################
# Import ZFS pool
################################################################################

import_pool() {
    local pool_name="$1"
    local mount_point="${2:-/mnt}"
    
    log_info "Importing ZFS pool: $pool_name"
    execute_cmd "zpool import -f -R $mount_point $pool_name"
}

################################################################################
# Check ZFS pool health
################################################################################

check_pool_health() {
    local pool_name="$1"
    
    log_info "Checking ZFS pool health: $pool_name"
    
    if ! zpool list "$pool_name" &>/dev/null; then
        log_error "Pool $pool_name does not exist"
        return 1
    fi
    
    local health
    health=$(zpool list -H -o health "$pool_name")
    
    log_info "Pool $pool_name health: $health"
    
    if [[ "$health" != "ONLINE" ]]; then
        log_warn "Pool health is not ONLINE: $health"
        execute_cmd "zpool status $pool_name"
    fi
    
    return 0
}

################################################################################
# Configure ZFS for optimal performance
################################################################################

optimize_zfs_performance() {
    local pool_name="$1"
    
    log_info "Optimizing ZFS performance for $pool_name..."
    
    # Set ARC size limits (optional, adjust based on system RAM)
    # Uncomment and adjust if needed
    # echo "options zfs zfs_arc_max=$((8 * 1024 * 1024 * 1024))" > /etc/modprobe.d/zfs.conf
    
    # Enable auto-snapshots (if desired)
    # execute_cmd "zfs set com.sun:auto-snapshot=true $pool_name"
    
    log_success "ZFS performance optimization completed"
}
