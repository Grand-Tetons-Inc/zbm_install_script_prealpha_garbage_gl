//! ZFS dataset creation and management

use crate::error::{InstallerError, Result};
use std::process::Command;

/// Dataset property
#[derive(Debug, Clone)]
pub struct DatasetProperty {
    pub key: String,
    pub value: String,
}

/// ZFS dataset manager
pub struct DatasetManager {
    pool_name: String,
    dry_run: bool,
}

impl DatasetManager {
    /// Create a new dataset manager
    pub fn new(pool_name: String, dry_run: bool) -> Self {
        Self { pool_name, dry_run }
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

    /// Create a dataset
    pub fn create_dataset(&self, name: &str, properties: &[DatasetProperty]) -> Result<()> {
        log::info!("Creating dataset: {}/{}", self.pool_name, name);

        let mut cmd = Command::new("zfs");
        cmd.arg("create");

        // Add properties
        for prop in properties {
            cmd.arg("-o").arg(format!("{}={}", prop.key, prop.value));
        }

        // Dataset name
        cmd.arg(format!("{}/{}", self.pool_name, name));

        self.execute(&mut cmd)?;
        Ok(())
    }

    /// Create the standard ZBM dataset hierarchy
    pub fn create_zbm_datasets(&self) -> Result<()> {
        log::info!("Creating ZBM dataset hierarchy");

        // Define dataset structure
        // Format: (dataset_path, [(property, value)])
        let datasets = vec![
            // Boot environment container
            ("ROOT", vec![("canmount", "off"), ("mountpoint", "none")]),
            // Default boot environment
            (
                "ROOT/default",
                vec![("canmount", "noauto"), ("mountpoint", "/")],
            ),
            // Home directories
            ("home", vec![("mountpoint", "/home")]),
            // Root user home
            ("home/root", vec![("mountpoint", "/root")]),
            // Var container
            ("var", vec![("canmount", "off"), ("mountpoint", "none")]),
            // System logs
            (
                "var/log",
                vec![
                    ("mountpoint", "/var/log"),
                    ("acltype", "posixacl"),
                    ("xattr", "sa"),
                ],
            ),
            // Cache
            (
                "var/cache",
                vec![
                    ("mountpoint", "/var/cache"),
                    ("com.sun:auto-snapshot", "false"),
                ],
            ),
            // Temporary files
            (
                "var/tmp",
                vec![
                    ("mountpoint", "/var/tmp"),
                    ("com.sun:auto-snapshot", "false"),
                ],
            ),
            // Optional packages
            ("opt", vec![("mountpoint", "/opt")]),
            // Service data
            ("srv", vec![("mountpoint", "/srv")]),
            // Local software container
            ("usr", vec![("canmount", "off"), ("mountpoint", "none")]),
            // Locally installed software
            ("usr/local", vec![("mountpoint", "/usr/local")]),
        ];

        // Create each dataset
        for (name, props) in datasets {
            let properties: Vec<DatasetProperty> = props
                .iter()
                .map(|(k, v)| DatasetProperty {
                    key: k.to_string(),
                    value: v.to_string(),
                })
                .collect();

            self.create_dataset(name, &properties)?;
        }

        log::info!("ZBM dataset hierarchy created successfully");
        Ok(())
    }

    /// Create a snapshot
    pub fn snapshot(&self, dataset: &str, snapshot_name: &str) -> Result<()> {
        log::info!(
            "Creating snapshot: {}/{}@{}",
            self.pool_name,
            dataset,
            snapshot_name
        );

        self.execute(
            Command::new("zfs")
                .arg("snapshot")
                .arg(format!("{}/{}@{}", self.pool_name, dataset, snapshot_name)),
        )?;

        Ok(())
    }

    /// Mount a dataset
    pub fn mount(&self, dataset: &str) -> Result<()> {
        log::info!("Mounting dataset: {}/{}", self.pool_name, dataset);

        self.execute(
            Command::new("zfs")
                .arg("mount")
                .arg(format!("{}/{}", self.pool_name, dataset)),
        )?;

        Ok(())
    }

    /// Unmount a dataset
    pub fn unmount(&self, dataset: &str) -> Result<()> {
        log::info!("Unmounting dataset: {}/{}", self.pool_name, dataset);

        self.execute(
            Command::new("zfs")
                .arg("unmount")
                .arg(format!("{}/{}", self.pool_name, dataset)),
        )?;

        Ok(())
    }

    /// Set a property on a dataset
    pub fn set_property(&self, dataset: &str, property: &DatasetProperty) -> Result<()> {
        log::info!(
            "Setting property {}={} on {}/{}",
            property.key,
            property.value,
            self.pool_name,
            dataset
        );

        self.execute(
            Command::new("zfs")
                .arg("set")
                .arg(format!("{}={}", property.key, property.value))
                .arg(format!("{}/{}", self.pool_name, dataset)),
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dataset_manager_creation() {
        let manager = DatasetManager::new("testpool".to_string(), true);
        assert_eq!(manager.pool_name, "testpool");
        assert!(manager.dry_run);
    }

    #[test]
    fn test_dataset_property() {
        let prop = DatasetProperty {
            key: "mountpoint".to_string(),
            value: "/mnt".to_string(),
        };

        assert_eq!(prop.key, "mountpoint");
        assert_eq!(prop.value, "/mnt");
    }
}
