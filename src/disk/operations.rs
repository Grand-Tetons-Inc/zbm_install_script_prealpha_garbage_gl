//! Disk operations: partitioning, formatting, wiping
//!
//! Provides safe wrappers around disk manipulation commands.

use crate::disk::block_device::BlockDevice;
use crate::error::{InstallerError, Result};
use bytesize::ByteSize;
use std::path::PathBuf;
use std::process::Command;

/// Partition specification
#[derive(Debug, Clone)]
pub struct PartitionSpec {
    /// Partition number
    pub number: u32,
    /// Start sector/offset
    pub start: String,
    /// End sector/offset (or size)
    pub end: String,
    /// Partition type GUID (for GPT)
    pub type_guid: Option<String>,
    /// Partition name
    pub name: Option<String>,
}

/// Disk operations manager
pub struct DiskOperations {
    /// Dry run mode - don't actually execute commands
    dry_run: bool,
}

impl DiskOperations {
    /// Create a new disk operations manager
    pub fn new(dry_run: bool) -> Self {
        Self { dry_run }
    }

    /// Execute a command, respecting dry-run mode
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
            return Err(InstallerError::CommandFailed {
                cmd: cmd_str,
                code: output.status.code().unwrap_or(-1),
                stderr: stderr.to_string(),
            });
        }

        Ok(output)
    }

    /// Wipe all data from a device
    pub fn wipe_device(&self, device: &BlockDevice) -> Result<()> {
        log::info!("Wiping device: {}", device.path.display());

        // Use wipefs to remove filesystem signatures
        self.execute(Command::new("wipefs").arg("-a").arg(&device.path))?;

        // Use sgdisk to zap GPT and MBR
        self.execute(Command::new("sgdisk").arg("--zap-all").arg(&device.path))?;

        // Wait for kernel to update
        self.execute(Command::new("partprobe").arg(&device.path))?;

        Ok(())
    }

    /// Create a GPT partition table
    pub fn create_gpt(&self, device: &BlockDevice) -> Result<()> {
        log::info!("Creating GPT on device: {}", device.path.display());

        self.execute(Command::new("sgdisk").arg("--clear").arg(&device.path))?;

        Ok(())
    }

    /// Create a partition
    pub fn create_partition(&self, device: &BlockDevice, spec: &PartitionSpec) -> Result<PathBuf> {
        log::info!(
            "Creating partition {} on {}: {} - {}",
            spec.number,
            device.path.display(),
            spec.start,
            spec.end
        );

        let mut cmd = Command::new("sgdisk");
        cmd.arg(&device.path);

        // Add partition with start and end
        cmd.arg(format!("--new={}:{}:{}", spec.number, spec.start, spec.end));

        // Set partition type if specified
        if let Some(ref type_guid) = spec.type_guid {
            cmd.arg(format!("--typecode={}:{}", spec.number, type_guid));
        }

        // Set partition name if specified
        if let Some(ref name) = spec.name {
            cmd.arg(format!("--change-name={}:{}", spec.number, name));
        }

        self.execute(&mut cmd)?;

        // Wait for kernel to update
        self.execute(Command::new("partprobe").arg(&device.path))?;

        // Construct partition path
        let partition_path = if device.name.starts_with("nvme") {
            PathBuf::from(format!("{}p{}", device.path.display(), spec.number))
        } else {
            PathBuf::from(format!("{}{}", device.path.display(), spec.number))
        };

        Ok(partition_path)
    }

    /// Create standard ZBM partitions on a device
    pub fn create_zbm_partitions(
        &self,
        device: &BlockDevice,
        efi_size: ByteSize,
        swap_size: ByteSize,
    ) -> Result<ZbmPartitions> {
        log::info!("Creating ZBM partitions on {}", device.path.display());

        // Wipe device first
        self.wipe_device(device)?;

        // Create GPT
        self.create_gpt(device)?;

        // Partition 1: EFI System Partition
        let efi_spec = PartitionSpec {
            number: 1,
            start: "1MiB".to_string(),
            end: format!("+{}MiB", efi_size.0 / (1024 * 1024)),
            type_guid: Some("EF00".to_string()), // EFI System
            name: Some("EFI".to_string()),
        };
        let efi_path = self.create_partition(device, &efi_spec)?;

        // Partition 2: Swap (if size > 0)
        let swap_path = if swap_size.0 > 0 {
            let swap_spec = PartitionSpec {
                number: 2,
                start: "0".to_string(), // Auto-start after previous
                end: format!("+{}GiB", swap_size.0 / (1024 * 1024 * 1024)),
                type_guid: Some("8200".to_string()), // Linux swap
                name: Some("swap".to_string()),
            };
            Some(self.create_partition(device, &swap_spec)?)
        } else {
            None
        };

        // Partition 3 (or 2 if no swap): ZFS pool
        let zfs_part_num = if swap_path.is_some() { 3 } else { 2 };
        let zfs_spec = PartitionSpec {
            number: zfs_part_num,
            start: "0".to_string(),
            end: "0".to_string(),                // Use remaining space
            type_guid: Some("BF00".to_string()), // Solaris root (ZFS)
            name: Some("zfs".to_string()),
        };
        let zfs_path = self.create_partition(device, &zfs_spec)?;

        Ok(ZbmPartitions {
            efi: efi_path,
            swap: swap_path,
            zfs: zfs_path,
        })
    }

    /// Format a partition as FAT32 (for EFI)
    pub fn format_efi(&self, partition: &PathBuf) -> Result<()> {
        log::info!("Formatting EFI partition: {}", partition.display());

        self.execute(
            Command::new("mkfs.vfat")
                .arg("-F32")
                .arg("-n")
                .arg("EFI")
                .arg(partition),
        )?;

        Ok(())
    }

    /// Create swap on a partition
    pub fn create_swap(&self, partition: &PathBuf) -> Result<()> {
        log::info!("Creating swap on: {}", partition.display());

        self.execute(Command::new("mkswap").arg(partition))?;

        Ok(())
    }
}

/// Result of creating ZBM partitions
#[derive(Debug)]
pub struct ZbmPartitions {
    /// EFI system partition path
    pub efi: PathBuf,
    /// Swap partition path (None if disabled)
    pub swap: Option<PathBuf>,
    /// ZFS partition path
    pub zfs: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_operations_dry_run() {
        let ops = DiskOperations::new(true);
        assert!(ops.dry_run);
    }

    #[test]
    fn test_partition_spec_creation() {
        let spec = PartitionSpec {
            number: 1,
            start: "1MiB".to_string(),
            end: "+512MiB".to_string(),
            type_guid: Some("EF00".to_string()),
            name: Some("EFI".to_string()),
        };

        assert_eq!(spec.number, 1);
        assert_eq!(spec.type_guid.unwrap(), "EF00");
    }
}
