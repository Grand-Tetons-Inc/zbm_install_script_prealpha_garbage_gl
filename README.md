# ZFSBootMenu Installation Script

A comprehensive BASH script solution for installing [ZFSBootMenu](https://zfsbootmenu.org/) on Linux systems with support for single or multiple drives in various RAID configurations.

## ⚠️ WARNING

**This script will DESTROY all data on the target drives during new installations!**

Always backup your data before running this script. Test in a VM first!

## Features

- 🚀 Automated ZFSBootMenu installation and configuration
- 💾 Support for 1 to N drives with configurable RAID levels
- 🔧 EFI partition creation and management
- 🏊 ZFS pool creation with optimized settings
- 📂 Hierarchical ZFS dataset structure
- 🔄 Support for both new installations and existing systems
- 🐧 Multi-distribution support (Fedora 42/43, Debian 13, MX Linux 25)
- 🛡️ Safety checks, dry-run mode, and comprehensive logging
- 📝 Detailed configuration options

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

## Installation

1. Clone this repository:
```bash
git clone https://github.com/Grand-Tetons-Inc/zbm_install_script_prealpha_garbage.git
cd zbm_install_script_prealpha_garbage
```

2. Make the script executable:
```bash
chmod +x zbm_install.sh
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
| `-e, --efi-size SIZE` | EFI partition size | No | 1G |
| `-s, --swap-size SIZE` | Swap partition size (0 to disable) | No | 8G |
| `-n, --dry-run` | Show what would be done without changes | No | false |
| `-f, --force` | Skip confirmation prompts | No | false |
| `-h, --help` | Display help message | No | - |

## Examples

### Single Drive Installation

Install ZFSBootMenu on a single drive:

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

### Custom Configuration

Install with custom pool name and partition sizes:

```bash
sudo ./zbm_install.sh -m new -d sda,sdb -r mirror -p mytank -e 512M -s 16G
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

More examples are available in the [examples/](examples/) directory.

## What the Script Does

1. **Validates Configuration** - Checks all parameters and system requirements
2. **Prepares Disks** - Wipes and cleans target drives (in new mode)
3. **Creates Partitions** - Sets up EFI, swap (optional), and ZFS partitions
4. **Creates ZFS Pool** - Initializes ZFS pool with specified RAID level
5. **Creates Datasets** - Sets up hierarchical ZFS dataset structure
6. **Installs ZFSBootMenu** - Downloads and configures ZFSBootMenu
7. **Configures Bootloader** - Sets up systemd-boot or rEFInd
8. **Finalizes** - Sets boot properties, creates snapshots, updates configuration

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
├── zbm_install.sh       # Main installation script
├── lib/
│   ├── common.sh       # Common utility functions
│   ├── disk.sh         # Disk management functions
│   ├── zfs.sh          # ZFS operations
│   └── bootloader.sh   # Bootloader configuration
├── examples/           # Usage examples
│   ├── single_drive.sh
│   ├── mirror.sh
│   ├── raidz1.sh
│   ├── existing_system.sh
│   └── dry_run.sh
├── README.md          # This file
└── LICENSE            # MIT License

```

## Logging

All operations are logged to `/var/log/zbm_install.log`. Check this file if you encounter issues.

## Troubleshooting

### Script fails with "ZFS not found"

Install ZFS packages manually:
- **Fedora**: `sudo dnf install zfs`
- **Debian/MX**: `sudo apt-get install zfsutils-linux`

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

Verify the bootloader configuration and ensure ZFSBootMenu images are in `/boot/efi/EFI/zbm/`.

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

## License

MIT License - See [LICENSE](LICENSE) for details.

## Disclaimer

This software is provided "as is", without warranty of any kind. Always backup your data before using this script. The authors are not responsible for any data loss or system damage.

## Support

For issues and questions:
- Open an issue on GitHub
- Check the [ZFSBootMenu documentation](https://docs.zfsbootmenu.org/)
- Consult the [ZFS documentation](https://openzfs.github.io/openzfs-docs/)
