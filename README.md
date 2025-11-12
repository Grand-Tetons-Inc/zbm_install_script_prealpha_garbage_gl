# ZFSBootMenu Installation Script

A comprehensive BASH script solution for installing [ZFSBootMenu](https://zfsbootmenu.org/) on Linux systems with support for single or multiple drives in various RAID configurations.

## 🎨 NEW: Interactive TUI Available!

We now have TWO beautiful Text User Interfaces for guided installation:

### Dialog-based TUI (Works Everywhere)
```bash
sudo ./zbm-tui.sh
```

### Notcurses TUI (Advanced Graphics)
```bash
sudo ./zbm-notcurses.sh
```

**TUI Features:**
- Visual device selection with size/model info
- Step-by-step configuration wizard
- Real-time validation with color-coded checks
- Live installation progress with log output
- Bootloader selection (ZBM standalone, systemd-boot, rEFInd)
- Perfect for beginners and experts alike

See [TUI_README.md](TUI_README.md) for details.

## ⚠️ WARNING

**This script will DESTROY all data on the target drives during new installations!**

**IMPORTANT SAFETY FEATURE:** The script will NEVER destroy data on your running system's device. It includes comprehensive checks to prevent accidental destruction of your source system when using existing system migration mode.

Always backup your data before running this script. Test in a VM first!

## Features

### Core Features
- 🚀 Automated ZFSBootMenu installation and configuration
- 💾 Support for 1 to N drives with configurable RAID levels
- 🔧 EFI partition creation and management
- 🏊 ZFS pool creation with optimized settings
- 📂 Hierarchical ZFS dataset structure
- 🔄 Support for both new installations and existing system migration
- 🐧 Multi-distribution support (Fedora 42/43, Debian 13, MX Linux 25)
- 🛡️ Safety checks, dry-run mode, and comprehensive logging
- 📝 Detailed configuration options

### Advanced Features 🆕
- 🔍 **Device Fitness Checks** - SMART health validation, mounted partition detection, MD RAID checks
- ⚡ **NVMe Optimization** - 4K sector formatting for NVMe drives (optional)
- 🎯 **Optimal ashift Calculation** - Automatic detection of physical block size
- 🚫 **Source System Protection** - Prevents destroying running system device
- 🌐 **Network Identity Zapping** - Complete network config cleanup for migrated systems
- 🥾 **Flexible Bootloader** - Choose ZBM standalone (default), systemd-boot, or rEFInd
- 🎨 **Dual TUI Support** - Both Dialog and Notcurses interfaces

## Supported Distributions

- **Fedora 42/43** - Full support
- **Debian 13** - Full support
- **MX Linux 25 RC1** - Full support
- Other distributions may work but are not officially tested

## Supported RAID Levels

| RAID Level | Min Drives | Description |
|------------|------------|-------------|
| `none` | 1 | Single drive (no redundancy) |
| `mirror` | 2+ | RAID1 mirroring (can lose N-1 drives) |
| `raidz1` | 3+ | RAID5 equivalent (can lose 1 drive) |
| `raidz2` | 4+ | RAID6 equivalent (can lose 2 drives) |
| `raidz3` | 5+ | Can lose 3 drives |

## Prerequisites

- Root/sudo access
- Target drive(s) for installation
- Working network connection (for package installation)
- Basic understanding of ZFS and disk partitioning
- EFI/UEFI system (BIOS not supported)

## Installation

1. Clone this repository:
```bash
git clone https://github.com/Grand-Tetons-Inc/zbm_install_script_prealpha_garbage.git
cd zbm_install_script_prealpha_garbage
```

2. Make the scripts executable:
```bash
chmod +x zbm_install.sh zbm-tui.sh zbm-notcurses.sh
```

3. Review the script and understand what it will do:
```bash
./zbm_install.sh --help
```

## Usage

### Basic Syntax

```bash
sudo ./zbm_install.sh [OPTIONS]
```

### Options

| Option | Description | Required | Default |
|--------|-------------|----------|---------|
| `-m, --mode MODE` | Installation mode: `new` or `existing` | Yes | - |
| `-d, --drives DRIVES` | Comma-separated list of drives (e.g., sda,sdb) | Yes | - |
| `-p, --pool NAME` | ZFS pool name | No | zroot |
| `-r, --raid LEVEL` | RAID level: none, mirror, raidz1, raidz2, raidz3 | No | none |
| `-b, --bootloader TYPE` | Bootloader: zbm, systemd-boot, refind | No | zbm |
| `-e, --efi-size SIZE` | EFI partition size | No | 1G |
| `-s, --swap-size SIZE` | Swap partition size (0 to disable) | No | 8G |
| `-a, --ashift VALUE` | ZFS ashift value (9-16, auto-detect if not specified) | No | auto |
| `-c, --compression TYPE` | ZFS compression: zstd, lz4, lzjb, gzip, off | No | zstd |
| `-H, --hostname NAME` | Set hostname for new installation | No | - |
| `--source-root PATH` | Source root for existing mode | No | / |
| `--exclude PATH` | Paths to exclude (can be used multiple times) | No | - |
| `--no-copy-home` | Don't copy home directories in existing mode | No | false |
| `--nvme-format-4k` | Format NVMe drives to 4K sectors (DESTROYS DATA!) | No | false |
| `-n, --dry-run` | Show what would be done without changes | No | false |
| `-f, --force` | Skip confirmation prompts | No | false |
| `-v, --verbose` | Enable verbose output | No | false |
| `-S, --skip-preflight` | Skip pre-flight system checks | No | false |
| `-B, --no-backup` | Don't backup existing configuration | No | false |
| `-l, --log-file PATH` | Custom log file path | No | /var/log/zbm_install.log |
| `-h, --help` | Display help message | No | - |

## Examples

### Single Drive Installation

Install ZFSBootMenu on a single drive (standalone bootloader):

```bash
sudo ./zbm_install.sh -m new -d sda
```

### Mirrored (RAID1) Installation

Install on two mirrored drives:

```bash
sudo ./zbm_install.sh -m new -d sda,sdb -r mirror
```

### RAIDZ1 Installation

Install on three drives with RAIDZ1:

```bash
sudo ./zbm_install.sh -m new -d sda,sdb,sdc -r raidz1
```

### With systemd-boot

Use systemd-boot as boot manager instead of standalone ZBM:

```bash
sudo ./zbm_install.sh -m new -d sda --bootloader systemd-boot
```

### Custom Configuration

Install with custom pool name and partition sizes:

```bash
sudo ./zbm_install.sh -m new -d sda,sdb -r mirror -p mytank -e 512M -s 16G
```

### NVMe with 4K Sectors

Install on NVMe drive with 4K sector formatting:

```bash
sudo ./zbm_install.sh -m new -d nvme0n1 --nvme-format-4k
```

### Dry Run

Test the configuration without making changes:

```bash
sudo ./zbm_install.sh -m new -d sda,sdb -r mirror --dry-run
```

### No Swap

Install without swap partition:

```bash
sudo ./zbm_install.sh -m new -d sda -s 0
```

### Migrate Existing System 🚀 NEW!

Copy your running system to a new ZFS installation with complete network identity cleanup:

```bash
# Basic migration to mirrored drives
sudo ./zbm_install.sh -m existing -d sda,sdb -r mirror

# Advanced migration with custom exclusions
sudo ./zbm_install.sh -m existing -d nvme0n1 \
  --exclude /home/*/Downloads \
  --exclude /var/cache \
  --no-copy-home \
  -H newhost \
  -v

# Migration with NVMe optimization
sudo ./zbm_install.sh -m existing -d nvme0n1,nvme1n1 \
  -r mirror \
  --nvme-format-4k \
  -H production-server
```

More examples are available in the [examples/](examples/) directory.

## What the Script Does

### New Installation Mode
1. **Validates Configuration** - Checks all parameters and system requirements
2. **Device Fitness Checks** - SMART health, mounted partitions, MD RAID detection
3. **Source System Protection** - Verifies target drives are not source system devices
4. **Device Tuning** - Optimizes parameters, optional NVMe 4K sector formatting
5. **Prepares Disks** - Wipes and cleans target drives
6. **Creates Partitions** - Sets up EFI, swap (optional), and ZFS partitions
7. **Creates ZFS Pool** - Initializes ZFS pool with specified RAID level
8. **Creates Datasets** - Sets up hierarchical ZFS dataset structure
9. **Installs ZFSBootMenu** - Downloads and configures ZFSBootMenu
10. **Configures Bootloader** - Sets up ZBM (standalone), systemd-boot, or rEFInd
11. **Finalizes** - Sets boot properties, creates snapshots, updates configuration

### Existing System Mode (Migration)
1. **Pre-flight Checks** - Validates source system, checks space requirements
2. **Device Fitness Checks** - SMART health, mounted partitions, MD RAID detection
3. **Source System Protection** - Verifies target drives are not source system devices
4. **Device Tuning** - Optimizes parameters, optional NVMe 4K sector formatting
5. **Prepares Disks** - Creates partitions on target drives
6. **Creates ZFS Pool** - Initializes ZFS pool with specified RAID level
7. **Creates Datasets** - Sets up hierarchical ZFS dataset structure
8. **Copies System** - Uses rsync to copy existing system with intelligent exclusions
9. **Post-Copy Config** - Clears machine-id, SSH keys, sets hostname
10. **Network Identity Zapping** - Removes all network configuration for clean identity
11. **Installs ZFSBootMenu** - Downloads and configures ZFSBootMenu
12. **Configures Bootloader** - Sets up boot from new ZFS pool
13. **Finalizes** - Sets boot properties, creates snapshots

## Bootloader Options

### ZBM - Standalone (Default) ⭐ RECOMMENDED

ZFSBootMenu as a direct EFI application (no intermediate boot manager):

```bash
sudo ./zbm_install.sh -m new -d sda --bootloader zbm
# or simply:
sudo ./zbm_install.sh -m new -d sda
```

**Advantages:**
- Simplest configuration
- Fewest moving parts
- Direct EFI boot to ZFSBootMenu
- No intermediate boot manager to maintain

### systemd-boot

Use systemd-boot as boot manager with ZBM as kernel:

```bash
sudo ./zbm_install.sh -m new -d sda --bootloader systemd-boot
```

**When to use:**
- You prefer systemd-boot's interface
- You want to dual-boot with other OSes
- Your distribution defaults to systemd-boot

**Requirements:** `bootctl` command must be available

### rEFInd

Use rEFInd as boot manager with automatic ZBM detection:

```bash
sudo ./zbm_install.sh -m new -d sda --bootloader refind
```

**When to use:**
- You prefer rEFInd's graphical interface
- You want advanced multi-boot capabilities
- You like automatic OS detection

**Requirements:** `refind-install` command must be available

### Existing System Mode
1. **Pre-flight Checks** - Validates source system, checks space requirements
2. **Prepares Disks** - Creates partitions on target drives
3. **Creates ZFS Pool** - Initializes ZFS pool with specified RAID level
4. **Creates Datasets** - Sets up hierarchical ZFS dataset structure
5. **Copies System** - Uses rsync to copy existing system with intelligent exclusions
6. **Post-Copy Config** - Clears machine-id, SSH keys, sets hostname
7. **Installs ZFSBootMenu** - Downloads and configures ZFSBootMenu
8. **Configures Bootloader** - Sets up boot from new ZFS pool
9. **Finalizes** - Sets boot properties, creates snapshots

## ZFS Dataset Structure

The script creates the following dataset hierarchy:

```
zroot/
├── ROOT/
│   └── default          # Root filesystem (/)
├── home/                # User home directories
│   └── root/           # Root user home
├── var/
│   ├── log/            # System logs
│   ├── cache/          # Cache files
│   └── tmp/            # Temporary files
├── opt/                # Optional packages
├── srv/                # Service data
└── usr/
    └── local/          # Locally installed software
```

## File Structure

```
zbm_install_script_prealpha_garbage/
├── zbm_install.sh          # Main installation script
├── zbm-tui.sh              # Dialog-based TUI
├── zbm-notcurses.sh        # Notcurses TUI launcher
├── lib/
│   ├── common.sh           # Common utility functions
│   ├── validation.sh       # Input validation using /proc and /sys
│   ├── disk.sh             # Disk management functions
│   ├── zfs.sh              # ZFS operations
│   ├── bootloader.sh       # Bootloader configuration
│   ├── system.sh           # System migration functions
│   ├── device_tuning.sh    # NVMe tuning, SMART checks, fitness validation
│   └── network_zap.sh      # Network configuration cleanup
├── tui/
│   ├── lib/
│   │   ├── tui_state.sh    # TUI state management
│   │   ├── tui_screens.sh  # Dialog screen definitions
│   │   ├── tui_widgets.sh  # Dialog widgets
│   │   └── notcurses_wrapper.sh  # Notcurses wrapper functions
│   └── python/
│       ├── notcurses_ui.py        # Main notcurses application
│       └── notcurses_components.py # UI components
├── examples/               # Usage examples
│   ├── single_drive.sh
│   ├── mirror.sh
│   ├── raidz1.sh
│   ├── existing_system.sh
│   └── dry_run.sh
├── README.md              # This file
├── TUI_README.md          # TUI documentation
├── LANGUAGE_ANALYSIS.md   # Why we chose Bash
├── QUICK_REFERENCE.md     # Quick command reference
├── CONTRIBUTING.md        # Contribution guidelines
└── LICENSE                # MIT License
```

## Logging

All operations are logged to `/var/log/zbm_install.log` (or custom path with `-l`). Check this file if you encounter issues.

## Safety Features

### Source System Protection

The script includes multiple safety checks to prevent destroying your running system:

- Detects mounted filesystems on target devices
- Identifies source system root device
- Blocks operations on devices containing running system
- Warns if devices are part of MD RAID arrays

**Never** use your system's boot drive as a target unless you understand the consequences!

### Device Fitness Checks

Before any destructive operations, the script validates:

- SMART health status (if smartctl available)
- Mounted partitions
- Active MD RAID membership
- Minimum device size requirements
- Physical vs. logical block sizes

### Network Identity Cleanup

When migrating an existing system, the script completely removes network identity:

- NetworkManager connection profiles
- systemd-networkd configurations
- netplan YAML files
- DHCP leases (all common locations)
- Persistent network interface rules
- cloud-init instance data
- Firewall rules

This ensures the migrated system boots with a clean network identity.

## Troubleshooting

### Script fails with "ZFS not found"

Install ZFS packages manually:
- **Fedora**: `sudo dnf install zfs`
- **Debian/MX**: `sudo apt-get install zfsutils-linux`

### Bootloader validation fails

If you request systemd-boot or rEFInd but the script errors:

```bash
# Install systemd-boot
sudo bootctl install

# Or install rEFInd (Fedora)
sudo dnf install refind

# Or install rEFInd (Debian/MX)
sudo apt-get install refind

# Or just use standalone ZBM (default)
sudo ./zbm_install.sh -m new -d sda
```

### Device fitness check fails

Common causes:
- Device has mounted partitions: Unmount them first
- Device is part of MD RAID: Remove from array first
- SMART health failing: Replace drive before using
- Device too small: Use larger drive or reduce partition sizes

### EFI partition not mounting

Check if the partition was created:
```bash
lsblk
```

Manually format and mount:
```bash
sudo mkfs.vfat -F32 /dev/sda1
sudo mount /dev/sda1 /boot/efi
```

### Pool import fails after reboot

Ensure cachefile is set:
```bash
sudo zpool set cachefile=/etc/zfs/zpool.cache zroot
```

### System doesn't boot to ZFSBootMenu

Verify the bootloader configuration:
```bash
# Check EFI boot entries
efibootmgr -v

# Verify ZBM images exist
ls -lh /boot/efi/EFI/zbm/
```

### NVMe 4K sector format fails

Requirements:
- NVMe drive must support 4K sectors
- Drive must not be in use (unmount all partitions)
- Use `--force` flag to override warnings

Check NVMe capabilities:
```bash
sudo nvme id-ns /dev/nvme0n1 -n 1
```

## References

This script is based on the official ZFSBootMenu documentation:
- [ZFSBootMenu Documentation](https://docs.zfsbootmenu.org/en/latest/)
- [ZFSBootMenu Installation Guides](https://docs.zfsbootmenu.org/en/latest/guides/)
- [ZFS on Linux](https://zfsonlinux.org/)

## Contributing

Contributions are welcome! Please:
1. Test your changes thoroughly
2. Follow the existing code style
3. Run shellcheck on your changes
4. Update documentation as needed

See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

MIT License - See [LICENSE](LICENSE) for details.

## Disclaimer

This software is provided "as is", without warranty of any kind. Always backup your data before using this script. The authors are not responsible for any data loss or system damage.

**IMPORTANT:** This is PRE-ALPHA software. Test thoroughly in VMs before using on production systems!

## Support

For issues and questions:
- Open an issue on GitHub
- Check the [ZFSBootMenu documentation](https://docs.zfsbootmenu.org/)
- Consult the [ZFS documentation](https://openzfs.github.io/openzfs-docs/)
