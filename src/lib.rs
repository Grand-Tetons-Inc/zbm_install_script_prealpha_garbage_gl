//! ZFSBootMenu Installer Library
//!
//! A comprehensive installer for ZFSBootMenu with support for:
//! - Multiple drives and RAID configurations
//! - Interactive TUI (Notcurses-based)
//! - CLI mode
//! - Existing system migration
//!
//! # Architecture
//!
//! The installer is organized into modules:
//! - `config`: Configuration types and validation
//! - `disk`: Device discovery and disk operations (inspired by Growlight)
//! - `zfs`: ZFS pool and dataset management
//! - `bootloader`: ZFSBootMenu and bootloader installation
//! - `system`: Distribution detection and package management
//! - `validation`: Pre-flight validation checks
//! - `ui`: TUI framework (Notcurses-based)
//! - `error`: Error types and handling
//!
//! # Example
//!
//! ```rust,no_run
//! use zbm_installer::*;
//!
//! # fn main() -> Result<()> {
//! // Create configuration
//! let mut config = Config::default();
//! config.mode = InstallMode::New;
//! config.devices = vec!["/dev/sda".into(), "/dev/sdb".into()];
//! config.raid_level = RaidLevel::Mirror;
//!
//! // Validate
//! let validator = Validator::new(config.clone());
//! let validation = validator.validate()?;
//! if !validation.is_ok() {
//!     eprintln!("Validation failed!");
//!     return Err(InstallerError::validation("See errors above"));
//! }
//!
//! // Run installer
//! let installer = Installer::new(config)?;
//! installer.install()?;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod bootloader;
pub mod config;
pub mod disk;
pub mod error;
pub mod system;
pub mod ui;
pub mod validation;
pub mod zfs;

// Re-export commonly used types
pub use config::{Compression, Config, InstallMode, RaidLevel};
pub use disk::{BlockDevice, DeviceDiscovery, DiskOperations};
pub use error::{InstallerError, Result};
pub use validation::{ValidationResult, Validator};
pub use zfs::{DatasetManager, ZfsPool};

use bootloader::{SystemdBoot, ZbmInstaller};
use disk::ZbmPartitions;
use std::fs;
use std::path::PathBuf;

/// Main installer orchestrator
pub struct Installer {
    config: Config,
}

impl Installer {
    /// Create a new installer with the given configuration
    pub fn new(config: Config) -> Result<Self> {
        // Validate configuration
        config.validate()?;

        Ok(Self { config })
    }

    /// Run the installation
    pub fn install(&self) -> Result<()> {
        log::info!("Starting ZFSBootMenu installation");
        log::info!("Mode: {}", self.config.mode);
        log::info!("Pool: {}", self.config.pool_name);
        log::info!("RAID: {}", self.config.raid_level);
        log::info!("Devices: {}", self.config.devices.len());

        // Check if dry run
        if self.config.dry_run {
            log::warn!("DRY RUN MODE - No changes will be made");
        }

        // Phase 1: Validate
        self.validate()?;

        // Phase 2: Prepare disks
        let partitions = self.prepare_disks()?;

        // Phase 3: Create ZFS pool and datasets
        self.create_zfs(&partitions)?;

        // Phase 4: Mount and prepare filesystem
        let mount_point = self.mount_filesystem()?;

        // Phase 5: Install system (if existing mode) or setup environment
        if self.config.mode == InstallMode::Existing {
            self.migrate_system(&mount_point)?;
        }

        // Phase 6: Install bootloader
        self.install_bootloader(&partitions)?;

        // Phase 7: Finalize
        self.finalize()?;

        log::info!("Installation completed successfully!");
        Ok(())
    }

    /// Validate configuration and system
    fn validate(&self) -> Result<()> {
        log::info!("Phase 1: Validation");

        if !self.config.skip_preflight {
            let validator = Validator::new(self.config.clone());
            let result = validator.validate()?;

            for warning in &result.warnings {
                log::warn!("Warning: {}", warning);
            }

            if !result.is_ok() {
                for error in &result.errors {
                    log::error!("Error: {}", error);
                }
                return Err(InstallerError::validation("Pre-flight checks failed"));
            }
        }

        Ok(())
    }

    /// Prepare disks (partition, format)
    fn prepare_disks(&self) -> Result<Vec<ZbmPartitions>> {
        log::info!("Phase 2: Preparing disks");

        let disk_ops = DiskOperations::new(self.config.dry_run);
        let discovery = DeviceDiscovery::new()?;

        let mut all_partitions = Vec::new();

        for device_path in &self.config.devices {
            let device_name = device_path
                .file_name()
                .ok_or_else(|| InstallerError::DeviceNotFound(device_path.clone()))?
                .to_string_lossy()
                .to_string();

            let device = discovery.find_device(&device_name)?;
            log::info!("Preparing device: {}", device.display_name());

            let partitions = disk_ops.create_zbm_partitions(
                &device,
                self.config.efi_size,
                self.config.swap_size,
            )?;

            // Format EFI partition
            disk_ops.format_efi(&partitions.efi)?;

            // Create swap if enabled
            if let Some(ref swap) = partitions.swap {
                disk_ops.create_swap(swap)?;
            }

            all_partitions.push(partitions);
        }

        Ok(all_partitions)
    }

    /// Create ZFS pool and datasets
    fn create_zfs(&self, partitions: &[ZbmPartitions]) -> Result<()> {
        log::info!("Phase 3: Creating ZFS pool");

        // Collect ZFS partition paths
        let zfs_devices: Vec<PathBuf> = partitions.iter().map(|p| p.zfs.clone()).collect();

        // Create pool
        let pool = ZfsPool::new(
            self.config.pool_name.clone(),
            self.config.raid_level,
            zfs_devices,
            self.config.ashift,
            self.config.compression,
            self.config.dry_run,
        );

        pool.create()?;

        // Create datasets
        let dataset_manager =
            DatasetManager::new(self.config.pool_name.clone(), self.config.dry_run);
        dataset_manager.create_zbm_datasets()?;

        Ok(())
    }

    /// Mount filesystem
    fn mount_filesystem(&self) -> Result<PathBuf> {
        log::info!("Phase 4: Mounting filesystem");

        let mount_point = PathBuf::from("/mnt");

        if !self.config.dry_run {
            // Mount ROOT/default
            let dataset_manager = DatasetManager::new(self.config.pool_name.clone(), false);
            dataset_manager.mount("ROOT/default")?;

            // Mount other datasets (they should auto-mount based on mountpoint property)
        }

        Ok(mount_point)
    }

    /// Migrate existing system
    fn migrate_system(&self, _mount_point: &PathBuf) -> Result<()> {
        log::info!("Phase 5: Migrating existing system");

        // TODO: Implement rsync-based system migration
        log::warn!("System migration not yet implemented");

        Ok(())
    }

    /// Install bootloader
    fn install_bootloader(&self, _partitions: &[ZbmPartitions]) -> Result<()> {
        log::info!("Phase 6: Installing bootloader");

        // Mount EFI partition
        let efi_mount = PathBuf::from("/mnt/boot/efi");
        if !self.config.dry_run {
            fs::create_dir_all(&efi_mount)?;
            // Mount first EFI partition
            // TODO: Proper mounting with nix crate
        }

        // Install ZFSBootMenu
        let zbm_installer = ZbmInstaller::new(
            self.config.pool_name.clone(),
            efi_mount.clone(),
            self.config.dry_run,
        );
        zbm_installer.install()?;

        // Install systemd-boot
        let systemd_boot = SystemdBoot::new(efi_mount, self.config.dry_run);
        systemd_boot.install()?;

        Ok(())
    }

    /// Finalize installation
    fn finalize(&self) -> Result<()> {
        log::info!("Phase 7: Finalizing");

        // Set bootfs property
        let pool = ZfsPool::new(
            self.config.pool_name.clone(),
            self.config.raid_level,
            Vec::new(),
            None,
            self.config.compression,
            self.config.dry_run,
        );
        pool.set_bootfs("ROOT/default")?;

        // Create initial snapshot
        let dataset_manager =
            DatasetManager::new(self.config.pool_name.clone(), self.config.dry_run);
        dataset_manager.snapshot("ROOT/default", "initial")?;

        // Sync
        system::sync()?;

        log::info!("Installation finalized");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_installer_creation_requires_valid_config() {
        let config = Config::default(); // Empty devices
        let result = Installer::new(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_installer_with_dry_run() {
        let mut config = Config::default();
        config.devices = vec![PathBuf::from("/dev/sda")];
        config.dry_run = true;

        let result = Installer::new(config);
        assert!(result.is_ok());
    }
}
