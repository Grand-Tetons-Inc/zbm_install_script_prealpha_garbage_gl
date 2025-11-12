//! ZFSBootMenu Installer - Main Entry Point
//!
//! CLI and TUI installer for ZFSBootMenu with RAID support.

use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use std::process;
use zbm_installer::*;

/// ZFSBootMenu Installer
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(after_help = "\
EXAMPLES:
    # Single drive installation
    zbm-installer --mode new --drives /dev/sda

    # Mirrored drives
    zbm-installer --mode new --drives /dev/sda,/dev/sdb --raid mirror

    # RAIDZ1 with custom settings
    zbm-installer --mode new --drives /dev/sda,/dev/sdb,/dev/sdc \\
        --raid raidz1 --pool-name mytank --compression lz4

    # Dry run (recommended for testing)
    zbm-installer --mode new --drives /dev/sda,/dev/sdb --raid mirror --dry-run

    # Interactive TUI mode
    zbm-installer --tui
")]
struct Args {
    /// Installation mode: new or existing
    #[arg(short, long, value_enum)]
    mode: Option<InstallModeArg>,

    /// Comma-separated list of drives (e.g., /dev/sda,/dev/sdb)
    #[arg(short, long, value_delimiter = ',')]
    drives: Vec<PathBuf>,

    /// ZFS pool name
    #[arg(short, long, default_value = "zroot")]
    pool_name: String,

    /// RAID level
    #[arg(short, long, value_enum, default_value = "none")]
    raid: RaidLevelArg,

    /// EFI partition size (e.g., 512M, 1G)
    #[arg(short, long, default_value = "1G")]
    efi_size: String,

    /// Swap partition size (0 to disable, e.g., 8G, 16G)
    #[arg(short, long, default_value = "8G")]
    swap_size: String,

    /// ZFS ashift value (9-16, auto-detect if not specified)
    #[arg(short, long)]
    ashift: Option<u8>,

    /// ZFS compression algorithm
    #[arg(short, long, value_enum, default_value = "zstd")]
    compression: CompressionArg,

    /// Hostname for new installation
    #[arg(short = 'H', long)]
    hostname: Option<String>,

    /// Source root for existing mode
    #[arg(long, default_value = "/")]
    source_root: PathBuf,

    /// Paths to exclude (can be used multiple times)
    #[arg(long)]
    exclude: Vec<PathBuf>,

    /// Don't copy home directories in existing mode
    #[arg(long)]
    no_copy_home: bool,

    /// Dry run - show what would be done without making changes
    #[arg(short = 'n', long)]
    dry_run: bool,

    /// Force mode - skip confirmations
    #[arg(short, long)]
    force: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Skip pre-flight system checks
    #[arg(short = 'S', long)]
    skip_preflight: bool,

    /// Launch interactive TUI
    #[arg(short, long)]
    tui: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum InstallModeArg {
    New,
    Existing,
}

impl From<InstallModeArg> for InstallMode {
    fn from(mode: InstallModeArg) -> Self {
        match mode {
            InstallModeArg::New => InstallMode::New,
            InstallModeArg::Existing => InstallMode::Existing,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum RaidLevelArg {
    None,
    Mirror,
    Raidz1,
    Raidz2,
    Raidz3,
}

impl From<RaidLevelArg> for RaidLevel {
    fn from(raid: RaidLevelArg) -> Self {
        match raid {
            RaidLevelArg::None => RaidLevel::None,
            RaidLevelArg::Mirror => RaidLevel::Mirror,
            RaidLevelArg::Raidz1 => RaidLevel::Raidz1,
            RaidLevelArg::Raidz2 => RaidLevel::Raidz2,
            RaidLevelArg::Raidz3 => RaidLevel::Raidz3,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CompressionArg {
    Off,
    Lz4,
    Zstd,
    Gzip,
    Lzjb,
}

impl From<CompressionArg> for Compression {
    fn from(comp: CompressionArg) -> Self {
        match comp {
            CompressionArg::Off => Compression::Off,
            CompressionArg::Lz4 => Compression::Lz4,
            CompressionArg::Zstd => Compression::Zstd,
            CompressionArg::Gzip => Compression::Gzip,
            CompressionArg::Lzjb => Compression::Lzjb,
        }
    }
}

fn parse_size(size_str: &str) -> Result<bytesize::ByteSize> {
    size_str
        .parse()
        .map_err(|e| InstallerError::ParseError(format!("Invalid size '{}': {}", size_str, e)))
}

fn main() {
    // Parse arguments
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.verbose { "debug" } else { "info" };

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();

    // Check root privileges
    if !system::is_root() {
        eprintln!("Error: This program must be run as root");
        eprintln!("Please run: sudo {}", std::env::args().next().unwrap());
        process::exit(1);
    }

    // Run installer
    let result = if args.tui {
        run_tui(args)
    } else {
        run_cli(args)
    };

    // Handle result
    match result {
        Ok(()) => {
            log::info!("Installation completed successfully!");
            process::exit(0);
        }
        Err(e) => {
            log::error!("Installation failed: {}", e);
            process::exit(1);
        }
    }
}

fn run_cli(args: Args) -> Result<()> {
    log::info!("ZFSBootMenu Installer - CLI Mode");

    // Validate required arguments
    if args.mode.is_none() {
        return Err(InstallerError::config(
            "Installation mode is required. Use --mode new or --mode existing",
        ));
    }

    if args.drives.is_empty() {
        return Err(InstallerError::config(
            "At least one drive must be specified with --drives",
        ));
    }

    // Build configuration
    let mut config = Config::new();
    config.mode = args.mode.unwrap().into();
    config.devices = args.drives;
    config.pool_name = args.pool_name;
    config.raid_level = args.raid.into();
    config.efi_size = parse_size(&args.efi_size)?;
    config.swap_size = parse_size(&args.swap_size)?;
    config.ashift = args.ashift;
    config.compression = args.compression.into();
    config.hostname = args.hostname;
    config.dry_run = args.dry_run;
    config.force = args.force;
    config.source_root = args.source_root;
    config.exclude_paths = args.exclude;
    config.copy_home = !args.no_copy_home;
    config.skip_preflight = args.skip_preflight;

    // Display configuration
    log::info!("Configuration:");
    log::info!("  Mode: {}", config.mode);
    log::info!("  Pool: {}", config.pool_name);
    log::info!(
        "  RAID: {} ({})",
        config.raid_level,
        config.raid_level.description()
    );
    log::info!("  Devices: {}", config.devices.len());
    for device in &config.devices {
        log::info!("    - {}", device.display());
    }
    log::info!("  EFI size: {}", config.efi_size);
    log::info!("  Swap size: {}", config.swap_size);
    log::info!("  Compression: {}", config.compression);
    if config.dry_run {
        log::warn!("  DRY RUN MODE - No changes will be made");
    }

    // Confirm unless force mode
    if !config.force && !config.dry_run {
        println!("\n⚠️  WARNING: This will DESTROY all data on the selected drives!");
        println!("Continue? (yes/no): ");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        if input.trim().to_lowercase() != "yes" {
            println!("Installation cancelled.");
            return Ok(());
        }
    }

    // Create and run installer
    let installer = Installer::new(config)?;
    installer.install()?;

    Ok(())
}

fn run_tui(args: Args) -> Result<()> {
    log::info!("ZFSBootMenu Installer - TUI Mode");

    // Build base configuration from CLI args (if any)
    let mut config = Config::new();
    if let Some(mode) = args.mode {
        config.mode = mode.into();
    }
    if !args.drives.is_empty() {
        config.devices = args.drives;
    }
    config.pool_name = args.pool_name;
    config.raid_level = args.raid.into();
    config.dry_run = args.dry_run;

    // Launch TUI
    let mut ui = ui::UiManager::new(config);
    let final_config = ui.run()?;

    // Run installation with TUI-configured settings
    let installer = Installer::new(final_config)?;
    installer.install()?;

    Ok(())
}
