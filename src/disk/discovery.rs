//! Device discovery using /sys/class/block and inotify
//!
//! Inspired by Growlight's approach to device discovery and hotplug detection.

use crate::disk::block_device::BlockDevice;
use crate::error::{InstallerError, Result};
use inotify::{Inotify, WatchMask};
use std::fs;
use std::path::Path;

/// Device discovery manager
pub struct DeviceDiscovery {
    /// Inotify instance for monitoring device changes
    inotify: Option<Inotify>,
}

impl DeviceDiscovery {
    /// Create a new device discovery manager
    pub fn new() -> Result<Self> {
        Ok(Self { inotify: None })
    }

    /// Initialize inotify watches for device hotplug detection
    pub fn enable_hotplug_detection(&mut self) -> Result<()> {
        let inotify = Inotify::init()?;

        // Watch /sys/class/block for device additions/removals
        inotify
            .watches()
            .add("/sys/class/block", WatchMask::CREATE | WatchMask::DELETE)
            .map_err(|e| {
                InstallerError::SystemError(format!("Failed to watch /sys/class/block: {}", e))
            })?;

        self.inotify = Some(inotify);
        Ok(())
    }

    /// Scan for all block devices
    pub fn scan_devices(&self) -> Result<Vec<BlockDevice>> {
        let mut devices = Vec::new();
        let block_path = Path::new("/sys/class/block");

        if !block_path.exists() {
            return Err(InstallerError::SystemError(
                "/sys/class/block not found - are you on Linux?".to_string(),
            ));
        }

        // Iterate through all entries in /sys/class/block
        for entry in fs::read_dir(block_path)? {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            // Skip partitions (they contain numbers or 'p' followed by numbers)
            if Self::is_partition(&name_str) {
                continue;
            }

            // Skip loop devices by default
            if name_str.starts_with("loop") {
                continue;
            }

            // Try to create BlockDevice - skip if it fails
            match BlockDevice::from_name(&name_str) {
                Ok(device) => {
                    // Additional filtering
                    if Self::should_include(&device) {
                        devices.push(device);
                    }
                }
                Err(e) => {
                    log::debug!("Failed to create device {}: {}", name_str, e);
                }
            }
        }

        // Sort devices by name
        devices.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(devices)
    }

    /// Check if a device name represents a partition
    fn is_partition(name: &str) -> bool {
        // For nvme: nvme0n1p1, nvme0n1p2, etc.
        if name.contains("nvme") && name.contains('p') {
            let after_p = name.split('p').last().unwrap_or("");
            return after_p.chars().all(|c| c.is_ascii_digit());
        }

        // For other devices: sda1, sdb2, etc.
        let last_char = name.chars().last();
        matches!(last_char, Some(c) if c.is_ascii_digit())
    }

    /// Determine if a device should be included in results
    fn should_include(device: &BlockDevice) -> bool {
        // Exclude devices smaller than 1GB
        if device.size < 1024 * 1024 * 1024 {
            return false;
        }

        // Exclude CD/DVD drives (sr0, sr1, etc.)
        if device.name.starts_with("sr") {
            return false;
        }

        true
    }

    /// Find a specific device by name
    pub fn find_device(&self, name: &str) -> Result<BlockDevice> {
        BlockDevice::from_name(name)
    }

    /// Find devices by path
    pub fn find_devices_by_path(&self, paths: &[std::path::PathBuf]) -> Result<Vec<BlockDevice>> {
        let mut devices = Vec::new();

        for path in paths {
            // Extract device name from path (e.g., /dev/sda -> sda)
            let name = path
                .file_name()
                .ok_or_else(|| InstallerError::DeviceNotFound(path.clone()))?
                .to_string_lossy()
                .to_string();

            devices.push(self.find_device(&name)?);
        }

        Ok(devices)
    }
}

impl Default for DeviceDiscovery {
    fn default() -> Self {
        Self::new().expect("Failed to create DeviceDiscovery")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_partition() {
        assert!(!DeviceDiscovery::is_partition("sda"));
        assert!(DeviceDiscovery::is_partition("sda1"));
        assert!(DeviceDiscovery::is_partition("sda12"));
        assert!(!DeviceDiscovery::is_partition("nvme0n1"));
        assert!(DeviceDiscovery::is_partition("nvme0n1p1"));
        assert!(DeviceDiscovery::is_partition("nvme0n1p12"));
        assert!(!DeviceDiscovery::is_partition("vda"));
        assert!(DeviceDiscovery::is_partition("vda1"));
    }

    #[test]
    fn test_should_include() {
        let device = BlockDevice {
            name: "sda".to_string(),
            path: std::path::PathBuf::from("/dev/sda"),
            sys_path: std::path::PathBuf::from("/sys/block/sda"),
            controller_type: crate::disk::block_device::ControllerType::Sata,
            size: 10_000_000_000, // 10GB
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

        assert!(DeviceDiscovery::should_include(&device));

        let mut small_device = device.clone();
        small_device.size = 512 * 1024 * 1024; // 512MB
        assert!(!DeviceDiscovery::should_include(&small_device));
    }

    #[test]
    fn test_device_discovery_creation() {
        let discovery = DeviceDiscovery::new();
        assert!(discovery.is_ok());
    }
}
