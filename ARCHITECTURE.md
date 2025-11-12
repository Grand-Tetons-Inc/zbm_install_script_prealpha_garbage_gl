# ZFSBootMenu Installer - Rust Architecture Design

## Overview

Complete rewrite of the ZFSBootMenu installer in Rust using Notcurses for TUI, inspired by Growlight's architecture.

## Language Choice: Rust

**Rationale:**
- **Memory Safety**: Critical for system installer that could brick machines
- **Testing**: Superior unit/integration testing framework
- **Tooling**: Cargo (build/test/doc), Clippy (linting), Rustfmt (formatting)
- **Notcurses Support**: libnotcurses-sys is well-maintained
- **Error Handling**: Result<T, E> provides robust error propagation
- **Maintainability**: Modern, safe, with growing systems programming ecosystem

## System Architecture

### High-Level Flow (from existing bash implementation)
```
1. Welcome Screen
2. Mode Selection (New/Existing)
3. Device Discovery & Selection
4. RAID Configuration
5. Settings Configuration (pool name, compression, etc.)
6. Pre-flight Validation
7. Confirmation
8. Execution (with progress)
9. Completion
```

### Module Structure

```
zbm-installer/
├── Cargo.toml                 # Project manifest
├── src/
│   ├── main.rs               # Entry point, CLI args, TUI init
│   ├── lib.rs                # Public library interface
│   ├── error.rs              # Error types and Result<T, E>
│   ├── config.rs             # Configuration state
│   ├── validation.rs         # Input validation
│   │
│   ├── ui/                   # Notcurses TUI (inspired by Growlight)
│   │   ├── mod.rs           # UI manager, event loop
│   │   ├── screens.rs       # Screen implementations
│   │   ├── widgets.rs       # Reusable UI components (panels, lists, dialogs)
│   │   ├── navigation.rs    # Keyboard/mouse navigation
│   │   └── renderer.rs      # Drawing utilities
│   │
│   ├── disk/                 # Disk management (Growlight-inspired)
│   │   ├── mod.rs           # Disk manager, controller abstraction
│   │   ├── discovery.rs     # Device discovery via /sys/class/block + inotify
│   │   ├── operations.rs    # Partitioning, formatting, wiping
│   │   ├── block_device.rs  # Block device representation
│   │   └── controller.rs    # Storage controller/adapter abstraction
│   │
│   ├── zfs/                  # ZFS operations
│   │   ├── mod.rs           # ZFS manager
│   │   ├── pool.rs          # Pool creation, RAID configuration
│   │   ├── dataset.rs       # Dataset hierarchy creation
│   │   └── commands.rs      # ZFS command execution wrappers
│   │
│   ├── bootloader/           # Bootloader management
│   │   ├── mod.rs           # Bootloader manager
│   │   ├── zbm.rs           # ZFSBootMenu installation
│   │   ├── systemd_boot.rs  # systemd-boot configuration
│   │   └── refind.rs        # rEFInd configuration
│   │
│   └── system/               # System operations
│       ├── mod.rs           # System utilities
│       ├── distro.rs        # Distribution detection
│       ├── packages.rs      # Package installation
│       └── migration.rs     # Existing system migration (rsync)
│
├── tests/                    # Integration tests
│   ├── disk_tests.rs
│   ├── zfs_tests.rs
│   └── ui_tests.rs
│
└── docs/                     # Documentation
    ├── API.md
    ├── TESTING.md
    └── DEVELOPMENT.md
```

## Core Components

### 1. UI System (Notcurses)

**Inspired by Growlight's approach:**
- Fullscreen TUI with panels for controllers/devices
- Real-time device discovery updates
- Vi-style keybindings
- Context-sensitive help

**Screens:**
```rust
enum Screen {
    Welcome,              // Splash, warnings
    ModeSelect,           // New vs Existing installation
    DeviceDiscovery,      // Show detected devices (Growlight style)
    DeviceSelect,         // Multi-select devices for pool
    RaidConfig,           // Choose RAID level
    Settings,             // Pool name, compression, swap, etc.
    PreflightCheck,       // Validation results
    Confirmation,         // Final review
    Execution,            // Progress bars, live logs
    Completion,           // Success/failure, next steps
}
```

**Widgets:**
- `DeviceList`: Hierarchical controller -> device -> partition view
- `ProgressBar`: Multi-step installation progress
- `ConfirmDialog`: Yes/no confirmations
- `SettingsForm`: Key-value configuration
- `LogViewer`: Real-time log display

### 2. Disk Management (Growlight Pattern)

**Discovery:**
```rust
struct DiskDiscovery {
    inotify: Inotify,  // Watch /sys/class/block for hotplug
}

impl DiskDiscovery {
    fn scan_devices() -> Vec<BlockDevice>;
    fn watch_for_changes() -> Result<()>;
}
```

**Block Device:**
```rust
struct BlockDevice {
    path: PathBuf,           // /dev/sda
    sys_path: PathBuf,       // /sys/block/sda
    controller: Controller,   // SATA, NVMe, USB, etc.
    size: u64,
    sector_size: u32,
    model: String,
    serial: String,
    partitions: Vec<Partition>,
    is_removable: bool,
    is_readonly: bool,
}
```

**Operations:**
```rust
trait DiskOperations {
    fn wipe(&self) -> Result<()>;
    fn create_gpt(&self) -> Result<()>;
    fn create_partition(&self, part: PartitionSpec) -> Result<Partition>;
    fn format(&self, fs_type: FileSystemType) -> Result<()>;
}
```

### 3. ZFS Management

**Pool Creation:**
```rust
struct ZfsPool {
    name: String,
    raid_level: RaidLevel,
    devices: Vec<PathBuf>,
    ashift: u8,
    compression: Compression,
}

enum RaidLevel {
    None,
    Mirror,
    Raidz1,
    Raidz2,
    Raidz3,
}

impl ZfsPool {
    fn create(&self) -> Result<()>;
    fn create_datasets(&self) -> Result<()>;
    fn set_bootfs(&self, dataset: &str) -> Result<()>;
}
```

**Dataset Hierarchy:**
```rust
const DATASETS: &[(&str, &[(&str, &str)])] = &[
    ("ROOT/default", &[("mountpoint", "/")]),
    ("home", &[("mountpoint", "/home")]),
    ("var/log", &[("mountpoint", "/var/log")]),
    // ... etc
];
```

### 4. Configuration State

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    mode: InstallMode,
    pool_name: String,
    devices: Vec<PathBuf>,
    raid_level: RaidLevel,
    efi_size: ByteSize,
    swap_size: ByteSize,
    ashift: Option<u8>,
    compression: Compression,
    hostname: Option<String>,
    dry_run: bool,
    force: bool,

    // Existing mode only
    source_root: Option<PathBuf>,
    exclude_paths: Vec<PathBuf>,
    copy_home: bool,
}

enum InstallMode {
    New,
    Existing,
}
```

### 5. Error Handling

```rust
#[derive(Debug, thiserror::Error)]
enum InstallerError {
    #[error("Device not found: {0}")]
    DeviceNotFound(PathBuf),

    #[error("ZFS command failed: {0}")]
    ZfsError(String),

    #[error("Validation failed: {0}")]
    ValidationError(String),

    #[error("UI error: {0}")]
    UiError(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, InstallerError>;
```

## Dependencies (Cargo.toml)

```toml
[dependencies]
# UI
libnotcurses-sys = "3.0"  # Notcurses bindings

# System
libc = "0.2"              # System calls
nix = "0.27"              # Unix APIs
udev = "0.8"              # Device enumeration
inotify = "0.10"          # Filesystem watching

# Utilities
clap = { version = "4.4", features = ["derive"] }  # CLI args
anyhow = "1.0"            # Error handling
thiserror = "1.0"         # Error derive
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
log = "0.4"
env_logger = "0.11"
regex = "1.10"
bytesize = "1.3"

# Testing
[dev-dependencies]
assert_cmd = "2.0"        # CLI testing
predicates = "3.0"
tempfile = "3.8"
```

## Testing Strategy

### Unit Tests
- Each module has `#[cfg(test)]` blocks
- Mock system calls for disk operations
- Test validation logic thoroughly
- Test error handling paths

### Integration Tests
- Test full workflow in `tests/` directory
- Use mock devices (loopback)
- Test dry-run mode extensively
- Test recovery from errors

### UI Tests
- Test navigation logic
- Test screen transitions
- Mock Notcurses for headless testing

### Safety Tests
- Test that dry-run never writes
- Test that validation catches bad configs
- Test cleanup on failure

## Build System

**Cargo commands:**
```bash
cargo build --release        # Production build
cargo test                   # Run all tests
cargo clippy -- -D warnings  # Linting
cargo fmt                    # Format code
cargo doc --open             # Generate docs
```

**CI Pipeline:**
```yaml
- Check: cargo check
- Test: cargo test --all-features
- Lint: cargo clippy -- -D warnings
- Format: cargo fmt -- --check
- Doc: cargo doc --no-deps
```

## Installation Flow Implementation

### Phase 1: Discovery
1. Initialize Notcurses
2. Scan /sys/class/block for devices
3. Read device properties (size, model, controller)
4. Group by controller (like Growlight)
5. Set up inotify watches for hotplug

### Phase 2: Configuration
1. Show device hierarchy in TUI
2. Allow multi-select for pool devices
3. Configure RAID level (validate minimum devices)
4. Configure pool settings
5. Validate entire configuration

### Phase 3: Execution
1. Confirm with user (show full plan)
2. Wipe selected devices
3. Create GPT partitions (EFI, swap, ZFS)
4. Create ZFS pool with RAID
5. Create dataset hierarchy
6. Install ZFSBootMenu
7. Configure bootloader
8. Create initial snapshot
9. Export/import pool for testing

### Phase 4: Completion
1. Show summary of operations
2. Provide next steps
3. Offer to reboot (if new install)
4. Save installation log

## Migration from Bash

**What we keep:**
- Overall workflow and user experience
- Configuration options
- Safety features (dry-run, validation)
- Support for same distributions

**What we improve:**
- Memory safety (no more bash string bugs)
- Better error handling (no more silent failures)
- Real TUI (not dialog/whiptail)
- Unit tests (hard to do in bash)
- Type safety (catch errors at compile time)
- Dynamic device discovery (inotify)
- Better code organization

## Development Phases

**Phase 1: Core Infrastructure**
- Set up Cargo project
- Implement error types
- Implement config types
- Basic CLI arg parsing

**Phase 2: Disk Management**
- Device discovery via /sys
- Block device representation
- Disk operations (wipe, partition)
- Unit tests

**Phase 3: UI Framework**
- Notcurses initialization
- Basic screen rendering
- Navigation system
- Widget library

**Phase 4: ZFS Integration**
- Pool creation
- Dataset management
- Command wrappers
- Unit tests with mocks

**Phase 5: Bootloader**
- ZFSBootMenu installation
- systemd-boot config
- EFI management

**Phase 6: Integration**
- Connect all components
- Full workflow implementation
- Integration tests

**Phase 7: Testing & Polish**
- Comprehensive test suite
- Documentation
- CI/CD setup
- User testing

## Documentation Requirements

1. **README.md**: Overview, quick start, features
2. **ARCHITECTURE.md**: This document
3. **API.md**: Public API documentation
4. **TESTING.md**: How to run tests, test strategy
5. **DEVELOPMENT.md**: How to contribute, build from source
6. **USER_GUIDE.md**: Detailed usage guide
7. **TROUBLESHOOTING.md**: Common issues and solutions
8. **Inline docs**: Every public fn/struct documented

## Success Criteria

✅ Compiles without warnings
✅ All tests pass
✅ Clippy shows no warnings
✅ Code is properly formatted (rustfmt)
✅ Full API documentation
✅ Integration tests for full workflow
✅ Can install ZFSBootMenu on test VM
✅ Handles errors gracefully
✅ TUI is responsive and intuitive
✅ Supports all RAID levels from bash version
✅ Works on Fedora, Debian, MX Linux
