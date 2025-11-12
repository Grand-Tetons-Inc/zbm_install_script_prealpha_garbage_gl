#!/bin/bash
################################################################################
# Network Configuration Zapping
#
# Removes all network configuration that would make the migrated system
# believe it is the old system. Ensures clean network setup on first boot.
################################################################################

################################################################################
# Zap NetworkManager connections
################################################################################
zap_networkmanager() {
    local target="$1"

    log_info "Zapping NetworkManager connections..."

    local nm_dir="${target}/etc/NetworkManager/system-connections"

    if [[ -d "$nm_dir" ]]; then
        log_verbose "Removing NetworkManager connections from $nm_dir"

        if [[ "$DRY_RUN" != "true" ]]; then
            # Backup before removing
            if [[ "$BACKUP_CONFIG" == "true" ]]; then
                local backup_dir="${nm_dir}.pre-zbm-backup"
                mkdir -p "$backup_dir"
                cp -a "$nm_dir"/* "$backup_dir/" 2>/dev/null || true
                log_info "Backed up NetworkManager connections to $backup_dir"
            fi

            # Remove all connection files
            rm -f "$nm_dir"/* 2>/dev/null || true
            log_success "NetworkManager connections removed"
        else
            log_info "[DRY RUN] Would remove: $nm_dir/*"
        fi
    else
        log_verbose "NetworkManager not configured or not present"
    fi

    # Also clear NetworkManager state
    local nm_state_dir="${target}/var/lib/NetworkManager"
    if [[ -d "$nm_state_dir" ]]; then
        log_verbose "Clearing NetworkManager state"

        if [[ "$DRY_RUN" != "true" ]]; then
            rm -rf "${nm_state_dir}"/* 2>/dev/null || true
        else
            log_info "[DRY RUN] Would remove: ${nm_state_dir}/*"
        fi
    fi
}

################################################################################
# Zap systemd-networkd configuration
################################################################################
zap_systemd_network() {
    local target="$1"

    log_info "Zapping systemd-networkd configuration..."

    local network_dir="${target}/etc/systemd/network"

    if [[ -d "$network_dir" ]]; then
        log_verbose "Removing systemd-networkd configs from $network_dir"

        if [[ "$DRY_RUN" != "true" ]]; then
            # Backup
            if [[ "$BACKUP_CONFIG" == "true" ]]; then
                local backup_dir="${network_dir}.pre-zbm-backup"
                mkdir -p "$backup_dir"
                cp -a "$network_dir"/* "$backup_dir/" 2>/dev/null || true
                log_info "Backed up systemd-networkd configs to $backup_dir"
            fi

            # Remove config files
            rm -f "$network_dir"/*.network 2>/dev/null || true
            rm -f "$network_dir"/*.link 2>/dev/null || true
            log_success "systemd-networkd configuration removed"
        else
            log_info "[DRY RUN] Would remove: $network_dir/*.{network,link}"
        fi
    else
        log_verbose "systemd-networkd not configured or not present"
    fi
}

################################################################################
# Zap netplan configuration
################################################################################
zap_netplan() {
    local target="$1"

    log_info "Zapping netplan configuration..."

    local netplan_dir="${target}/etc/netplan"

    if [[ -d "$netplan_dir" ]]; then
        log_verbose "Removing netplan configs from $netplan_dir"

        if [[ "$DRY_RUN" != "true" ]]; then
            # Backup
            if [[ "$BACKUP_CONFIG" == "true" ]]; then
                local backup_dir="${netplan_dir}.pre-zbm-backup"
                mkdir -p "$backup_dir"
                cp -a "$netplan_dir"/* "$backup_dir/" 2>/dev/null || true
                log_info "Backed up netplan configs to $backup_dir"
            fi

            # Remove YAML files
            rm -f "$netplan_dir"/*.yaml 2>/dev/null || true
            log_success "Netplan configuration removed"
        else
            log_info "[DRY RUN] Would remove: $netplan_dir/*.yaml"
        fi
    else
        log_verbose "Netplan not configured or not present"
    fi
}

################################################################################
# Zap wicked/SUSE network configuration
################################################################################
zap_wicked() {
    local target="$1"

    log_info "Zapping wicked/SUSE network configuration..."

    local network_dir="${target}/etc/sysconfig/network"

    if [[ -d "$network_dir" ]]; then
        log_verbose "Removing wicked configs from $network_dir"

        if [[ "$DRY_RUN" != "true" ]]; then
            # Backup
            if [[ "$BACKUP_CONFIG" == "true" ]]; then
                local backup_dir="${network_dir}.pre-zbm-backup"
                mkdir -p "$backup_dir"
                cp -a "$network_dir"/* "$backup_dir/" 2>/dev/null || true
                log_info "Backed up wicked configs to $backup_dir"
            fi

            # Remove interface configs
            rm -f "$network_dir"/ifcfg-* 2>/dev/null || true
            log_success "Wicked configuration removed"
        else
            log_info "[DRY RUN] Would remove: $network_dir/ifcfg-*"
        fi
    else
        log_verbose "Wicked not configured or not present"
    fi
}

################################################################################
# Clear DHCP leases
################################################################################
clear_dhcp_leases() {
    local target="$1"

    log_info "Clearing DHCP leases..."

    local dhcp_dirs=(
        "${target}/var/lib/dhcp"
        "${target}/var/lib/dhclient"
        "${target}/var/lib/NetworkManager"
    )

    for dir in "${dhcp_dirs[@]}"; do
        if [[ -d "$dir" ]]; then
            log_verbose "Clearing leases in $dir"

            if [[ "$DRY_RUN" != "true" ]]; then
                rm -f "$dir"/*.lease* 2>/dev/null || true
                rm -f "$dir"/*.leases 2>/dev/null || true
            else
                log_info "[DRY RUN] Would remove leases from: $dir"
            fi
        fi
    done

    log_success "DHCP leases cleared"
}

################################################################################
# Clear persistent network interface naming rules
################################################################################
clear_persistent_net_rules() {
    local target="$1"

    log_info "Clearing persistent network rules..."

    local rules_files=(
        "${target}/etc/udev/rules.d/70-persistent-net.rules"
        "${target}/etc/udev/rules.d/75-persistent-net-generator.rules"
        "${target}/lib/udev/rules.d/75-persistent-net-generator.rules"
    )

    for rules_file in "${rules_files[@]}"; do
        if [[ -f "$rules_file" ]]; then
            log_verbose "Removing: $rules_file"

            if [[ "$DRY_RUN" != "true" ]]; then
                rm -f "$rules_file"
            else
                log_info "[DRY RUN] Would remove: $rules_file"
            fi
        fi
    done

    log_success "Persistent network rules cleared"
}

################################################################################
# Clear hostname files
################################################################################
clear_hostname_files() {
    local target="$1"
    local new_hostname="${HOSTNAME}"

    log_info "Clearing hostname configuration..."

    # Handle /etc/hostname
    local hostname_file="${target}/etc/hostname"
    if [[ -f "$hostname_file" ]]; then
        if [[ -n "$new_hostname" ]]; then
            log_info "Setting hostname to: $new_hostname"
            if [[ "$DRY_RUN" != "true" ]]; then
                echo "$new_hostname" > "$hostname_file"
            fi
        else
            log_info "Clearing /etc/hostname (will regenerate on boot)"
            if [[ "$DRY_RUN" != "true" ]]; then
                echo "" > "$hostname_file"
            fi
        fi
    fi

    # Handle /etc/hosts - remove old hostname references
    local hosts_file="${target}/etc/hosts"
    if [[ -f "$hosts_file" ]]; then
        log_verbose "Backing up /etc/hosts"
        if [[ "$DRY_RUN" != "true" ]] && [[ "$BACKUP_CONFIG" == "true" ]]; then
            cp "$hosts_file" "${hosts_file}.pre-zbm-backup"
        fi

        # Remove old hostname entries (127.0.1.1 lines typically)
        # Keep only localhost entries
        if [[ "$DRY_RUN" != "true" ]]; then
            grep -v "^127.0.1.1" "$hosts_file" > "${hosts_file}.tmp" || true
            mv "${hosts_file}.tmp" "$hosts_file"

            # Add new hostname if specified
            if [[ -n "$new_hostname" ]]; then
                echo "127.0.1.1	$new_hostname" >> "$hosts_file"
            fi
        fi
    fi

    log_success "Hostname configuration updated"
}

################################################################################
# Zap cloud-init instance data
################################################################################
zap_cloud_init() {
    local target="$1"

    log_info "Zapping cloud-init instance data..."

    local cloud_init_dirs=(
        "${target}/var/lib/cloud/instances"
        "${target}/var/lib/cloud/instance"
        "${target}/var/lib/cloud/data"
    )

    for dir in "${cloud_init_dirs[@]}"; do
        if [[ -d "$dir" ]]; then
            log_verbose "Removing: $dir"

            if [[ "$DRY_RUN" != "true" ]]; then
                rm -rf "$dir" 2>/dev/null || true
            else
                log_info "[DRY RUN] Would remove: $dir"
            fi
        fi
    done

    # Also remove cloud-init semaphore files
    local sem_dir="${target}/var/lib/cloud/sem"
    if [[ -d "$sem_dir" ]]; then
        if [[ "$DRY_RUN" != "true" ]]; then
            rm -rf "$sem_dir" 2>/dev/null || true
        fi
    fi

    log_success "Cloud-init instance data cleared"
}

################################################################################
# Zap firewall rules referencing old IPs
################################################################################
zap_firewall_rules() {
    local target="$1"

    log_info "Clearing firewall rules..."

    # Clear iptables rules
    local iptables_file="${target}/etc/iptables/rules.v4"
    if [[ -f "$iptables_file" ]]; then
        log_verbose "Backing up iptables rules"
        if [[ "$DRY_RUN" != "true" ]] && [[ "$BACKUP_CONFIG" == "true" ]]; then
            cp "$iptables_file" "${iptables_file}.pre-zbm-backup"
        fi

        log_warn "Clearing iptables rules (will use defaults on boot)"
        if [[ "$DRY_RUN" != "true" ]]; then
            # Create minimal iptables config
            cat > "$iptables_file" <<'EOF'
*filter
:INPUT ACCEPT [0:0]
:FORWARD ACCEPT [0:0]
:OUTPUT ACCEPT [0:0]
COMMIT
EOF
        fi
    fi

    # Clear firewalld state
    local firewalld_dir="${target}/etc/firewalld"
    if [[ -d "$firewalld_dir" ]]; then
        log_verbose "Clearing firewalld state"
        # Don't remove configs, just clear runtime state
    fi

    log_success "Firewall rules cleared"
}

################################################################################
# Main network zapping function
################################################################################
zap_all_network_config() {
    local target="$1"

    log_step "Zapping Network Configuration"

    # Zap all network managers
    zap_networkmanager "$target"
    zap_systemd_network "$target"
    zap_netplan "$target"
    zap_wicked "$target"

    # Clear DHCP leases
    clear_dhcp_leases "$target"

    # Clear persistent rules
    clear_persistent_net_rules "$target"

    # Clear hostname
    clear_hostname_files "$target"

    # Clear cloud-init
    zap_cloud_init "$target"

    # Clear firewall rules
    zap_firewall_rules "$target"

    log_success "Network configuration completely zapped"
    log_info "System will get fresh network config on first boot"
}
