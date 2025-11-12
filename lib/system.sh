#!/bin/bash
################################################################################
# System copy and migration functions for existing system installations
################################################################################

################################################################################
# Detect current system root from /proc
################################################################################
get_current_root() {
    # Get the root device from /proc/mounts
    if [[ -f /proc/mounts ]]; then
        grep " / " /proc/mounts | head -1 | awk '{print $1}'
    else
        echo "/"
    fi
}

################################################################################
# Calculate directory size using du
################################################################################
get_directory_size() {
    local dir="$1"

    if [[ -d "$dir" ]]; then
        # Use du to get size in bytes
        du -sb "$dir" 2>/dev/null | awk '{print $1}'
    else
        echo "0"
    fi
}

################################################################################
# Get filesystem usage from /proc/mounts and df
################################################################################
get_filesystem_usage() {
    local mount_point="${1:-/}"

    # Get total and used space in bytes
    df -B1 "$mount_point" 2>/dev/null | tail -1 | awk '{print $2, $3, $4}'
}

################################################################################
# Validate source and target are different
################################################################################
validate_source_target() {
    local source="$1"
    local target="$2"

    log_info "Validating source and target are different..."

    # Get real paths
    local source_real target_real
    source_real=$(readlink -f "$source" 2>/dev/null || echo "$source")
    target_real=$(readlink -f "$target" 2>/dev/null || echo "$target")

    if [[ "$source_real" == "$target_real" ]]; then
        log_error "Source and target are the same: $source_real"
        return 1
    fi

    # Check if target is a subdirectory of source
    if [[ "$target_real" == "$source_real"* ]]; then
        log_error "Target is a subdirectory of source (would cause recursion)"
        return 1
    fi

    log_success "Source and target validation passed"
    return 0
}

################################################################################
# Estimate space required for copy
################################################################################
estimate_copy_size() {
    local source="${1:-/}"
    local exclude_paths=("${@:2}")

    log_info "Estimating space required for copy..."
    log_verbose "Source: $source"

    # Build du exclusion arguments
    local du_excludes=()
    for exclude in "${exclude_paths[@]}"; do
        du_excludes+=(--exclude="$exclude")
    done

    # Calculate size with exclusions
    local size_bytes
    size_bytes=$(du -sb "${du_excludes[@]}" "$source" 2>/dev/null | awk '{print $1}')

    if [[ -z "$size_bytes" ]] || [[ "$size_bytes" -eq 0 ]]; then
        log_warn "Could not estimate size, using filesystem usage"
        local fs_stats
        fs_stats=$(get_filesystem_usage "$source")
        size_bytes=$(echo "$fs_stats" | awk '{print $2}')
    fi

    local size_gb=$((size_bytes / 1024 / 1024 / 1024))
    log_info "Estimated size: ${size_gb} GB (${size_bytes} bytes)"

    echo "$size_bytes"
}

################################################################################
# Check if target has sufficient space
################################################################################
check_target_space() {
    local target="$1"
    local required_bytes="$2"

    log_info "Checking target has sufficient space..."

    # Get available space on target
    local fs_stats available_bytes
    fs_stats=$(get_filesystem_usage "$target")
    available_bytes=$(echo "$fs_stats" | awk '{print $3}')

    local available_gb=$((available_bytes / 1024 / 1024 / 1024))
    local required_gb=$((required_bytes / 1024 / 1024 / 1024))

    log_info "Available space: ${available_gb} GB"
    log_info "Required space: ${required_gb} GB"

    # Add 10% buffer for safety
    local required_with_buffer=$((required_bytes * 110 / 100))

    if [[ $available_bytes -lt $required_with_buffer ]]; then
        log_error "Insufficient space on target"
        log_error "Required (with 10% buffer): $((required_with_buffer / 1024 / 1024 / 1024)) GB"
        log_error "Available: ${available_gb} GB"
        return 1
    fi

    log_success "Target has sufficient space"
    return 0
}

################################################################################
# Build default exclusion list
################################################################################
get_default_exclusions() {
    # Return array of default paths to exclude
    cat <<'EOF'
/dev/*
/proc/*
/sys/*
/tmp/*
/run/*
/mnt/*
/media/*
/lost+found
/boot/efi/*
/var/tmp/*
/var/cache/apt/*
/var/cache/dnf/*
/var/cache/yum/*
/var/lib/docker/overlay2/*
/var/lib/docker/containers/*
/var/log/journal/*
/.snapshots/*
/swap.img
/swapfile
*.tmp
*.cache
EOF
}

################################################################################
# Detect services that should be stopped
################################################################################
detect_running_services() {
    log_info "Checking for services that may interfere with copy..."

    local problematic_services=(
        "docker"
        "containerd"
        "mysql"
        "mariadb"
        "postgresql"
        "mongodb"
        "redis"
    )

    local running=()

    for service in "${problematic_services[@]}"; do
        if systemctl is-active --quiet "$service" 2>/dev/null; then
            running+=("$service")
            log_warn "Service is running: $service"
        fi
    done

    if [[ ${#running[@]} -gt 0 ]]; then
        log_warn "The following services are running and may cause issues:"
        for svc in "${running[@]}"; do
            log_warn "  - $svc"
        done
        log_warn "Consider stopping these services before copying"

        if [[ "$FORCE" != "true" ]]; then
            if ! confirm "Continue anyway?"; then
                return 1
            fi
        fi
    else
        log_success "No problematic services detected"
    fi

    return 0
}

################################################################################
# Validate source is a bootable system
################################################################################
validate_bootable_system() {
    local source="${1:-/}"

    log_info "Validating source is a bootable system..."

    local errors=0

    # Check for essential directories
    local essential_dirs=("/etc" "/bin" "/sbin" "/usr" "/var" "/lib")
    for dir in "${essential_dirs[@]}"; do
        local full_path="${source}${dir}"
        if [[ ! -d "$full_path" ]]; then
            log_error "Missing essential directory: $dir"
            errors=$((errors + 1))
        fi
    done

    # Check for init system
    if [[ ! -f "${source}/sbin/init" ]] && [[ ! -L "${source}/sbin/init" ]]; then
        log_error "Missing /sbin/init - system may not be bootable"
        errors=$((errors + 1))
    fi

    # Check for kernel
    if [[ ! -d "${source}/boot" ]] && [[ "$source" == "/" ]]; then
        log_warn "No /boot directory found"
    fi

    # Check for /etc/fstab
    if [[ ! -f "${source}/etc/fstab" ]]; then
        log_warn "No /etc/fstab found (will be created)"
    fi

    if [[ $errors -gt 0 ]]; then
        log_error "Source validation failed with $errors error(s)"
        return 1
    fi

    log_success "Source appears to be a valid bootable system"
    return 0
}

################################################################################
# Copy system with rsync
################################################################################
copy_system() {
    local source="${1:-/}"
    local target="$2"
    local exclude_paths=("${@:3}")

    log_step "Copying system from $source to $target"

    # Validate inputs
    if [[ -z "$target" ]]; then
        log_error "Target directory not specified"
        return 1
    fi

    if [[ ! -d "$target" ]]; then
        log_error "Target directory does not exist: $target"
        return 1
    fi

    # Validate source and target are different
    if ! validate_source_target "$source" "$target"; then
        return 1
    fi

    # Validate source is bootable
    if ! validate_bootable_system "$source"; then
        if [[ "$FORCE" != "true" ]]; then
            return 1
        else
            log_warn "Continuing anyway (--force specified)"
        fi
    fi

    # Build rsync command
    local rsync_cmd="rsync -aAXHv"

    # Add exclusions
    local default_exclusions
    mapfile -t default_exclusions < <(get_default_exclusions)

    for exclude in "${default_exclusions[@]}"; do
        rsync_cmd+=" --exclude='$exclude'"
    done

    # Add custom exclusions
    for exclude in "${exclude_paths[@]}"; do
        rsync_cmd+=" --exclude='$exclude'"
    done

    # Add progress reporting
    rsync_cmd+=" --info=progress2"
    rsync_cmd+=" --stats"

    # Source and target
    rsync_cmd+=" ${source}/ ${target}/"

    log_info "Starting system copy..."
    log_info "This may take a long time depending on data size..."

    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would execute: rsync --dry-run ${rsync_cmd#rsync }"
        execute_cmd "rsync --dry-run ${rsync_cmd#rsync }"
    else
        execute_cmd "$rsync_cmd"
    fi

    if [[ $? -eq 0 ]]; then
        log_success "System copy completed successfully"
    else
        log_error "System copy failed"
        return 1
    fi

    return 0
}

################################################################################
# Post-copy configuration adjustments
################################################################################
post_copy_configuration() {
    local target="$1"

    log_step "Performing post-copy configuration"

    # Generate new machine-id if it exists
    if [[ -f "${target}/etc/machine-id" ]]; then
        log_info "Clearing machine-id (will regenerate on first boot)"
        if [[ "$DRY_RUN" != "true" ]]; then
            : > "${target}/etc/machine-id"
        fi
    fi

    # Create /etc/machine-id for systemd if missing
    if [[ ! -f "${target}/etc/machine-id" ]]; then
        log_info "Creating empty machine-id"
        if [[ "$DRY_RUN" != "true" ]]; then
            touch "${target}/etc/machine-id"
        fi
    fi

    # Set hostname if specified
    if [[ -n "$HOSTNAME" ]]; then
        log_info "Setting hostname to: $HOSTNAME"
        if [[ "$DRY_RUN" != "true" ]]; then
            echo "$HOSTNAME" > "${target}/etc/hostname"
        fi
    fi

    # Backup original fstab if it exists
    if [[ -f "${target}/etc/fstab" ]]; then
        log_info "Backing up original fstab"
        if [[ "$DRY_RUN" != "true" ]]; then
            cp "${target}/etc/fstab" "${target}/etc/fstab.pre-zbm-backup"
        fi
    fi

    # Clear SSH host keys (will regenerate on first boot)
    if [[ -d "${target}/etc/ssh" ]]; then
        log_info "Removing SSH host keys (will regenerate on first boot)"
        if [[ "$DRY_RUN" != "true" ]]; then
            rm -f "${target}"/etc/ssh/ssh_host_*_key*
        fi
    fi

    # Zap all network configuration to ensure clean network identity
    # This includes NetworkManager, systemd-networkd, netplan, wicked,
    # DHCP leases, persistent rules, hostname, cloud-init, and firewall rules
    log_info "Zapping network configuration for clean identity..."
    zap_all_network_config "$target"

    log_success "Post-copy configuration completed"
    return 0
}

################################################################################
# Main function to copy existing system
################################################################################
copy_existing_system() {
    local source="${SOURCE_ROOT:-/}"
    local target="/mnt"
    local exclude_paths=("${EXCLUDE_PATHS[@]}")

    log_step "Copying Existing System"

    # Estimate size
    local estimated_size
    estimated_size=$(estimate_copy_size "$source" "${exclude_paths[@]}")

    # Check target space
    if ! check_target_space "$target" "$estimated_size"; then
        if [[ "$FORCE" != "true" ]]; then
            return 1
        else
            log_warn "Continuing anyway (--force specified)"
        fi
    fi

    # Detect running services
    if ! detect_running_services; then
        return 1
    fi

    # Confirm before copying
    if [[ "$FORCE" != "true" ]]; then
        log_warn "About to copy system from $source to $target"
        if ! confirm "Proceed with system copy?"; then
            log_info "System copy cancelled by user"
            return 1
        fi
    fi

    # Perform the copy
    if ! copy_system "$source" "$target" "${exclude_paths[@]}"; then
        return 1
    fi

    # Post-copy configuration
    if ! post_copy_configuration "$target"; then
        log_warn "Post-copy configuration had issues"
    fi

    log_success "Existing system copy completed successfully"
    return 0
}

################################################################################
# Display what will be copied (for dry-run/verbose)
################################################################################
display_copy_summary() {
    local source="${SOURCE_ROOT:-/}"
    local target="/mnt"

    cat << EOF

================================================================================
                        System Copy Summary
================================================================================

Source:             $source
Target:             $target
Copy Method:        rsync with archive mode
Preserve:           Permissions, ACLs, Extended Attributes, Hard Links

Default Exclusions:
EOF

    local default_exclusions
    mapfile -t default_exclusions < <(get_default_exclusions)
    for exclude in "${default_exclusions[@]}"; do
        echo "  - $exclude"
    done

    if [[ ${#EXCLUDE_PATHS[@]} -gt 0 ]]; then
        echo ""
        echo "Custom Exclusions:"
        for exclude in "${EXCLUDE_PATHS[@]}"; do
            echo "  - $exclude"
        done
    fi

    echo ""
    echo "Post-Copy Actions:"
    echo "  - Clear machine-id"
    echo "  - Remove SSH host keys"
    echo "  - Clear network persistent rules"
    if [[ -n "$HOSTNAME" ]]; then
        echo "  - Set hostname to: $HOSTNAME"
    fi

    echo ""
}
