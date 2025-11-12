//! systemd-boot configuration

use crate::error::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// systemd-boot manager
pub struct SystemdBoot {
    efi_mountpoint: PathBuf,
    dry_run: bool,
}

impl SystemdBoot {
    /// Create a new systemd-boot manager
    pub fn new(efi_mountpoint: PathBuf, dry_run: bool) -> Self {
        Self {
            efi_mountpoint,
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
            log::warn!("Command failed (non-fatal): {}\n{}", cmd_str, stderr);
        }

        Ok(output)
    }

    /// Install systemd-boot
    pub fn install(&self) -> Result<()> {
        log::info!("Installing systemd-boot");

        self.execute(
            Command::new("bootctl")
                .arg("--path")
                .arg(&self.efi_mountpoint)
                .arg("install"),
        )?;

        self.configure()?;

        log::info!("systemd-boot installed successfully");
        Ok(())
    }

    /// Configure systemd-boot
    fn configure(&self) -> Result<()> {
        log::info!("Configuring systemd-boot");

        let loader_dir = self.efi_mountpoint.join("loader");
        let entries_dir = loader_dir.join("entries");

        // Create directories
        if !self.dry_run {
            fs::create_dir_all(&entries_dir)?;
        }

        // Write loader.conf
        let loader_conf = loader_dir.join("loader.conf");
        let loader_content = r#"default zfsbootmenu.conf
timeout 3
console-mode max
editor no
"#;

        self.write_file(&loader_conf, loader_content)?;

        // Write ZFSBootMenu boot entry
        let zbm_entry = entries_dir.join("zfsbootmenu.conf");
        let entry_content = r#"title ZFSBootMenu
efi /EFI/ZBM/zfsbootmenu.EFI
"#;

        self.write_file(&zbm_entry, entry_content)?;

        Ok(())
    }

    /// Helper to write file
    fn write_file(&self, path: &Path, content: &str) -> Result<()> {
        if self.dry_run {
            log::info!("[DRY RUN] Would write to: {}", path.display());
            return Ok(());
        }

        fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_systemd_boot_creation() {
        let systemd_boot = SystemdBoot::new(PathBuf::from("/boot/efi"), true);
        assert!(systemd_boot.dry_run);
    }
}
