# Building ZBM Installer

## System Requirements

### Runtime Dependencies
- ZFS utilities (`zfsutils-linux` or `zfs-utils`)
- systemd-boot or another EFI bootloader
- UEFI system with EFI partition
- Linux kernel 4.x or newer

### Build Dependencies

#### Debian/Ubuntu
```bash
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libudev-dev \
    libnotcurses-dev \
    libclang-dev
```

#### Fedora/RHEL
```bash
sudo dnf install -y \
    gcc \
    pkg-config \
    systemd-devel \
    notcurses-devel \
    clang-devel
```

#### Arch Linux
```bash
sudo pacman -S \
    base-devel \
    systemd \
    notcurses \
    clang
```

## Building

### Standard Build
```bash
cargo build --release
```

### Build without TUI (CLI only)
```bash
cargo build --release --no-default-features
```

### Development Build
```bash
cargo build
```

## Features

- `tui` (enabled by default): Includes the notcurses-based TUI interface

## Running

The installer must be run as root:

```bash
# TUI mode (interactive)
sudo ./target/release/zbm-installer --tui

# CLI mode
sudo ./target/release/zbm-installer --mode new --drives /dev/sda --raid mirror

# Dry run (recommended for testing)
sudo ./target/release/zbm-installer --mode new --drives /dev/sda --dry-run
```

## Troubleshooting

### libnotcurses not found
Install the notcurses development package for your distribution, or build without TUI:
```bash
cargo build --release --no-default-features
```

### libudev not found
Install the libudev/systemd development package for your distribution.

### Permission denied
The installer must be run as root to access block devices and create file systems.
