//! System utilities: distribution detection, package management, etc.

pub mod distro;
pub mod packages;

pub use distro::Distro;
pub use packages::PackageInstaller;

use crate::error::Result;
use std::process::Command;

/// Check if running as root
pub fn is_root() -> bool {
    nix::unistd::Uid::effective().is_root()
}

/// Check if system is booted with UEFI
pub fn is_uefi() -> bool {
    std::path::Path::new("/sys/firmware/efi").exists()
}

/// Get system memory in KB
pub fn get_system_memory_kb() -> Result<u64> {
    let meminfo = std::fs::read_to_string("/proc/meminfo")?;
    for line in meminfo.lines() {
        if line.starts_with("MemTotal:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                return parts[1]
                    .parse()
                    .map_err(|e| crate::error::InstallerError::ParseError(format!("{}", e)));
            }
        }
    }
    Ok(0)
}

/// Sync filesystems
pub fn sync() -> Result<()> {
    Command::new("sync").status()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_uefi() {
        // This will vary by system, just test it doesn't panic
        let _ = is_uefi();
    }

    #[test]
    fn test_get_system_memory() {
        let result = get_system_memory_kb();
        assert!(result.is_ok());
        if let Ok(mem) = result {
            assert!(mem > 0);
        }
    }
}
