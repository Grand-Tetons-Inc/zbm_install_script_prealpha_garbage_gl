#!/bin/bash
################################################################################
# Device Tuning and Fitness Checks
#
# Advanced device inspection, NVMe sector size configuration, and fitness
# validation before any destructive operations
################################################################################

################################################################################
# Check if nvme-cli is available
################################################################################
has_nvme_cli() {
    command -v nvme &>/dev/null
}

################################################################################
# Check if hdparm is available
################################################################################
has_hdparm() {
    command -v hdparm &>/dev/null
}

################################################################################
# Check if smartctl is available
################################################################################
has_smartctl() {
    command -v smartctl &>/dev/null
}

################################################################################
# Get NVMe namespace information
################################################################################
get_nvme_namespace_info() {
    local device="$1"
    local namespace="${device}n1"  # Assume namespace 1

    if ! has_nvme_cli; then
        log_warn "nvme-cli not available, skipping NVMe inspection"
        return 1
    fi

    if [[ ! "$device" =~ ^nvme ]]; then
        return 1  # Not an NVMe device
    fi

    # Get namespace info
    local ns_info
    ns_info=$(nvme id-ns "/dev/${namespace}" 2>/dev/null)

    if [[ $? -ne 0 ]]; then
        log_warn "Failed to get NVMe namespace info for $namespace"
        return 1
    fi

    echo "$ns_info"
    return 0
}

################################################################################
# Get supported LBA formats for NVMe device
################################################################################
get_nvme_lba_formats() {
    local device="$1"
    local namespace="${device}n1"

    local ns_info
    ns_info=$(get_nvme_namespace_info "$device")

    if [[ $? -ne 0 ]]; then
        return 1
    fi

    # Extract LBA formats
    echo "$ns_info" | grep "^LBA Format" | while read -r line; do
        # Parse format: "LBA Format  0 : Metadata Size: 0   bytes - Data Size: 512 bytes - Relative Performance: 0x2 Good (in use)"
        local format_num
        format_num=$(echo "$line" | awk '{print $3}' | tr -d ':')

        local data_size
        data_size=$(echo "$line" | grep -oP 'Data Size: \K[0-9]+')

        local in_use=""
        if echo "$line" | grep -q "(in use)"; then
            in_use="*"
        fi

        echo "${format_num}:${data_size}:${in_use}"
    done
}

################################################################################
# Check if NVMe supports 4K sectors
################################################################################
nvme_supports_4k() {
    local device="$1"

    local formats
    formats=$(get_nvme_lba_formats "$device")

    if [[ -z "$formats" ]]; then
        return 1
    fi

    # Check if any format has 4096 byte sectors
    echo "$formats" | grep -q ":4096:"
}

################################################################################
# Get current NVMe sector size
################################################################################
get_nvme_sector_size() {
    local device="$1"

    local formats
    formats=$(get_nvme_lba_formats "$device")

    if [[ -z "$formats" ]]; then
        echo "512"  # Default
        return
    fi

    # Find the format in use
    local current
    current=$(echo "$formats" | grep "\*$" | cut -d: -f2)

    if [[ -n "$current" ]]; then
        echo "$current"
    else
        echo "512"
    fi
}

################################################################################
# Set NVMe to 4K sectors (DESTRUCTIVE!)
################################################################################
set_nvme_4k_sectors() {
    local device="$1"
    local namespace="${device}n1"

    log_warn "Setting NVMe device $device to 4K sectors - THIS WILL DESTROY ALL DATA!"

    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would execute: nvme format /dev/${namespace} --lbaf=1"
        return 0
    fi

    # Check if device supports 4K
    if ! nvme_supports_4k "$device"; then
        log_error "Device $device does not support 4K sectors"
        return 1
    fi

    # Find the 4K format ID
    local formats
    formats=$(get_nvme_lba_formats "$device")

    local format_id
    format_id=$(echo "$formats" | grep ":4096:" | head -1 | cut -d: -f1)

    if [[ -z "$format_id" ]]; then
        log_error "Could not determine 4K format ID"
        return 1
    fi

    log_info "Formatting NVMe device with 4K sectors (format ID: $format_id)..."

    if [[ "$FORCE" != "true" ]]; then
        log_warn "This will DESTROY all data on $device!"
        if ! confirm "Proceed with NVMe format?"; then
            log_info "NVMe format cancelled"
            return 1
        fi
    fi

    # Format the device
    if nvme format "/dev/${namespace}" --lbaf="$format_id" 2>&1 | tee -a "$LOG_FILE"; then
        log_success "NVMe device formatted to 4K sectors"
        return 0
    else
        log_error "NVMe format failed"
        return 1
    fi
}

################################################################################
# Get SSD TRIM support via hdparm
################################################################################
check_trim_support() {
    local device="$1"

    if ! has_hdparm; then
        log_verbose "hdparm not available, skipping TRIM check"
        return 1
    fi

    local hdparm_output
    hdparm_output=$(hdparm -I "/dev/$device" 2>/dev/null)

    if echo "$hdparm_output" | grep -q "TRIM supported"; then
        log_info "Device $device supports TRIM"
        return 0
    else
        log_verbose "Device $device does not support TRIM (or not an SSD)"
        return 1
    fi
}

################################################################################
# Check SMART health
################################################################################
check_smart_health() {
    local device="$1"

    if ! has_smartctl; then
        log_verbose "smartctl not available, skipping SMART check"
        return 0  # Don't fail if smartctl not available
    fi

    log_info "Checking SMART health for $device..."

    local smart_output
    smart_output=$(smartctl -H "/dev/$device" 2>&1)

    if echo "$smart_output" | grep -q "PASSED"; then
        log_success "SMART health check PASSED for $device"
        return 0
    elif echo "$smart_output" | grep -q "FAILED"; then
        log_error "SMART health check FAILED for $device!"
        log_error "This device may be failing - use with caution"
        return 1
    else
        log_warn "SMART status unknown for $device"
        return 0  # Continue anyway
    fi
}

################################################################################
# Calculate optimal ashift for device
################################################################################
calculate_optimal_ashift() {
    local device="$1"

    # Check if NVMe
    if [[ "$device" =~ ^nvme ]]; then
        local sector_size
        sector_size=$(get_nvme_sector_size "$device")

        case "$sector_size" in
            512)
                echo "9"
                ;;
            4096)
                echo "12"
                ;;
            8192)
                echo "13"
                ;;
            *)
                log_warn "Unknown sector size: $sector_size, defaulting to ashift=12"
                echo "12"
                ;;
        esac
        return 0
    fi

    # For non-NVMe, check /sys
    local phys_bs
    if [[ -f "/sys/block/${device}/queue/physical_block_size" ]]; then
        phys_bs=$(cat "/sys/block/${device}/queue/physical_block_size")

        case "$phys_bs" in
            512)
                echo "9"
                ;;
            4096)
                echo "12"
                ;;
            8192)
                echo "13"
                ;;
            *)
                log_warn "Unknown physical block size: $phys_bs, defaulting to ashift=12"
                echo "12"
                ;;
        esac
    else
        log_warn "Cannot determine physical block size, defaulting to ashift=12"
        echo "12"
    fi
}

################################################################################
# Comprehensive device fitness check
################################################################################
check_device_fitness() {
    local device="$1"

    log_step "Checking fitness of device: $device"

    local errors=0

    # Check device exists
    if [[ ! -b "/dev/$device" ]]; then
        log_error "Device /dev/$device does not exist"
        return 1
    fi

    # Check if device is mounted (source system check)
    if is_device_mounted "$device"; then
        log_error "Device $device has mounted partitions - may be source system!"
        log_error "Refusing to proceed to protect running system"
        errors=$((errors + 1))
    fi

    # Check for partitions mounted from this device
    if [[ -f /proc/mounts ]]; then
        if grep -q "^/dev/${device}" /proc/mounts; then
            log_error "Device $device has mounted partitions:"
            grep "^/dev/${device}" /proc/mounts | while read -r line; do
                log_error "  $line"
            done
            errors=$((errors + 1))
        fi
    fi

    # Check if part of MD RAID
    if [[ -f /proc/mdstat ]]; then
        if grep -q "$device" /proc/mdstat; then
            log_warn "Device $device appears in /proc/mdstat"
            log_warn "Device may be part of MD RAID array"
            errors=$((errors + 1))
        fi
    fi

    # Check SMART health
    if ! check_smart_health "$device"; then
        log_warn "SMART health check failed or inconclusive"
        # Don't fail on SMART issues, just warn
    fi

    # Check minimum size (8GB)
    local size_bytes
    size_bytes=$(get_block_device_size "$device")

    local min_size=$((8 * 1024 * 1024 * 1024))
    if [[ $size_bytes -lt $min_size ]]; then
        log_error "Device is too small (minimum 8GB required)"
        errors=$((errors + 1))
    fi

    # Check if NVMe and suggest 4K sectors
    if [[ "$device" =~ ^nvme ]]; then
        if nvme_supports_4k "$device"; then
            local current_size
            current_size=$(get_nvme_sector_size "$device")

            if [[ "$current_size" != "4096" ]]; then
                log_warn "NVMe device supports 4K sectors but is using ${current_size}-byte sectors"
                log_info "Consider formatting to 4K for optimal ZFS performance"
                log_info "Use: --nvme-format-4k flag (WILL DESTROY DATA!)"
            else
                log_success "NVMe device already using 4K sectors"
            fi
        fi
    fi

    # Check TRIM support for SSDs
    if ! is_rotational_device "$device"; then
        check_trim_support "$device"
    fi

    if [[ $errors -gt 0 ]]; then
        log_error "Device fitness check failed with $errors error(s)"
        return 1
    fi

    log_success "Device $device passed fitness checks"
    return 0
}

################################################################################
# Optimize device parameters
################################################################################
optimize_device_parameters() {
    local device="$1"

    log_info "Optimizing parameters for $device..."

    # Check if SSD and enable discard
    if ! is_rotational_device "$device"; then
        log_info "Device is SSD/NVMe, discard will be enabled in ZFS"
    fi

    # Calculate optimal ashift
    local ashift
    ashift=$(calculate_optimal_ashift "$device")
    log_info "Calculated optimal ashift: $ashift"

    # Store for later use
    CALCULATED_ASHIFT="$ashift"
}

################################################################################
# Verify device is not part of source system
################################################################################
verify_not_source_device() {
    local device="$1"

    # Get root device
    local root_device
    root_device=$(get_current_root)

    # Extract just the device name (remove partition number)
    local root_base_device
    root_base_device=$(echo "$root_device" | sed 's/[0-9]*$//' | sed 's/p$//')

    if [[ "/dev/$device" == "$root_base_device" ]] || [[ "$device" == "$(basename "$root_base_device")" ]]; then
        log_error "Device $device appears to be the source system device!"
        log_error "Root is mounted from: $root_device"
        log_error "Refusing to destroy source system"
        return 1
    fi

    log_success "Device $device is not the source system device"
    return 0
}
