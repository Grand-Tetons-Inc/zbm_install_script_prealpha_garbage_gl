//! Package installation management

use crate::error::{InstallerError, Result};
use crate::system::distro::Distro;
use std::process::Command;

/// Package installer
pub struct PackageInstaller {
    distro: Distro,
    dry_run: bool,
}

impl PackageInstaller {
    /// Create a new package installer
    pub fn new(dry_run: bool) -> Result<Self> {
        let distro = Distro::detect()?;

        if !distro.is_supported() {
            return Err(InstallerError::Unsupported(format!(
                "Distribution {} is not supported",
                distro
            )));
        }

        Ok(Self { distro, dry_run })
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
            return Err(InstallerError::SystemError(format!(
                "Package command failed: {}\n{}",
                cmd_str, stderr
            )));
        }

        Ok(output)
    }

    /// Update package database
    pub fn update(&self) -> Result<()> {
        log::info!("Updating package database");

        match self.distro {
            Distro::Debian | Distro::Ubuntu | Distro::MxLinux => {
                self.execute(Command::new("apt-get").arg("update"))?;
            }
            Distro::Arch => {
                self.execute(Command::new("pacman").arg("-Sy"))?;
            }
            _ => {
                // Fedora and others don't need explicit update
            }
        }

        Ok(())
    }

    /// Install packages
    pub fn install(&self, packages: &[&str]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }

        log::info!("Installing packages: {}", packages.join(", "));

        let cmd_parts = self.distro.install_command(packages);
        let mut cmd = Command::new(&cmd_parts[0]);
        for arg in &cmd_parts[1..] {
            cmd.arg(arg);
        }

        self.execute(&mut cmd)?;
        Ok(())
    }

    /// Install ZFS packages
    pub fn install_zfs(&self) -> Result<()> {
        log::info!("Installing ZFS packages");

        let packages = self.distro.zfs_packages();
        self.install(&packages)?;

        Ok(())
    }

    /// Install ZFSBootMenu dependencies
    pub fn install_zbm_deps(&self) -> Result<()> {
        log::info!("Installing ZFSBootMenu dependencies");

        let packages = self.distro.zbm_packages();
        self.install(&packages)?;

        Ok(())
    }

    /// Check if a package is installed
    pub fn is_installed(&self, package: &str) -> bool {
        let result = match self.distro {
            Distro::Fedora => Command::new("rpm").arg("-q").arg(package).output(),
            Distro::Debian | Distro::Ubuntu | Distro::MxLinux => {
                Command::new("dpkg").arg("-s").arg(package).output()
            }
            Distro::Arch => Command::new("pacman").arg("-Q").arg(package).output(),
            Distro::Unknown => return false,
        };

        result.map(|o| o.status.success()).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_installer_creation() {
        let result = PackageInstaller::new(true);
        // May fail on unsupported distros, which is expected
        if result.is_ok() {
            let installer = result.unwrap();
            assert!(installer.dry_run);
        }
    }
}
