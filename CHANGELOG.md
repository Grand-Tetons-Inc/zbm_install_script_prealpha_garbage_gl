# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial implementation of ZFSBootMenu installation script
- Main orchestrator script (`zbm_install.sh`) with comprehensive CLI interface
- Library modules for modular functionality:
  - `lib/common.sh`: Logging, distribution detection, package management
  - `lib/disk.sh`: Disk preparation, partitioning, RAID validation
  - `lib/zfs.sh`: ZFS pool and dataset creation and management
  - `lib/bootloader.sh`: ZFSBootMenu installation and bootloader configuration
- Support for multiple RAID levels:
  - Single drive (none)
  - Mirror (RAID1)
  - RAIDZ1, RAIDZ2, RAIDZ3
- Distribution support:
  - Fedora 42/43
  - Debian 13
  - MX Linux 25 RC1
- Installation modes:
  - New installation (fresh setup)
  - Existing system (framework for migration)
- EFI partition creation and management
- Hierarchical ZFS dataset structure
- Bootloader support:
  - systemd-boot (preferred)
  - rEFInd (fallback)
- Configuration features:
  - Configurable EFI partition size
  - Configurable swap partition size (or disabled)
  - Custom pool naming
  - Dry-run mode for testing
  - Force mode to skip confirmations
- Error handling and validation:
  - Input validation
  - RAID level validation
  - Drive existence checks
  - Root permission checks
  - Cleanup on error
- Logging:
  - Color-coded console output
  - File logging to /var/log or /tmp
  - Command execution tracking
- Documentation:
  - Comprehensive README.md
  - Usage examples in `examples/` directory
  - CONTRIBUTING.md guidelines
  - Inline code comments
- Example scripts:
  - Single drive installation
  - Mirrored drives (RAID1)
  - RAIDZ1 installation
  - Existing system migration
  - Dry-run testing
- `.gitignore` for log and temporary files

### Changed
- Updated README from minimal description to comprehensive documentation

### Security
- All scripts pass shellcheck validation
- Proper error handling with `set -e -u -o pipefail`
- Input validation to prevent common mistakes
- Confirmation prompts before destructive operations

## [0.1.0] - 2025-11-10

### Added
- Initial project structure
- MIT License
- Basic README

[Unreleased]: https://github.com/Grand-Tetons-Inc/zbm_install_script_prealpha_garbage/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Grand-Tetons-Inc/zbm_install_script_prealpha_garbage/releases/tag/v0.1.0
