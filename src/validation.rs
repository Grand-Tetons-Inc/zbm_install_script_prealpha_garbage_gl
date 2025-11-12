//! Pre-flight validation checks

use crate::config::Config;
use crate::disk::DeviceDiscovery;
use crate::error::{InstallerError, Result};
use crate::system::{is_root, is_uefi};
use crate::zfs;

/// Validation result
#[derive(Debug)]
pub struct ValidationResult {
    pub passed: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Create a new validation result
    pub fn new() -> Self {
        Self {
            passed: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Add an error
    pub fn add_error(&mut self, msg: String) {
        self.passed = false;
        self.errors.push(msg);
    }

    /// Add a warning
    pub fn add_warning(&mut self, msg: String) {
        self.warnings.push(msg);
    }

    /// Check if validation passed
    pub fn is_ok(&self) -> bool {
        self.passed
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// System validator
pub struct Validator {
    config: Config,
}

impl Validator {
    /// Create a new validator
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Run all validation checks
    pub fn validate(&self) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        // Check root privileges
        if !is_root() {
            result.add_error("This program must be run as root".to_string());
        }

        // Check UEFI
        if !is_uefi() {
            result.add_error("System must be booted in UEFI mode".to_string());
        }

        // Validate config
        if let Err(e) = self.config.validate() {
            result.add_error(format!("Configuration error: {}", e));
        }

        // Check ZFS availability
        match zfs::check_zfs_available() {
            Ok(true) => {}
            Ok(false) => {
                result.add_error(
                    "ZFS is not available on this system. Please install ZFS first.".to_string(),
                );
            }
            Err(e) => {
                result.add_error(format!("Failed to check ZFS availability: {}", e));
            }
        }

        // Validate devices
        if let Err(e) = self.validate_devices(&mut result) {
            result.add_error(format!("Device validation failed: {}", e));
        }

        // Check system requirements
        self.check_system_requirements(&mut result)?;

        Ok(result)
    }

    /// Validate selected devices
    fn validate_devices(&self, result: &mut ValidationResult) -> Result<()> {
        let discovery = DeviceDiscovery::new()?;

        for device_path in &self.config.devices {
            let device_name = device_path
                .file_name()
                .ok_or_else(|| InstallerError::DeviceNotFound(device_path.clone()))?
                .to_string_lossy()
                .to_string();

            match discovery.find_device(&device_name) {
                Ok(device) => {
                    // Check if device is suitable
                    if let Err(e) = device.is_suitable() {
                        if device.removable && self.config.force {
                            result.add_warning(format!(
                                "Device {} is removable but --force was specified",
                                device.path.display()
                            ));
                        } else {
                            result.add_error(format!("{}", e));
                        }
                    }

                    // Check minimum size
                    let min_size = self.config.min_device_size();
                    if device.size < min_size.0 {
                        result.add_error(format!(
                            "Device {} is too small ({}, need at least {})",
                            device.path.display(),
                            device.size_human(),
                            min_size
                        ));
                    }
                }
                Err(e) => {
                    result.add_error(format!("Device {}: {}", device_path.display(), e));
                }
            }
        }

        Ok(())
    }

    /// Check system requirements
    fn check_system_requirements(&self, result: &mut ValidationResult) -> Result<()> {
        // Check memory (minimum 2GB for ZFS)
        let mem_kb = crate::system::get_system_memory_kb()?;
        let mem_gb = mem_kb / (1024 * 1024);

        if mem_gb < 2 {
            result.add_warning(format!(
                "System has only {}GB of RAM. ZFS recommends at least 2GB.",
                mem_gb
            ));
        }

        // Check required commands
        let required_commands = vec!["sgdisk", "mkfs.vfat", "zpool", "zfs"];
        for cmd in required_commands {
            if !self.command_exists(cmd) {
                result.add_error(format!("Required command not found: {}", cmd));
            }
        }

        Ok(())
    }

    /// Check if a command exists
    fn command_exists(&self, cmd: &str) -> bool {
        std::process::Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result() {
        let mut result = ValidationResult::new();
        assert!(result.is_ok());

        result.add_warning("Test warning".to_string());
        assert!(result.is_ok());

        result.add_error("Test error".to_string());
        assert!(!result.is_ok());
    }

    #[test]
    fn test_validator_creation() {
        let config = Config::default();
        let validator = Validator::new(config);
        // Just test that it creates successfully
        assert_eq!(validator.config.pool_name, "zroot");
    }
}
