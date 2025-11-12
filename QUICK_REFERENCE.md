# ZFSBootMenu Installation Quick Reference

## Quick Start

```bash
# 1. Clone the repository
git clone https://github.com/Grand-Tetons-Inc/zbm_install_script_prealpha_garbage.git
cd zbm_install_script_prealpha_garbage

# 2. Make executable
chmod +x zbm_install.sh zbm-tui.sh zbm-notcurses.sh

# 3. Run with desired options (as root)
sudo ./zbm_install.sh -m new -d sda
```

## Interactive TUI

```bash
# Dialog-based TUI (works everywhere)
sudo ./zbm-tui.sh

# Notcurses TUI (advanced graphics)
sudo ./zbm-notcurses.sh
```

## Common Commands

### Single Drive (Standalone ZBM)
```bash
sudo ./zbm_install.sh -m new -d sda
```

### Mirrored Drives (RAID1)
```bash
sudo ./zbm_install.sh -m new -d sda,sdb -r mirror
```

### RAIDZ1 (3+ drives)
```bash
sudo ./zbm_install.sh -m new -d sda,sdb,sdc -r raidz1
```

### With systemd-boot
```bash
sudo ./zbm_install.sh -m new -d sda --bootloader systemd-boot
```

### With rEFInd
```bash
sudo ./zbm_install.sh -m new -d sda --bootloader refind
```

### NVMe with 4K Sector Formatting
```bash
sudo ./zbm_install.sh -m new -d nvme0n1 --nvme-format-4k
```

### Custom Configuration
```bash
sudo ./zbm_install.sh -m new -d sda,sdb -r mirror -p mytank -e 512M -s 16G
```

### Advanced Configuration (SSD with zstd compression)
```bash
sudo ./zbm_install.sh -m new -d nvme0n1 -a 12 -c zstd -s 16G -v
```

### No Swap Installation
```bash
sudo ./zbm_install.sh -m new -d sda -s 0
```

### Test Without Changes (Dry Run)
```bash
sudo ./zbm_install.sh -m new -d sda,sdb -r mirror --dry-run
```

### Verbose Mode (for debugging)
```bash
sudo ./zbm_install.sh -m new -d sda -v
```

### Migrate Existing System (EXCITING NEW FEATURE!)
```bash
# Copy running system to new ZFS mirrored setup
# Includes automatic network identity cleanup!
sudo ./zbm_install.sh -m existing -d sda,sdb -r mirror

# Exclude specific paths during migration
sudo ./zbm_install.sh -m existing -d nvme0n1 \
  --exclude /home/user/Downloads \
  --exclude /var/cache

# Don't copy home directories
sudo ./zbm_install.sh -m existing -d sda --no-copy-home

# With NVMe optimization
sudo ./zbm_install.sh -m existing -d nvme0n1,nvme1n1 \
  -r mirror \
  --nvme-format-4k \
  -H newhost
```

## Option Reference

### Required Options
| Option | Values | Description |
|--------|--------|-------------|
| `-m, --mode` | `new`, `existing` | Installation mode |
| `-d, --drives` | `sda`, `sda,sdb`, etc. | Comma-separated drive list |

### Bootloader Configuration 🆕
| Option | Values | Default | Description |
|--------|--------|---------|-------------|
| `-b, --bootloader` | `zbm`, `systemd-boot`, `refind` | `zbm` | Bootloader type (zbm is standalone) |

### Storage Configuration
| Option | Values | Default | Description |
|--------|--------|---------|-------------|
| `-p, --pool` | Any string | `zroot` | ZFS pool name |
| `-r, --raid` | `none`, `mirror`, `raidz1`, `raidz2`, `raidz3` | `none` | RAID level |
| `-e, --efi-size` | `512M`, `1G`, etc. | `1G` | EFI partition size |
| `-s, --swap-size` | `8G`, `16G`, `0` | `8G` | Swap size (0=disable) |

### Device Tuning 🆕
| Option | Values | Default | Description |
|--------|--------|---------|-------------|
| `--nvme-format-4k` | flag | `false` | Format NVMe to 4K sectors (DESTROYS DATA!) |

### ZFS Tuning
| Option | Values | Default | Description |
|--------|--------|---------|-------------|
| `-a, --ashift` | `9-16` or `auto` | `auto` | ZFS block alignment (9=512B, 12=4K, 13=8K) |
### ZFS Tuning
| Option | Values | Default | Description |
|--------|--------|---------|-------------|
| `-a, --ashift` | `9-16` | auto-detect | ZFS block alignment (9=512B, 12=4K, 13=8K) |
| `-c, --compression` | `zstd`, `lz4`, `lzjb`, `gzip`, `off` | `zstd` | Compression algorithm |

### System Configuration
| Option | Values | Default | Description |
|--------|--------|---------|-------------|
| `-H, --hostname` | Any string | - | Set hostname for new installation |
| `-l, --log-file` | File path | `/var/log/zbm_install.log` | Custom log file location |

### Existing System Migration
| Option | Values | Default | Description |
|--------|--------|---------|-------------|
| `--source-root` | Directory path | `/` | Source root for existing mode |
| `--exclude` | Path pattern | - | Exclude path from copy (repeatable) |
| `--no-copy-home` | flag | `false` | Skip copying home directories |

### Execution Control
| Option | Values | Default | Description |
|--------|--------|---------|-------------|
| `-n, --dry-run` | flag | `false` | Test mode (no changes) |
| `-f, --force` | flag | `false` | Skip confirmations |
| `-v, --verbose` | flag | `false` | Enable verbose output |
| `-S, --skip-preflight` | flag | `false` | Skip pre-flight checks (not recommended) |
| `-B, --no-backup` | flag | `false` | Don't backup existing configuration |
| `-h, --help` | flag | - | Show help message |

## Bootloader Options Explained

### zbm (Default) ⭐ RECOMMENDED
- ZFSBootMenu as standalone EFI bootloader
- No intermediate boot manager needed
- Simplest configuration, fewest moving parts
- Direct EFI boot to ZFSBootMenu

```bash
sudo ./zbm_install.sh -m new -d sda
# or explicitly:
sudo ./zbm_install.sh -m new -d sda --bootloader zbm
```

### systemd-boot
- Uses systemd-boot as boot manager
- ZBM loaded as kernel by systemd-boot
- Good for dual-boot setups
- Requires `bootctl` command

```bash
sudo ./zbm_install.sh -m new -d sda --bootloader systemd-boot
```

### refind
- Uses rEFInd as graphical boot manager
- Automatic OS detection
- Advanced multi-boot support
- Requires `refind-install` command

```bash
sudo ./zbm_install.sh -m new -d sda --bootloader refind
```

## RAID Requirements

| RAID Level | Min Drives | Redundancy | Description |
|------------|------------|------------|-------------|
| `none` | 1 | None | Single drive |
| `mirror` | 2+ | N-1 drives | Full mirroring |
| `raidz1` | 3+ | 1 drive | RAID5-like |
| `raidz2` | 4+ | 2 drives | RAID6-like |
| `raidz3` | 5+ | 3 drives | Triple parity |

## Pre-Installation Checklist

- [ ] Backup all important data
- [ ] Identify target drive(s) with `lsblk`
- [ ] Verify system is EFI (check `/sys/firmware/efi` exists)
- [ ] Ensure sufficient RAM (minimum 2GB)
- [ ] Verify sufficient disk space (minimum 8GB per drive)
- [ ] Ensure system has network access
- [ ] Test with `--dry-run` first
- [ ] Understand that target drives will be COMPLETELY WIPED
- [ ] For migration: ensure target drives are NOT your system drive

## Safety Features 🆕

### Automatic Device Fitness Checks
- SMART health validation
- Mounted partition detection
- MD RAID membership check
- Source system protection (never destroys running system)
- Minimum size requirements

### Network Identity Cleanup (Migration Mode)
- Removes NetworkManager connections
- Clears systemd-networkd configs
- Removes netplan configurations
- Clears DHCP leases
- Removes persistent network rules
- Clears cloud-init data
- Removes firewall rules

## Post-Installation

1. Verify installation:
   ```bash
   zpool status
   zfs list
   ```

2. Check boot configuration:
   ```bash
   # Check EFI entries
   efibootmgr -v

   # Verify ZBM images
   ls -l /boot/efi/EFI/zbm/

   # If using systemd-boot
   bootctl status
   ```

3. Reboot system:
   ```bash
   reboot
   ```

4. At boot, select ZFSBootMenu (or it should auto-boot)

## Troubleshooting

### Check logs
```bash
cat /var/log/zbm_install.log
# or
cat /tmp/zbm_install.log
```

### View disk layout
```bash
lsblk
fdisk -l
```

### Check ZFS status
```bash
zpool status
zfs list
```

### Verify EFI boot
```bash
efibootmgr -v
ls -l /boot/efi/EFI/
```

### Check device fitness
```bash
# Check SMART health
sudo smartctl -H /dev/sda

# Check NVMe sector size
sudo nvme id-ns /dev/nvme0n1 -n 1

# View mounted partitions
mount | grep sda
```

## Example Workflow

```bash
# 1. Check system requirements
cat /proc/meminfo | grep MemTotal
ls /sys/firmware/efi  # Verify EFI system

# 2. Check available disks
lsblk
ls -l /sys/block/  # See all block devices

# 3. Test configuration (dry run)
sudo ./zbm_install.sh -m new -d sda,sdb -r mirror --dry-run -v

# 4. Run actual installation
sudo ./zbm_install.sh -m new -d sda,sdb -r mirror -v

# 5. Verify after installation
zpool status zroot
zfs list -r zroot
efibootmgr -v  # Check boot entry
cat /var/log/zbm_install.log  # Check logs

# 6. Reboot
sudo reboot
```

## Advanced Examples

### High-Performance NVMe Setup with 4K Sectors
```bash
# NVMe with 4K sector formatting
### High-Performance SSD Setup
```bash
# NVMe SSD with optimal settings
sudo ./zbm_install.sh \
  -m new \
  -d nvme0n1,nvme1n1 \
  -r mirror \
  --nvme-format-4k \
  -a 12 \
  -c zstd \
  -s 32G \
  -H myserver \
  -v
```

### Minimal Installation (No Swap, Standalone ZBM)
```bash
# Single drive, no swap, smaller EFI
sudo ./zbm_install.sh \
  -m new \
  -d sda \
  -e 512M \
  -s 0 \
  -c lz4 \
  --bootloader zbm
```

### RAIDZ2 for Data Integrity
```bash
# 6-drive RAIDZ2 (can lose 2 drives)
sudo ./zbm_install.sh \
  -m new \
  -d sda,sdb,sdc,sdd,sde,sdf \
  -r raidz2 \
  -c zstd \
  -v
```

### Migrate Running System to ZFS with Network Cleanup
```bash
# Copy current system to new ZFS mirror with custom exclusions
# Network identity is automatically cleaned!
sudo ./zbm_install.sh \
  -m existing \
  -d sda,sdb \
  -r mirror \
  -c zstd \
  --exclude /home/*/Downloads \
  --exclude /var/tmp \
  --exclude /var/cache \
  -H mynewhost \
  -v

# Quick migration without home directories
sudo ./zbm_install.sh \
  -m existing \
  -d nvme0n1 \
  --no-copy-home \
  -f

# Migration with NVMe optimization
sudo ./zbm_install.sh \
  -m existing \
  -d nvme0n1,nvme1n1 \
  -r mirror \
  --nvme-format-4k \
  -H production \
  -v
```

### Dual Boot Setup with rEFInd
```bash
# Install with rEFInd for multi-OS boot management
sudo ./zbm_install.sh \
  -m new \
  -d sda \
  --bootloader refind \
  -v
```

## Need Help?

- Read full documentation: `README.md`
- Check TUI documentation: `TUI_README.md`
- Check examples: `examples/` directory
- Review logs: `/var/log/zbm_install.log`
- Visit: https://docs.zfsbootmenu.org/

## Safety Tips

⚠️ **ALWAYS:**
- Backup data before installation
- Test in VM first
- Use `--dry-run` to preview
- Verify drive names with `lsblk`
- Double-check RAID configuration
- Verify target drives are NOT your system drive
- Read and understand all warnings

❌ **NEVER:**
- Run on production without backup
- Use wrong drive identifiers
- Interrupt installation process
- Ignore error messages
- Use your system's boot drive as target (unless you know what you're doing)
- Format NVMe to 4K without understanding implications

## Quick Troubleshooting

| Issue | Solution |
|-------|----------|
| "Bootloader validation fails" | Install requested bootloader or use `--bootloader zbm` |
| "Device fitness check fails" | Unmount partitions, remove from MD RAID, or check SMART health |
| "Source system protection" | Don't use your running system's drive as target |
| "Network not working after migration" | Normal! Network config was cleaned for fresh identity |
| "System doesn't boot" | Check `efibootmgr -v` and verify `/boot/efi/EFI/zbm/` exists |
