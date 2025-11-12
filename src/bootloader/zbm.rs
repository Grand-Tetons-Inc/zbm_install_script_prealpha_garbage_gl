//! ZFSBootMenu installation and configuration

use crate::error::{InstallerError, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// ZFSBootMenu installer
pub struct ZbmInstaller {
    #[allow(dead_code)] // May be used in future for pool-specific config
    pool_name: String,
    efi_mountpoint: PathBuf,
    dry_run: bool,
}

impl ZbmInstaller {
    /// Create a new ZBM installer
    pub fn new(pool_name: String, efi_mountpoint: PathBuf, dry_run: bool) -> Self {
        Self {
            pool_name,
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
            return Err(InstallerError::BootloaderError(format!(
                "Command failed: {}\n{}",
                cmd_str, stderr
            )));
        }

        Ok(output)
    }

    /// Download ZFSBootMenu release
    pub fn download_zbm(&self, version: &str) -> Result<PathBuf> {
        log::info!("Downloading ZFSBootMenu version {}", version);

        let url = format!(
            "https://get.zfsbootmenu.org/efi/zfsbootmenu-release-x86_64-v{}.EFI",
            version
        );

        let temp_path = PathBuf::from(format!("/tmp/zfsbootmenu-v{}.EFI", version));

        if self.dry_run {
            log::info!(
                "[DRY RUN] Would download {} to {}",
                url,
                temp_path.display()
            );
            return Ok(temp_path);
        }

        // Use curl to download
        self.execute(
            Command::new("curl")
                .arg("-L")
                .arg("-o")
                .arg(&temp_path)
                .arg(&url),
        )?;

        Ok(temp_path)
    }

    /// Install ZFSBootMenu to EFI partition
    pub fn install(&self) -> Result<()> {
        log::info!("Installing ZFSBootMenu");

        // Create EFI directory structure
        let zbm_dir = self.efi_mountpoint.join("EFI").join("ZBM");
        self.create_directory(&zbm_dir)?;

        // Download latest ZBM
        let zbm_efi = self.download_zbm("2.3.0")?; // Use stable version

        // Copy to EFI partition
        let dest = zbm_dir.join("zfsbootmenu.EFI");
        self.copy_file(&zbm_efi, &dest)?;

        // Generate ZBM configuration
        self.generate_config()?;

        // Generate initramfs with ZFS support
        self.generate_initramfs()?;

        log::info!("ZFSBootMenu installed successfully");
        Ok(())
    }

    /// Generate ZFSBootMenu configuration
    fn generate_config(&self) -> Result<()> {
        log::info!("Generating ZFSBootMenu configuration");

        let config_dir = Path::new("/etc/zfsbootmenu");
        self.create_directory(config_dir)?;

        let config_content = format!(
            r#"# ZFSBootMenu configuration
Global:
  ManageImages: true
  BootMountPoint: {}
  DracutConfDir: /etc/zfsbootmenu/dracut.conf.d
  PreHooksDir: /etc/zfsbootmenu/hooks.d
  InitCPIOHookDirs: /etc/zfsbootmenu/initcpio.d

Components:
  Enabled: true
  ImageDir: {}/EFI/ZBM
  Versions: 3
  Cmdline: ro quiet loglevel=0

EFI:
  ImageDir: {}/EFI/ZBM
  Versions: false
  Enabled: true

Kernel:
  CommandLine: ro quiet loglevel=4
"#,
            self.efi_mountpoint.display(),
            self.efi_mountpoint.display(),
            self.efi_mountpoint.display()
        );

        let config_file = config_dir.join("config.yaml");
        self.write_file(&config_file, &config_content)?;

        Ok(())
    }

    /// Generate initramfs with ZFS support
    fn generate_initramfs(&self) -> Result<()> {
        log::info!("Generating initramfs with ZFS support");

        // Check if we're using dracut or mkinitcpio
        if Path::new("/usr/bin/dracut").exists() {
            self.generate_dracut()?;
        } else if Path::new("/usr/bin/mkinitcpio").exists() {
            self.generate_mkinitcpio()?;
        } else {
            return Err(InstallerError::BootloaderError(
                "No supported initramfs generator found (dracut or mkinitcpio)".to_string(),
            ));
        }

        Ok(())
    }

    /// Generate initramfs using dracut
    fn generate_dracut(&self) -> Result<()> {
        log::info!("Generating initramfs with dracut");

        // Create dracut config for ZFS
        let dracut_conf_dir = Path::new("/etc/dracut.conf.d");
        self.create_directory(dracut_conf_dir)?;

        let dracut_conf = dracut_conf_dir.join("zfsbootmenu.conf");
        let conf_content = r#"# ZFSBootMenu dracut configuration
add_dracutmodules+=" zfsbootmenu "
omit_dracutmodules+=" network "
hostonly=no
compress="zstd"
"#;

        self.write_file(&dracut_conf, conf_content)?;

        // Run generate-zbm if available
        if Path::new("/usr/bin/generate-zbm").exists() {
            self.execute(&mut Command::new("generate-zbm"))?;
        } else {
            // Fallback: run dracut manually
            self.execute(
                Command::new("dracut")
                    .arg("--force")
                    .arg("--add")
                    .arg("zfsbootmenu")
                    .arg(self.efi_mountpoint.join("EFI/ZBM/vmlinuz.efi")),
            )?;
        }

        Ok(())
    }

    /// Generate initramfs using mkinitcpio
    fn generate_mkinitcpio(&self) -> Result<()> {
        log::info!("Generating initramfs with mkinitcpio");

        // Create mkinitcpio config
        let conf_content = r#"# ZFSBootMenu mkinitcpio configuration
HOOKS=(base udev autodetect modconf block filesystems keyboard fsck zfsbootmenu)
"#;

        self.write_file(
            Path::new("/etc/mkinitcpio.conf.d/zfsbootmenu.conf"),
            conf_content,
        )?;

        // Run mkinitcpio
        self.execute(Command::new("mkinitcpio").arg("-P"))?;

        Ok(())
    }

    /// Helper to create directory
    fn create_directory(&self, path: &Path) -> Result<()> {
        if self.dry_run {
            log::info!("[DRY RUN] Would create directory: {}", path.display());
            return Ok(());
        }

        fs::create_dir_all(path)?;
        Ok(())
    }

    /// Helper to copy file
    fn copy_file(&self, src: &Path, dest: &Path) -> Result<()> {
        if self.dry_run {
            log::info!(
                "[DRY RUN] Would copy {} to {}",
                src.display(),
                dest.display()
            );
            return Ok(());
        }

        fs::copy(src, dest)?;
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
    fn test_zbm_installer_creation() {
        let installer = ZbmInstaller::new("zroot".to_string(), PathBuf::from("/boot/efi"), true);

        assert_eq!(installer.pool_name, "zroot");
        assert!(installer.dry_run);
    }
}
