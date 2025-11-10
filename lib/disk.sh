#!/bin/bash
################################################################################
# Disk management functions for ZFSBootMenu installation
################################################################################

################################################################################
# Validate RAID level against number of drives
################################################################################

validate_raid_level() {
    local raid="$1"
    local num_drives="$2"
    
    case "$raid" in
        none)
            if [[ $num_drives -ne 1 ]]; then
                log_error "RAID level 'none' requires exactly 1 drive, got $num_drives"
                exit 1
            fi
            ;;
        mirror)
            if [[ $num_drives -lt 2 ]]; then
                log_error "RAID level 'mirror' requires at least 2 drives, got $num_drives"
                exit 1
            fi
            ;;
        raidz1)
            if [[ $num_drives -lt 3 ]]; then
                log_error "RAID level 'raidz1' requires at least 3 drives, got $num_drives"
                exit 1
            fi
            ;;
        raidz2)
            if [[ $num_drives -lt 4 ]]; then
                log_error "RAID level 'raidz2' requires at least 4 drives, got $num_drives"
                exit 1
            fi
            ;;
        raidz3)
            if [[ $num_drives -lt 5 ]]; then
                log_error "RAID level 'raidz3' requires at least 5 drives, got $num_drives"
                exit 1
            fi
            ;;
        *)
            log_error "Invalid RAID level: $raid (valid: none, mirror, raidz1, raidz2, raidz3)"
            exit 1
            ;;
    esac
}

################################################################################
# Prepare disks (wipe and clear)
################################################################################

prepare_disks() {
    local drives=("$@")
    
    log_info "Preparing disks..."
    
    for drive in "${drives[@]}"; do
        log_info "Preparing /dev/$drive"
        
        # Check if drive exists
        if [[ ! -b "/dev/$drive" ]]; then
            log_error "Drive /dev/$drive does not exist"
            exit 1
        fi
        
        # Display drive information
        log_info "Drive information for /dev/$drive:"
        execute_cmd "lsblk -o NAME,SIZE,TYPE,MOUNTPOINT /dev/$drive" || true
        
        if [[ "$INSTALL_MODE" == "new" ]]; then
            # Warn and confirm before wiping
            log_warn "About to wipe all data on /dev/$drive"
            
            # Unmount any mounted partitions
            log_info "Unmounting any mounted partitions on /dev/$drive"
            for partition in /dev/"${drive}"*; do
                if [[ -b "$partition" ]]; then
                    umount "$partition" 2>/dev/null || true
                fi
            done
            
            # Wipe filesystem signatures
            log_info "Wiping filesystem signatures on /dev/$drive"
            execute_cmd "wipefs -a /dev/$drive"
            
            # Clear partition table
            log_info "Clearing partition table on /dev/$drive"
            execute_cmd "sgdisk --zap-all /dev/$drive"
            
            # Inform kernel of partition table changes
            execute_cmd "partprobe /dev/$drive" || true
            
            # Short delay to let kernel recognize changes
            sleep 2
        fi
    done
    
    log_success "Disk preparation completed"
}

################################################################################
# Create partitions on drives
################################################################################

create_partitions() {
    local drives=("$@")
    local partition_num=1
    
    log_info "Creating partitions..."
    
    for drive in "${drives[@]}"; do
        log_info "Creating partitions on /dev/$drive"
        
        # Partition 1: EFI System Partition (ESP)
        log_info "Creating EFI partition (${EFI_SIZE}) on /dev/$drive"
        execute_cmd "sgdisk -n ${partition_num}:1M:+${EFI_SIZE} -t ${partition_num}:EF00 -c ${partition_num}:'EFI System Partition' /dev/$drive"
        
        partition_num=$((partition_num + 1))
        
        # Partition 2: Swap partition (optional)
        if [[ -n "$SWAP_SIZE" ]] && [[ "$SWAP_SIZE" != "0" ]]; then
            log_info "Creating swap partition (${SWAP_SIZE}) on /dev/$drive"
            execute_cmd "sgdisk -n ${partition_num}:0:+${SWAP_SIZE} -t ${partition_num}:8200 -c ${partition_num}:'Linux swap' /dev/$drive"
            partition_num=$((partition_num + 1))
        fi
        
        # Partition 3: ZFS root partition (use remaining space)
        log_info "Creating ZFS partition (remaining space) on /dev/$drive"
        execute_cmd "sgdisk -n ${partition_num}:0:0 -t ${partition_num}:BF00 -c ${partition_num}:'ZFS root pool' /dev/$drive"
        
        # Inform kernel of partition table changes
        execute_cmd "partprobe /dev/$drive" || true
        
        # Wait for partition devices to appear
        sleep 2
        
        # Display partition layout
        log_info "Partition layout for /dev/$drive:"
        execute_cmd "sgdisk -p /dev/$drive" || true
        
        # Reset partition number for next drive
        partition_num=1
    done
    
    log_success "Partition creation completed"
}

################################################################################
# Format EFI partitions
################################################################################

format_efi_partitions() {
    local drives=("$@")
    
    log_info "Formatting EFI partitions..."
    
    for drive in "${drives[@]}"; do
        local efi_partition
        
        # Determine partition naming scheme
        if [[ "$drive" =~ nvme ]]; then
            efi_partition="/dev/${drive}p1"
        else
            efi_partition="/dev/${drive}1"
        fi
        
        log_info "Formatting EFI partition: $efi_partition"
        execute_cmd "mkfs.vfat -F32 -n EFI $efi_partition"
    done
    
    log_success "EFI partition formatting completed"
}

################################################################################
# Setup swap partitions
################################################################################

setup_swap() {
    local drives=("$@")
    
    if [[ -z "$SWAP_SIZE" ]] || [[ "$SWAP_SIZE" == "0" ]]; then
        log_info "Skipping swap setup (size is 0 or not specified)"
        return 0
    fi
    
    log_info "Setting up swap partitions..."
    
    for drive in "${drives[@]}"; do
        local swap_partition
        
        # Determine partition naming scheme
        if [[ "$drive" =~ nvme ]]; then
            swap_partition="/dev/${drive}p2"
        else
            swap_partition="/dev/${drive}2"
        fi
        
        log_info "Setting up swap on: $swap_partition"
        execute_cmd "mkswap -L swap $swap_partition"
        execute_cmd "swapon $swap_partition"
    done
    
    log_success "Swap setup completed"
}

################################################################################
# Get ZFS partition for a drive
################################################################################

get_zfs_partition() {
    local drive="$1"
    local partition_num=3
    
    # If swap is disabled, ZFS partition is the second one
    if [[ -z "$SWAP_SIZE" ]] || [[ "$SWAP_SIZE" == "0" ]]; then
        partition_num=2
    fi
    
    # Determine partition naming scheme
    if [[ "$drive" =~ nvme ]]; then
        echo "/dev/${drive}p${partition_num}"
    else
        echo "/dev/${drive}${partition_num}"
    fi
}

################################################################################
# Display disk information
################################################################################

display_disk_info() {
    log_info "Available disks:"
    execute_cmd "lsblk -o NAME,SIZE,TYPE,MOUNTPOINT"
}

################################################################################
# Check disk health
################################################################################

check_disk_health() {
    local drive="$1"
    
    if command_exists smartctl; then
        log_info "Checking SMART status for /dev/$drive"
        smartctl -H "/dev/$drive" 2>&1 | tee -a "$LOG_FILE" || log_warn "SMART check failed or not supported"
    else
        log_warn "smartctl not available, skipping SMART check"
    fi
}
