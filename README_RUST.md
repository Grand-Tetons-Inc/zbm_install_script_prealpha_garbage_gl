# ZFSBootMenu Installer - Rust Edition

A modern, safe, and feature-rich installer for ZFSBootMenu written in **Rust**, inspired by Growlight's architecture and designed for use with Notcurses TUI.

## 🚀 Features

- ✅ **Memory-safe Rust implementation** - No buffer overflows, no undefined behavior
- ✅ **Comprehensive CLI** - Full-featured command-line interface with all options
- ✅ **Device discovery** - Growlight-inspired device discovery via `/sys/class/block`
- ✅ **Multiple RAID levels** - Support for none, mirror, raidz1, raidz2, raidz3
- ✅ **Dry-run mode** - Test configurations without making changes
- ✅ **Pre-flight validation** - Comprehensive system checks before installation
- ✅ **Existing system migration** - Migrate running systems to ZFS (planned)
- 🚧 **Notcurses TUI** - Framework in place, full implementation pending
- ✅ **Extensive testing** - Unit and integration test infrastructure

## 📋 Requirements

### System Requirements

- Linux kernel with ZFS support
- UEFI boot mode
- Root privileges
- Minimum 2GB RAM (recommended)
- At least 8GB of disk space

### Software Dependencies

**Build Dependencies:**
```bash
# Ubuntu/Debian
sudo apt-get install build-essential pkg-config libudev-dev libnotcurses-dev

# Fedora
sudo dnf install gcc pkg-config systemd-devel libnotcurses-devel

# Arch
sudo pacman -S base-devel pkg-config systemd libnotcurses
```

**Runtime Dependencies:**
- ZFS utilities (`zfsutils-linux` or `zfs`)
- `sgdisk` (gdisk package)
- `mkfs.vfat` (dosfstools)
- `efibootmgr`
- `dracut` or `mkinitcpio`

## 🔨 Building

```bash
# Clone the repository
git clone https://github.com/Grand-Tetons-Inc/zbm_install_script_prealpha_garbage.git
cd zbm_install_script_prealpha_garbage

# Build in release mode
cargo build --release

# The binary will be at:
# target/release/zbm-installer
```

### Optional: Build with TUI support

*Note: Requires Notcurses >= 3.0.11*

```bash
cargo build --release --features tui
```

## 📖 Usage

### Basic Examples

#### Single Drive Installation
```bash
sudo ./target/release/zbm-installer --mode new --drives /dev/sda
```

#### Mirrored Drives (RAID1)
```bash
sudo ./target/release/zbm-installer --mode new \
  --drives /dev/sda,/dev/sdb \
  --raid mirror
```

#### RAIDZ1 with Custom Settings
```bash
sudo ./target/release/zbm-installer --mode new \
  --drives /dev/sda,/dev/sdb,/dev/sdc \
  --raid raidz1 \
  --pool-name mytank \
  --compression lz4 \
  --efi-size 512M \
  --swap-size 16G
```

#### Dry Run (Recommended for Testing)
```bash
sudo ./target/release/zbm-installer --mode new \
  --drives /dev/sda,/dev/sdb \
  --raid mirror \
  --dry-run
```

### CLI Options

```
OPTIONS:
  -m, --mode <MODE>              Installation mode: new or existing
  -d, --drives <DRIVES>          Comma-separated list of drives (e.g., /dev/sda,/dev/sdb)
  -p, --pool-name <NAME>         ZFS pool name [default: zroot]
  -r, --raid <LEVEL>             RAID level: none, mirror, raidz1, raidz2, raidz3 [default: none]
  -e, --efi-size <SIZE>          EFI partition size [default: 1G]
  -s, --swap-size <SIZE>         Swap partition size (0 to disable) [default: 8G]
  -a, --ashift <VALUE>           ZFS ashift value (9-16, auto-detect if not specified)
  -c, --compression <TYPE>       ZFS compression: zstd, lz4, lzjb, gzip, off [default: zstd]
  -H, --hostname <NAME>          Hostname for new installation
      --source-root <PATH>       Source root for existing mode [default: /]
      --exclude <PATH>           Paths to exclude (can be used multiple times)
      --no-copy-home             Don't copy home directories in existing mode
  -n, --dry-run                  Show what would be done without making changes
  -f, --force                    Skip confirmation prompts
  -v, --verbose                  Enable verbose output
  -S, --skip-preflight           Skip pre-flight system checks (not recommended)
  -t, --tui                      Launch interactive TUI
  -h, --help                     Display help message
```

## 🏗️ Architecture

The installer is built with a modular architecture inspired by Growlight:

```
zbm-installer/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Main installer orchestration
│   ├── error.rs             # Error types
│   ├── config.rs            # Configuration management
│   ├── validation.rs        # Pre-flight checks
│   │
│   ├── disk/                # Growlight-inspired disk management
│   │   ├── mod.rs           # Disk manager
│   │   ├── block_device.rs  # Block device representation
│   │   ├── discovery.rs     # Device discovery via /sys
│   │   └── operations.rs    # Disk operations
│   │
│   ├── zfs/                 # ZFS operations
│   │   ├── mod.rs
│   │   ├── pool.rs          # Pool creation
│   │   └── dataset.rs       # Dataset management
│   │
│   ├── bootloader/          # Bootloader installation
│   │   ├── mod.rs
│   │   ├── zbm.rs           # ZFSBootMenu
│   │   └── systemd_boot.rs  # systemd-boot
│   │
│   ├── system/              # System utilities
│   │   ├── mod.rs
│   │   ├── distro.rs        # Distribution detection
│   │   └── packages.rs      # Package management
│   │
│   └── ui/                  # TUI framework (Notcurses)
│       ├── mod.rs
│       ├── screens.rs
│       └── runner.rs
```

### Key Design Decisions

1. **Rust over C/Bash**:
   - Memory safety without garbage collection
   - Superior error handling with `Result<T, E>`
   - Excellent testing infrastructure
   - Modern tooling (cargo, clippy, rustfmt)

2. **Growlight-Inspired Disk Management**:
   - Device discovery via `/sys/class/block`
   - Controller-based organization
   - Inotify support for hotplug detection

3. **Modular Architecture**:
   - Each subsystem is independent
   - Easy to test in isolation
   - Clear separation of concerns

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_config_validation

# Run with verbose output
cargo test --verbose
```

### Test Coverage

- Unit tests for all core modules
- Integration tests for full workflows
- Validation tests
- Error handling tests

## 🔍 Development

### Code Style

```bash
# Format code
cargo fmt

# Check code style
cargo fmt -- --check

# Run linter
cargo clippy

# Run linter with all warnings as errors
cargo clippy -- -D warnings
```

### Documentation

```bash
# Generate and open documentation
cargo doc --open

# Generate documentation for private items
cargo doc --document-private-items --open
```

## 📚 Documentation

- **[ARCHITECTURE.md](ARCHITECTURE.md)** - Detailed architecture design
- **[API Documentation](https://docs.rs/zbm-installer)** - Generated from code
- Inline documentation for all public APIs

## 🛠️ Troubleshooting

### Build Issues

**Error: `notcurses.pc` not found**
```bash
# Install libnotcurses-dev
sudo apt-get install libnotcurses-dev  # Ubuntu/Debian
sudo dnf install libnotcurses-devel    # Fedora
```

**Error: `libudev.pc` not found**
```bash
# Install libudev-dev
sudo apt-get install libudev-dev  # Ubuntu/Debian
sudo dnf install systemd-devel    # Fedora
```

### Runtime Issues

**Error: "This program must be run as root"**
```bash
# Run with sudo
sudo ./target/release/zbm-installer [OPTIONS]
```

**Error: "System must be booted in UEFI mode"**
- Check that `/sys/firmware/efi` exists
- Reboot in UEFI mode if using legacy BIOS

**Error: "ZFS is not available"**
```bash
# Install ZFS
sudo apt-get install zfsutils-linux  # Ubuntu/Debian
sudo dnf install zfs                  # Fedora
```

## 🚀 Roadmap

### Completed ✅
- [x] Core Rust implementation
- [x] CLI interface
- [x] Device discovery
- [x] Disk operations (partition, format)
- [x] ZFS pool creation
- [x] ZFS dataset management
- [x] Bootloader installation
- [x] Pre-flight validation
- [x] Dry-run mode
- [x] Error handling
- [x] Comprehensive testing framework

### In Progress 🚧
- [ ] Full Notcurses TUI implementation
- [ ] System migration (rsync-based)
- [ ] Additional filesystem support

### Planned 📝
- [ ] GUI (GTK or egui)
- [ ] Snapshot management
- [ ] Boot environment management
- [ ] Recovery tools
- [ ] Remote installation support

## 🤝 Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Run linter: `cargo clippy`
6. Format code: `cargo fmt`
7. Submit a pull request

### Coding Standards

- Follow Rust conventions
- Document all public APIs
- Write tests for new features
- Keep functions focused and small
- Use descriptive variable names

## 📄 License

MIT License - See [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- **[ZFSBootMenu](https://zfsbootmenu.org/)** - The bootloader this installer targets
- **[Growlight](https://github.com/dankamongmen/growlight)** - Inspiration for disk management architecture
- **[Notcurses](https://github.com/dankamongmen/notcurses)** - TUI library
- **Rust Community** - For excellent tools and ecosystem

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/Grand-Tetons-Inc/zbm_install_script_prealpha_garbage/issues)
- **Documentation**: [Architecture Guide](ARCHITECTURE.md)
- **ZFSBootMenu**: [Official Docs](https://docs.zfsbootmenu.org/)

## ⚠️ Disclaimer

This software is provided "as is", without warranty of any kind. **Always backup your data** before using this installer. Test in a VM first!

The authors are not responsible for any data loss or system damage.

---

Built with ❤️ and 🦀 (Rust)
