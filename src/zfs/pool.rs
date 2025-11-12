//! ZFS pool creation and management

use crate::config::{Compression, RaidLevel};
use crate::error::{InstallerError, Result};
use std::path::PathBuf;
use std::process::Command;

/// ZFS pool manager
pub struct ZfsPool {
    /// Pool name
    name: String,
    /// RAID level
    raid_level: RaidLevel,
    /// Devices to use
    devices: Vec<PathBuf>,
    /// ashift value
    ashift: Option<u8>,
    /// Compression algorithm
    compression: Compression,
    /// Dry run mode
    dry_run: bool,
}

impl ZfsPool {
    /// Create a new ZFS pool configuration
    pub fn new(
        name: String,
        raid_level: RaidLevel,
        devices: Vec<PathBuf>,
        ashift: Option<u8>,
        compression: Compression,
        dry_run: bool,
    ) -> Self {
        Self {
            name,
            raid_level,
            devices,
            ashift,
            compression,
            dry_run,
        }
    }

    /// Execute a command
    fn execute(&self, cmd: &mut Command) -> Result<std::process::Output> {
        let cmd_str = format!("{:?}", cmd);

        if self.dry_run {
            log::info!("[DRY RUN] Would execute: {}", cmd_str);
            return Ok(std::process::Output {
                status: std::process::ExitStatus::default(),
                stdout: Vec::new(),
                stderr: Vec::new(),
            });
        }

        log::debug!("Executing: {}", cmd_str);
        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(InstallerError::ZfsError {
                operation: cmd_str,
                details: stderr.to_string(),
            });
        }

        Ok(output)
    }

    /// Create the ZFS pool
    pub fn create(&self) -> Result<()> {
        log::info!("Creating ZFS pool: {}", self.name);

        let mut cmd = Command::new("zpool");
        cmd.arg("create")
            .arg("-f") // Force
            .arg("-m")
            .arg("none"); // Don't mount automatically

        // Add pool features
        cmd.arg("-o").arg("ashift={}".to_string());
        if let Some(ashift) = self.ashift {
            cmd.arg("-o").arg(format!("ashift={}", ashift));
        }

        // Pool properties
        cmd.arg("-O")
            .arg("acltype=posixacl")
            .arg("-O")
            .arg("xattr=sa")
            .arg("-O")
            .arg("dnodesize=auto")
            .arg("-O")
            .arg(format!("compression={}", self.compression))
            .arg("-O")
            .arg("normalization=formD")
            .arg("-O")
            .arg("relatime=on");

        // Pool name
        cmd.arg(&self.name);

        // Add vdev type if not "none"
        if let Some(vdev_type) = self.raid_level.vdev_type() {
            cmd.arg(vdev_type);
        }

        // Add devices
        for device in &self.devices {
            cmd.arg(device);
        }

        self.execute(&mut cmd)?;
        log::info!("ZFS pool {} created successfully", self.name);

        Ok(())
    }

    /// Destroy the pool (for testing/cleanup)
    pub fn destroy(&self) -> Result<()> {
        log::info!("Destroying ZFS pool: {}", self.name);

        self.execute(
            Command::new("zpool")
                .arg("destroy")
                .arg("-f")
                .arg(&self.name),
        )?;

        Ok(())
    }

    /// Export the pool
    pub fn export(&self) -> Result<()> {
        log::info!("Exporting ZFS pool: {}", self.name);

        self.execute(Command::new("zpool").arg("export").arg(&self.name))?;

        Ok(())
    }

    /// Import the pool
    pub fn import(&self) -> Result<()> {
        log::info!("Importing ZFS pool: {}", self.name);

        self.execute(
            Command::new("zpool")
                .arg("import")
                .arg("-f")
                .arg(&self.name),
        )?;

        Ok(())
    }

    /// Set bootfs property
    pub fn set_bootfs(&self, dataset: &str) -> Result<()> {
        log::info!("Setting bootfs to: {}/{}", self.name, dataset);

        self.execute(
            Command::new("zpool")
                .arg("set")
                .arg(format!("bootfs={}/{}", self.name, dataset))
                .arg(&self.name),
        )?;

        Ok(())
    }

    /// Get pool status
    pub fn status(&self) -> Result<String> {
        let output = self.execute(Command::new("zpool").arg("status").arg(&self.name))?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Check if pool exists
    pub fn exists(&self) -> bool {
        Command::new("zpool")
            .arg("list")
            .arg(&self.name)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zfs_pool_creation() {
        let pool = ZfsPool::new(
            "testpool".to_string(),
            RaidLevel::None,
            vec![PathBuf::from("/dev/sda3")],
            Some(12),
            Compression::Zstd,
            true, // dry run
        );

        assert_eq!(pool.name, "testpool");
        assert_eq!(pool.raid_level, RaidLevel::None);
    }

    #[test]
    fn test_pool_exists_nonexistent() {
        let pool = ZfsPool::new(
            "nonexistent_pool_xyz123".to_string(),
            RaidLevel::None,
            vec![],
            None,
            Compression::Zstd,
            false,
        );

        // This should return false for a pool that doesn't exist
        assert!(!pool.exists());
    }
}
