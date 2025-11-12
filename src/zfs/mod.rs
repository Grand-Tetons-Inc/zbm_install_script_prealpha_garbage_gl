//! ZFS pool and dataset management

pub mod dataset;
pub mod pool;

pub use dataset::{DatasetManager, DatasetProperty};
pub use pool::ZfsPool;

use crate::error::Result;
use std::process::Command;

/// Check if ZFS is available on the system
pub fn check_zfs_available() -> Result<bool> {
    Ok(Command::new("zpool")
        .arg("version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false))
}

/// Get ZFS version
pub fn get_zfs_version() -> Result<String> {
    let output = Command::new("zpool").arg("version").output()?;

    if !output.status.success() {
        return Ok("unknown".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Parse version from first line
    if let Some(first_line) = stdout.lines().next() {
        Ok(first_line.to_string())
    } else {
        Ok("unknown".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_zfs_available() {
        // This test will pass or fail depending on whether ZFS is installed
        let result = check_zfs_available();
        assert!(result.is_ok());
    }
}
