//! Configuration types and management
//!
//! Defines the configuration state for the installer, including installation mode,
//! device selection, RAID configuration, and all user-configurable options.

use crate::error::{InstallerError, Result};
use bytesize::ByteSize;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Installation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallMode {
    /// Fresh installation on empty drives
    New,
    /// Migrate existing system to ZFS
    Existing,
}

impl std::fmt::Display for InstallMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::New => write!(f, "New Installation"),
            Self::Existing => write!(f, "Migrate Existing System"),
        }
    }
}

/// ZFS RAID level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RaidLevel {
    /// No redundancy (single device or striped)
    None,
    /// Mirror (RAID1)
    Mirror,
    /// RAIDZ1 (single parity)
    Raidz1,
    /// RAIDZ2 (double parity)
    Raidz2,
    /// RAIDZ3 (triple parity)
    Raidz3,
}

impl RaidLevel {
    /// Get minimum number of drives required for this RAID level
    pub fn min_drives(&self) -> usize {
        match self {
            Self::None => 1,
            Self::Mirror => 2,
            Self::Raidz1 => 3,
            Self::Raidz2 => 4,
            Self::Raidz3 => 5,
        }
    }

    /// Get the ZFS vdev type string
    pub fn vdev_type(&self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::Mirror => Some("mirror"),
            Self::Raidz1 => Some("raidz1"),
            Self::Raidz2 => Some("raidz2"),
            Self::Raidz3 => Some("raidz3"),
        }
    }

    /// Get description of RAID level
    pub fn description(&self) -> &'static str {
        match self {
            Self::None => "No redundancy",
            Self::Mirror => "RAID1 - Can lose N-1 drives",
            Self::Raidz1 => "RAID5 equivalent - Can lose 1 drive",
            Self::Raidz2 => "RAID6 equivalent - Can lose 2 drives",
            Self::Raidz3 => "Can lose 3 drives",
        }
    }
}

impl std::fmt::Display for RaidLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Mirror => write!(f, "mirror"),
            Self::Raidz1 => write!(f, "raidz1"),
            Self::Raidz2 => write!(f, "raidz2"),
            Self::Raidz3 => write!(f, "raidz3"),
        }
    }
}

/// ZFS compression algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Compression {
    Off,
    Lz4,
    #[default]
    Zstd,
    Gzip,
    Lzjb,
}

impl std::fmt::Display for Compression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Off => write!(f, "off"),
            Self::Lz4 => write!(f, "lz4"),
            Self::Zstd => write!(f, "zstd"),
            Self::Gzip => write!(f, "gzip"),
            Self::Lzjb => write!(f, "lzjb"),
        }
    }
}

/// Main installer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Installation mode
    pub mode: InstallMode,

    /// ZFS pool name
    pub pool_name: String,

    /// Devices to use for the pool
    pub devices: Vec<PathBuf>,

    /// RAID level
    pub raid_level: RaidLevel,

    /// EFI partition size
    pub efi_size: ByteSize,

    /// Swap partition size (0 to disable)
    pub swap_size: ByteSize,

    /// ZFS ashift value (None = auto-detect)
    pub ashift: Option<u8>,

    /// Compression algorithm
    pub compression: Compression,

    /// Hostname for new installation
    pub hostname: Option<String>,

    /// Dry run mode (don't actually make changes)
    pub dry_run: bool,

    /// Force mode (skip confirmations)
    pub force: bool,

    /// Source root for existing mode
    pub source_root: PathBuf,

    /// Paths to exclude from existing system migration
    pub exclude_paths: Vec<PathBuf>,

    /// Copy home directories in existing mode
    pub copy_home: bool,

    /// Skip pre-flight checks (not recommended)
    pub skip_preflight: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: InstallMode::New,
            pool_name: "zroot".to_string(),
            devices: Vec::new(),
            raid_level: RaidLevel::None,
            efi_size: ByteSize::gib(1),
            swap_size: ByteSize::gib(8),
            ashift: None,
            compression: Compression::default(),
            hostname: None,
            dry_run: false,
            force: false,
            source_root: PathBuf::from("/"),
            exclude_paths: Vec::new(),
            copy_home: true,
            skip_preflight: false,
        }
    }
}

impl Config {
    /// Create a new configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate pool name
        if self.pool_name.is_empty() {
            return Err(InstallerError::validation("Pool name cannot be empty"));
        }
        if !self
            .pool_name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
        {
            return Err(InstallerError::validation(
                "Pool name must contain only alphanumeric characters, underscores, hyphens, and dots",
            ));
        }

        // Validate devices
        if self.devices.is_empty() {
            return Err(InstallerError::validation(
                "At least one device must be selected",
            ));
        }

        // Validate RAID level vs device count
        let min_drives = self.raid_level.min_drives();
        if self.devices.len() < min_drives {
            return Err(InstallerError::validation(format!(
                "RAID level {} requires at least {} device(s), but only {} provided",
                self.raid_level,
                min_drives,
                self.devices.len()
            )));
        }

        // Validate ashift
        if let Some(ashift) = self.ashift {
            if !(9..=16).contains(&ashift) {
                return Err(InstallerError::validation(format!(
                    "ashift must be between 9 and 16, got {}",
                    ashift
                )));
            }
        }

        // Validate EFI size
        if self.efi_size < ByteSize::mib(100) {
            return Err(InstallerError::validation(
                "EFI partition must be at least 100MB",
            ));
        }

        // Validate source root for existing mode
        if self.mode == InstallMode::Existing && !self.source_root.exists() {
            return Err(InstallerError::validation(format!(
                "Source root does not exist: {}",
                self.source_root.display()
            )));
        }

        Ok(())
    }

    /// Get the total number of partitions that will be created per device
    pub fn partitions_per_device(&self) -> usize {
        let mut count = 2; // EFI + ZFS
        if self.swap_size > ByteSize(0) {
            count += 1;
        }
        count
    }

    /// Calculate estimated total size needed per device
    pub fn min_device_size(&self) -> ByteSize {
        self.efi_size + self.swap_size + ByteSize::gib(10) // 10GB minimum for ZFS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raid_level_min_drives() {
        assert_eq!(RaidLevel::None.min_drives(), 1);
        assert_eq!(RaidLevel::Mirror.min_drives(), 2);
        assert_eq!(RaidLevel::Raidz1.min_drives(), 3);
        assert_eq!(RaidLevel::Raidz2.min_drives(), 4);
        assert_eq!(RaidLevel::Raidz3.min_drives(), 5);
    }

    #[test]
    fn test_config_validation_empty_devices() {
        let config = Config::default();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_raid_mismatch() {
        let mut config = Config::default();
        config.devices = vec![PathBuf::from("/dev/sda")];
        config.raid_level = RaidLevel::Mirror; // Needs 2+ devices
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_valid() {
        let mut config = Config::default();
        config.devices = vec![PathBuf::from("/dev/sda"), PathBuf::from("/dev/sdb")];
        config.raid_level = RaidLevel::Mirror;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_invalid_pool_name() {
        let mut config = Config::default();
        config.pool_name = "pool/name".to_string(); // Invalid character
        config.devices = vec![PathBuf::from("/dev/sda")];
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_compression_display() {
        assert_eq!(Compression::Zstd.to_string(), "zstd");
        assert_eq!(Compression::Lz4.to_string(), "lz4");
    }
}
