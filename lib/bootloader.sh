#!/bin/bash
################################################################################
# Bootloader and ZFSBootMenu installation functions
################################################################################

################################################################################
# Install ZFSBootMenu based on distribution
################################################################################

install_zfsbootmenu() {
    local distro="$1"
    local pool_name="$2"
    
    log_info "Installing ZFSBootMenu for $distro..."
    
    # Install required packages
    install_zfsbootmenu_packages "$distro"
    
    # Create ZFSBootMenu configuration
    create_zfsbootmenu_config "$pool_name"
    
    # Generate ZFSBootMenu image
    generate_zfsbootmenu_image
    
    log_success "ZFSBootMenu installed successfully"
}

################################################################################
# Install ZFSBootMenu packages
################################################################################

install_zfsbootmenu_packages() {
    local distro="$1"
    
    log_info "Installing ZFSBootMenu packages..."
    
    case "$distro" in
        fedora)
            # Install required dependencies
            install_package "fzf"
            install_package "kexec-tools"
            install_package "cpanminus"
            install_package "perl-Config-IniFiles"
            
            # Install ZFSBootMenu from GitHub or COPR
            if ! command_exists generate-zbm; then
                log_info "Installing ZFSBootMenu from source..."
                execute_cmd "dnf copr enable -y zbm/zfsbootmenu" || {
                    log_warn "COPR repository not available, attempting manual installation"
                    install_zfsbootmenu_manual
                    return
                }
                install_package "zfsbootmenu"
            fi
            ;;
        debian|mx)
            # Install required dependencies
            install_package "fzf"
            install_package "kexec-tools"
            install_package "cpanminus"
            install_package "libconfig-inifiles-perl"
            install_package "libsort-versions-perl"
            install_package "libboolean-perl"
            install_package "libyaml-pp-perl"
            
            # Install ZFSBootMenu
            if ! command_exists generate-zbm; then
                log_info "Installing ZFSBootMenu manually..."
                install_zfsbootmenu_manual
            fi
            ;;
        *)
            log_warn "Installing ZFSBootMenu manually for unsupported distribution"
            install_zfsbootmenu_manual
            ;;
    esac
    
    log_success "ZFSBootMenu packages installed"
}

################################################################################
# Install ZFSBootMenu manually from GitHub
################################################################################

install_zfsbootmenu_manual() {
    local install_dir="/usr/local/src/zfsbootmenu"
    
    log_info "Installing ZFSBootMenu manually from GitHub..."
    
    # Install required tools
    if ! command_exists git; then
        case "$DETECTED_DISTRO" in
            fedora)
                install_package "git"
                ;;
            debian|mx)
                install_package "git"
                ;;
        esac
    fi
    
    # Clone or update ZFSBootMenu repository
    if [[ -d "$install_dir" ]]; then
        log_info "Updating existing ZFSBootMenu repository..."
        execute_cmd "cd $install_dir && git pull"
    else
        log_info "Cloning ZFSBootMenu repository..."
        execute_cmd "git clone https://github.com/zbm-dev/zfsbootmenu.git $install_dir"
    fi
    
    # Install ZFSBootMenu
    execute_cmd "cd $install_dir && make install"
    
    log_success "ZFSBootMenu manual installation completed"
}

################################################################################
# Create ZFSBootMenu configuration
################################################################################

create_zfsbootmenu_config() {
    local pool_name="$1"
    local config_dir="/etc/zfsbootmenu"
    local config_file="$config_dir/config.yaml"
    
    log_info "Creating ZFSBootMenu configuration..."
    
    # Create config directory
    execute_cmd "mkdir -p $config_dir"
    
    # Create configuration file
    cat > "$config_file" << EOF
Global:
  ManageImages: true
  BootMountPoint: /boot/efi
  DracutConfDir: /etc/zfsbootmenu/dracut.conf.d
  PreHooksDir: /etc/zfsbootmenu/hooks/pre.d
  PostHooksDir: /etc/zfsbootmenu/hooks/post.d
  InitCPIO: false
  InitCPIOConfig: /etc/zfsbootmenu/mkinitcpio.conf

Components:
  ImageDir: /boot/efi/EFI/zbm
  Versions: 3
  Enabled: true
  syslinux:
    Config: /boot/efi/syslinux/syslinux.cfg
    Enabled: false

EFI:
  ImageDir: /boot/efi/EFI/zbm
  Versions: 3
  Enabled: true

Kernel:
  CommandLine: ro quiet loglevel=3

EOF

    if [[ "$DRY_RUN" == "false" ]]; then
        log_info "ZFSBootMenu configuration created at $config_file"
        tee -a "$LOG_FILE" < "$config_file"
    fi
    
    # Create dracut configuration directory
    execute_cmd "mkdir -p $config_dir/dracut.conf.d"
    
    # Create dracut ZFS configuration
    cat > "$config_dir/dracut.conf.d/zfsbootmenu.conf" << EOF
# ZFSBootMenu dracut configuration
add_dracutmodules+=" zfsbootmenu "
omit_dracutmodules+=" btrfs "
install_items+=" /etc/zfs/zpool.cache "
zfsbootmenu_module_root="/usr/lib/dracut/modules.d/90zfsbootmenu"
zfsbootmenu_tmux=1
zfsbootmenu_teardown=1
EOF

    log_success "ZFSBootMenu configuration created"
}

################################################################################
# Generate ZFSBootMenu image
################################################################################

generate_zfsbootmenu_image() {
    log_info "Generating ZFSBootMenu image..."
    
    # Ensure EFI directory exists
    execute_cmd "mkdir -p /boot/efi/EFI/zbm"
    
    # Generate ZFSBootMenu
    if command_exists generate-zbm; then
        execute_cmd "generate-zbm --debug"
    else
        log_warn "generate-zbm not found, attempting manual image generation..."
        generate_zbm_manual
    fi
    
    # Verify image was created
    if [[ "$DRY_RUN" == "false" ]] && [[ -d /boot/efi/EFI/zbm ]]; then
        log_info "ZFSBootMenu images:"
        find /boot/efi/EFI/zbm/ -type f -exec ls -lh {} \; | tee -a "$LOG_FILE" || true
    fi
    
    log_success "ZFSBootMenu image generated"
}

################################################################################
# Generate ZFSBootMenu manually using dracut
################################################################################

generate_zbm_manual() {
    local kernel_version
    kernel_version=$(uname -r)
    
    log_info "Generating ZFSBootMenu manually with dracut..."
    
    # Install dracut if not present
    if ! command_exists dracut; then
        case "$DETECTED_DISTRO" in
            fedora)
                install_package "dracut"
                ;;
            debian|mx)
                install_package "dracut"
                ;;
        esac
    fi
    
    # Generate initramfs with ZFS support
    execute_cmd "dracut --force --kver $kernel_version --add zfs /boot/efi/EFI/zbm/vmlinuz-$kernel_version"
    
    log_success "Manual ZFSBootMenu generation completed"
}

################################################################################
# Configure bootloader (systemd-boot or rEFInd)
################################################################################

configure_bootloader() {
    local drives=("$@")
    
    log_info "Configuring bootloader..."
    
    # Format and mount EFI partitions
    format_efi_partitions "${drives[@]}"
    mount_efi_partition "${drives[0]}"
    
    # Install bootloader (prefer systemd-boot)
    if command_exists bootctl; then
        install_systemd_boot "${drives[@]}"
    else
        log_warn "systemd-boot not available, installing rEFInd..."
        install_refind "${drives[@]}"
    fi
    
    # Setup swap if configured
    setup_swap "${drives[@]}"
    
    log_success "Bootloader configuration completed"
}

################################################################################
# Mount EFI partition
################################################################################

mount_efi_partition() {
    local drive="$1"
    local efi_partition
    
    # Determine partition naming scheme
    if [[ "$drive" =~ nvme ]]; then
        efi_partition="/dev/${drive}p1"
    else
        efi_partition="/dev/${drive}1"
    fi
    
    log_info "Mounting EFI partition: $efi_partition"
    
    execute_cmd "mkdir -p /boot/efi"
    execute_cmd "mount $efi_partition /boot/efi"
    
    log_success "EFI partition mounted at /boot/efi"
}

################################################################################
# Install systemd-boot
################################################################################

install_systemd_boot() {
    local drives=("$@")
    
    log_info "Installing systemd-boot..."
    
    # Install systemd-boot to EFI partition
    execute_cmd "bootctl install"
    
    # Create boot entry for ZFSBootMenu
    local entry_file="/boot/efi/loader/entries/zfsbootmenu.conf"
    
    execute_cmd "mkdir -p /boot/efi/loader/entries"
    
    cat > "$entry_file" << EOF
title   ZFSBootMenu
linux   /EFI/zbm/vmlinuz-bootmenu
initrd  /EFI/zbm/initramfs-bootmenu.img
options zfsbootmenu quiet loglevel=4
EOF

    # Configure loader
    cat > "/boot/efi/loader/loader.conf" << EOF
default zfsbootmenu
timeout 5
console-mode max
editor no
EOF

    log_success "systemd-boot installed"
}

################################################################################
# Install rEFInd bootloader
################################################################################

install_refind() {
    local drives=("$@")
    
    log_info "Installing rEFInd bootloader..."
    
    # Install rEFInd package
    case "$DETECTED_DISTRO" in
        fedora)
            install_package "refind"
            ;;
        debian|mx)
            install_package "refind"
            ;;
    esac
    
    # Install rEFInd to EFI partition
    if command_exists refind-install; then
        execute_cmd "refind-install"
    else
        log_error "refind-install command not found"
        return 1
    fi
    
    # Configure rEFInd for ZFSBootMenu
    local refind_conf="/boot/efi/EFI/refind/refind.conf"
    
    if [[ -f "$refind_conf" ]]; then
        # Add ZFSBootMenu entry
        cat >> "$refind_conf" << EOF

# ZFSBootMenu entry
menuentry "ZFSBootMenu" {
    loader /EFI/zbm/vmlinuz-bootmenu
    initrd /EFI/zbm/initramfs-bootmenu.img
    options "zfsbootmenu quiet loglevel=4"
}
EOF
    fi
    
    log_success "rEFInd installed"
}

################################################################################
# Finalize installation
################################################################################

finalize_installation() {
    local pool_name="$1"
    
    log_info "Finalizing installation..."
    
    # Set boot properties
    set_boot_properties "$pool_name"
    
    # Create initial snapshot
    create_snapshot "$pool_name/ROOT/default" "initial-install"
    
    # Generate zpool.cache
    execute_cmd "mkdir -p /etc/zfs"
    execute_cmd "zpool set cachefile=/etc/zfs/zpool.cache $pool_name"
    
    # Update /etc/fstab for EFI partition
    # Note: DRIVES is a global array from the main script
    # shellcheck disable=SC2153
    if [[ "$INSTALL_MODE" == "new" ]]; then
        log_info "Updating /etc/fstab..."
        local first_drive="${DRIVES[0]}"
        local efi_partition
        
        if [[ "$first_drive" =~ nvme ]]; then
            efi_partition="/dev/${first_drive}p1"
        else
            efi_partition="/dev/${first_drive}1"
        fi
        
        # Backup existing fstab if it exists
        if [[ -f /etc/fstab ]]; then
            execute_cmd "cp /etc/fstab /etc/fstab.bak"
        fi
        
        # Add EFI mount
        echo "$efi_partition /boot/efi vfat defaults 0 1" >> /etc/fstab || true
    fi
    
    # Unmount EFI partition
    execute_cmd "umount /boot/efi" || true
    
    # Check pool health one final time
    check_pool_health "$pool_name"
    
    log_success "Installation finalized"
}
