# ZFSBootMenu Installation Quick Reference

## Quick Start

```bash
# 1. Clone the repository
git clone https://github.com/Grand-Tetons-Inc/zbm_install_script_prealpha_garbage.git
cd zbm_install_script_prealpha_garbage

# 2. Make executable
chmod +x zbm_install.sh

# 3. Run with desired options (as root)
sudo ./zbm_install.sh -m new -d sda
```

## Common Commands

### Single Drive
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

### Custom Configuration
```bash
sudo ./zbm_install.sh -m new -d sda,sdb -r mirror -p mytank -e 512M -s 16G
```

### Test Without Changes (Dry Run)
```bash
sudo ./zbm_install.sh -m new -d sda,sdb -r mirror --dry-run
```

## Option Reference

| Option | Values | Default | Description |
|--------|--------|---------|-------------|
| `-m, --mode` | `new`, `existing` | *required* | Installation mode |
| `-d, --drives` | `sda`, `sda,sdb`, etc. | *required* | Comma-separated drive list |
| `-p, --pool` | Any string | `zroot` | ZFS pool name |
| `-r, --raid` | `none`, `mirror`, `raidz1`, `raidz2`, `raidz3` | `none` | RAID level |
| `-e, --efi-size` | `512M`, `1G`, etc. | `1G` | EFI partition size |
| `-s, --swap-size` | `8G`, `16G`, `0` | `8G` | Swap size (0=disable) |
| `-n, --dry-run` | flag | `false` | Test mode (no changes) |
| `-f, --force` | flag | `false` | Skip confirmations |
| `-h, --help` | flag | - | Show help |

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
- [ ] Verify sufficient disk space
- [ ] Ensure system has network access
- [ ] Test with `--dry-run` first
- [ ] Understand that target drives will be wiped

## Post-Installation

1. Verify installation:
   ```bash
   zpool status
   zfs list
   ```

2. Check boot configuration:
   ```bash
   ls -l /boot/efi/EFI/zbm/
   bootctl status  # or: efibootmgr
   ```

3. Reboot system:
   ```bash
   reboot
   ```

4. At boot, select ZFSBootMenu

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

## Example Workflow

```bash
# 1. Check available disks
lsblk

# 2. Test configuration
sudo ./zbm_install.sh -m new -d sda,sdb -r mirror --dry-run

# 3. Run actual installation
sudo ./zbm_install.sh -m new -d sda,sdb -r mirror

# 4. Verify after installation
zpool status zroot
zfs list -r zroot

# 5. Reboot
sudo reboot
```

## Need Help?

- Read full documentation: `README.md`
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

❌ **NEVER:**
- Run on production without backup
- Use wrong drive identifiers
- Interrupt installation process
- Ignore error messages
