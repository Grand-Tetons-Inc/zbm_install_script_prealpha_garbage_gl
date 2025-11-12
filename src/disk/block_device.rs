//! Block device representation
//!
//! Represents a physical or virtual block device with all relevant properties.

use crate::error::{InstallerError, Result};
use std::fs;
use std::path::PathBuf;

/// Represents a partition on a block device
#[derive(Debug, Clone)]
pub struct Partition {
    /// Partition device path (e.g., /dev/sda1)
    pub path: PathBuf,
    /// Partition number
    pub number: u32,
    /// Partition size in bytes
    pub size: u64,
    /// Filesystem type (if any)
    pub fstype: Option<String>,
    /// Mount point (if mounted)
    pub mountpoint: Option<PathBuf>,
}

/// Storage controller type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerType {
    Sata,
    Nvme,
    Scsi,
    Usb,
    Mmc,
    Virtual,
    Unknown,
}

impl std::fmt::Display for ControllerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sata => write!(f, "SATA"),
            Self::Nvme => write!(f, "NVMe"),
            Self::Scsi => write!(f, "SCSI"),
            Self::Usb => write!(f, "USB"),
            Self::Mmc => write!(f, "MMC"),
            Self::Virtual => write!(f, "Virtual"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Represents a block device
#[derive(Debug, Clone)]
pub struct BlockDevice {
    /// Device name (e.g., sda, nvme0n1)
    pub name: String,
    /// Device path (e.g., /dev/sda)
    pub path: PathBuf,
    /// Sysfs path (e.g., /sys/block/sda)
    pub sys_path: PathBuf,
    /// Controller type
    pub controller_type: ControllerType,
    /// Device size in bytes
    pub size: u64,
    /// Logical sector size
    pub logical_block_size: u32,
    /// Physical sector size
    pub physical_block_size: u32,
    /// Device model
    pub model: Option<String>,
    /// Device serial number
    pub serial: Option<String>,
    /// Device vendor
    pub vendor: Option<String>,
    /// Is device removable
    pub removable: bool,
    /// Is device read-only
    pub readonly: bool,
    /// Is device rotational (HDD vs SSD)
    pub rotational: bool,
    /// Partitions on this device
    pub partitions: Vec<Partition>,
}

impl BlockDevice {
    /// Create a new BlockDevice from a device name (e.g., "sda")
    pub fn from_name(name: &str) -> Result<Self> {
        let path = PathBuf::from(format!("/dev/{}", name));
        let sys_path = PathBuf::from(format!("/sys/block/{}", name));

        if !sys_path.exists() {
            return Err(InstallerError::DeviceNotFound(path));
        }

        // Read size (in 512-byte sectors)
        let size = Self::read_sys_value(&sys_path, "size")?
            .parse::<u64>()
            .map_err(|e| InstallerError::ParseError(e.to_string()))?
            * 512;

        // Read sector sizes
        let logical_block_size = Self::read_sys_value(&sys_path, "queue/logical_block_size")?
            .parse::<u32>()
            .map_err(|e| InstallerError::ParseError(e.to_string()))?;

        let physical_block_size = Self::read_sys_value(&sys_path, "queue/physical_block_size")?
            .parse::<u32>()
            .map_err(|e| InstallerError::ParseError(e.to_string()))?;

        // Read device properties
        let removable =
            Self::read_sys_value(&sys_path, "removable").unwrap_or_else(|_| "0".to_string()) == "1";
        let readonly =
            Self::read_sys_value(&sys_path, "ro").unwrap_or_else(|_| "0".to_string()) == "1";
        let rotational = Self::read_sys_value(&sys_path, "queue/rotational")
            .unwrap_or_else(|_| "1".to_string())
            == "1";

        // Try to read model, vendor, serial (may not exist for all devices)
        let model = Self::read_sys_value(&sys_path, "device/model").ok();
        let vendor = Self::read_sys_value(&sys_path, "device/vendor").ok();
        let serial = Self::read_sys_value(&sys_path, "device/serial").ok();

        // Determine controller type
        let controller_type = Self::detect_controller_type(name);

        // Discover partitions
        let partitions = Self::discover_partitions(&sys_path, name)?;

        Ok(Self {
            name: name.to_string(),
            path,
            sys_path,
            controller_type,
            size,
            logical_block_size,
            physical_block_size,
            model,
            serial,
            vendor,
            removable,
            readonly,
            rotational,
            partitions,
        })
    }

    /// Read a value from sysfs
    fn read_sys_value(sys_path: &PathBuf, attr: &str) -> Result<String> {
        let path = sys_path.join(attr);
        fs::read_to_string(&path)
            .map(|s| s.trim().to_string())
            .map_err(|e| InstallerError::Io(e))
    }

    /// Detect controller type from device name
    fn detect_controller_type(name: &str) -> ControllerType {
        if name.starts_with("nvme") {
            ControllerType::Nvme
        } else if name.starts_with("sd") {
            // Could be SATA, SCSI, or USB - need more investigation
            // For now, default to SATA
            ControllerType::Sata
        } else if name.starts_with("mmcblk") {
            ControllerType::Mmc
        } else if name.starts_with("vd") || name.starts_with("loop") {
            ControllerType::Virtual
        } else {
            ControllerType::Unknown
        }
    }

    /// Discover partitions on this device
    fn discover_partitions(sys_path: &PathBuf, device_name: &str) -> Result<Vec<Partition>> {
        let mut partitions = Vec::new();

        // Read partition entries from sysfs
        if let Ok(entries) = fs::read_dir(sys_path) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();

                // Check if this is a partition (starts with device name + number)
                if name_str.starts_with(device_name) && name_str.len() > device_name.len() {
                    let suffix = &name_str[device_name.len()..];
                    // For NVMe, partitions are like nvme0n1p1, for others like sda1
                    let part_num_str = if device_name.starts_with("nvme") {
                        suffix.strip_prefix('p').unwrap_or(suffix)
                    } else {
                        suffix
                    };

                    if let Ok(part_num) = part_num_str.parse::<u32>() {
                        let part_path = PathBuf::from(format!("/dev/{}", name_str));
                        let part_sys_path = sys_path.join(name_str.as_ref());

                        // Read partition size
                        let size = Self::read_sys_value(&part_sys_path, "size")
                            .ok()
                            .and_then(|s| s.parse::<u64>().ok())
                            .unwrap_or(0)
                            * 512;

                        partitions.push(Partition {
                            path: part_path,
                            number: part_num,
                            size,
                            fstype: None,     // Would need blkid to determine
                            mountpoint: None, // Would need to parse /proc/mounts
                        });
                    }
                }
            }
        }

        // Sort partitions by number
        partitions.sort_by_key(|p| p.number);
        Ok(partitions)
    }

    /// Check if device is currently mounted
    pub fn is_mounted(&self) -> Result<bool> {
        let mounts = fs::read_to_string("/proc/mounts")?;
        Ok(mounts.lines().any(|line| {
            line.starts_with(&format!("{} ", self.path.display()))
                || self
                    .partitions
                    .iter()
                    .any(|p| line.starts_with(&format!("{} ", p.path.display())))
        }))
    }

    /// Check if device is part of a ZFS pool
    pub fn is_in_zfs_pool(&self) -> Result<bool> {
        // Check if zpool.cache exists and contains this device
        // This is a simplified check; a full implementation would parse zpool.cache
        // or run `zpool status -v` and check output
        // For now, we'll just check if any partition has zfs_member filesystem
        Ok(false) // TODO: Implement proper ZFS check
    }

    /// Check if device is suitable for installation
    pub fn is_suitable(&self) -> Result<()> {
        if self.readonly {
            return Err(InstallerError::InvalidDevice {
                path: self.path.clone(),
                reason: "Device is read-only".to_string(),
            });
        }

        if self.removable {
            return Err(InstallerError::InvalidDevice {
                path: self.path.clone(),
                reason: "Device is removable (use --force to override)".to_string(),
            });
        }

        if self.size < 8 * 1024 * 1024 * 1024 {
            // 8GB minimum
            return Err(InstallerError::InvalidDevice {
                path: self.path.clone(),
                reason: format!("Device too small ({} bytes, need at least 8GB)", self.size),
            });
        }

        if self.is_mounted()? {
            return Err(InstallerError::DeviceInUse(self.path.clone()));
        }

        Ok(())
    }

    /// Get human-readable size
    pub fn size_human(&self) -> String {
        bytesize::ByteSize(self.size).to_string()
    }

    /// Get recommended ashift value based on physical block size
    pub fn recommended_ashift(&self) -> u8 {
        match self.physical_block_size {
            512 => 9,
            4096 => 12,
            8192 => 13,
            _ => 12, // Default to 4K
        }
    }

    /// Get a display name for the device
    pub fn display_name(&self) -> String {
        let model_info = self
            .model
            .as_ref()
            .map(|m| format!(" ({})", m.trim()))
            .unwrap_or_default();
        format!("{}{} - {}", self.name, model_info, self.size_human())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controller_type_detection() {
        assert_eq!(
            BlockDevice::detect_controller_type("sda"),
            ControllerType::Sata
        );
        assert_eq!(
            BlockDevice::detect_controller_type("nvme0n1"),
            ControllerType::Nvme
        );
        assert_eq!(
            BlockDevice::detect_controller_type("vda"),
            ControllerType::Virtual
        );
        assert_eq!(
            BlockDevice::detect_controller_type("mmcblk0"),
            ControllerType::Mmc
        );
    }

    #[test]
    fn test_recommended_ashift() {
        let mut device = BlockDevice {
            name: "sda".to_string(),
            path: PathBuf::from("/dev/sda"),
            sys_path: PathBuf::from("/sys/block/sda"),
            controller_type: ControllerType::Sata,
            size: 1_000_000_000_000,
            logical_block_size: 512,
            physical_block_size: 4096,
            model: None,
            serial: None,
            vendor: None,
            removable: false,
            readonly: false,
            rotational: false,
            partitions: Vec::new(),
        };

        assert_eq!(device.recommended_ashift(), 12);

        device.physical_block_size = 512;
        assert_eq!(device.recommended_ashift(), 9);
    }
}
