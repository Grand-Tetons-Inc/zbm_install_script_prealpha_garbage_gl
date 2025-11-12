//! Distribution detection and support

use crate::error::Result;
use std::fs;

/// Supported Linux distributions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Distro {
    Fedora,
    Debian,
    Ubuntu,
    MxLinux,
    Arch,
    Unknown,
}

impl std::fmt::Display for Distro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fedora => write!(f, "Fedora"),
            Self::Debian => write!(f, "Debian"),
            Self::Ubuntu => write!(f, "Ubuntu"),
            Self::MxLinux => write!(f, "MX Linux"),
            Self::Arch => write!(f, "Arch Linux"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

impl Distro {
    /// Detect the current distribution
    pub fn detect() -> Result<Self> {
        // Try /etc/os-release first (standard)
        if let Ok(content) = fs::read_to_string("/etc/os-release") {
            for line in content.lines() {
                if line.starts_with("ID=") {
                    let id = line.trim_start_matches("ID=").trim_matches('"');
                    return Ok(match id {
                        "fedora" => Self::Fedora,
                        "debian" => Self::Debian,
                        "ubuntu" => Self::Ubuntu,
                        "mx" => Self::MxLinux,
                        "arch" => Self::Arch,
                        _ => Self::Unknown,
                    });
                }
            }
        }

        // Fallback to checking specific files
        if fs::metadata("/etc/fedora-release").is_ok() {
            return Ok(Self::Fedora);
        }
        if fs::metadata("/etc/debian_version").is_ok() {
            return Ok(Self::Debian);
        }
        if fs::metadata("/etc/arch-release").is_ok() {
            return Ok(Self::Arch);
        }

        Ok(Self::Unknown)
    }

    /// Check if distribution is supported
    pub fn is_supported(&self) -> bool {
        !matches!(self, Self::Unknown)
    }

    /// Get package manager for this distribution
    pub fn package_manager(&self) -> &'static str {
        match self {
            Self::Fedora => "dnf",
            Self::Debian | Self::Ubuntu | Self::MxLinux => "apt-get",
            Self::Arch => "pacman",
            Self::Unknown => "unknown",
        }
    }

    /// Get ZFS package names for this distribution
    pub fn zfs_packages(&self) -> Vec<&'static str> {
        match self {
            Self::Fedora => vec!["zfs"],
            Self::Debian | Self::Ubuntu | Self::MxLinux => vec!["zfsutils-linux", "zfs-dkms"],
            Self::Arch => vec!["zfs-linux", "zfs-utils"],
            Self::Unknown => vec![],
        }
    }

    /// Get required packages for ZFSBootMenu
    pub fn zbm_packages(&self) -> Vec<&'static str> {
        match self {
            Self::Fedora => vec![
                "kernel-devel",
                "dracut",
                "efibootmgr",
                "gdisk",
                "util-linux",
            ],
            Self::Debian | Self::Ubuntu | Self::MxLinux => {
                vec!["dracut-core", "efibootmgr", "gdisk", "util-linux"]
            }
            Self::Arch => vec!["dracut", "efibootmgr", "gptfdisk", "util-linux"],
            Self::Unknown => vec![],
        }
    }

    /// Get install command for packages
    pub fn install_command(&self, packages: &[&str]) -> Vec<String> {
        let mut cmd = vec![self.package_manager().to_string()];

        match self {
            Self::Fedora => {
                cmd.push("install".to_string());
                cmd.push("-y".to_string());
            }
            Self::Debian | Self::Ubuntu | Self::MxLinux => {
                cmd.push("install".to_string());
                cmd.push("-y".to_string());
            }
            Self::Arch => {
                cmd.push("-S".to_string());
                cmd.push("--noconfirm".to_string());
            }
            Self::Unknown => {}
        }

        cmd.extend(packages.iter().map(|s| s.to_string()));
        cmd
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distro_detection() {
        let distro = Distro::detect();
        assert!(distro.is_ok());
    }

    #[test]
    fn test_distro_package_managers() {
        assert_eq!(Distro::Fedora.package_manager(), "dnf");
        assert_eq!(Distro::Debian.package_manager(), "apt-get");
        assert_eq!(Distro::Arch.package_manager(), "pacman");
    }

    #[test]
    fn test_distro_supported() {
        assert!(Distro::Fedora.is_supported());
        assert!(Distro::Debian.is_supported());
        assert!(!Distro::Unknown.is_supported());
    }

    #[test]
    fn test_zfs_packages() {
        let fedora_packages = Distro::Fedora.zfs_packages();
        assert!(fedora_packages.contains(&"zfs"));

        let debian_packages = Distro::Debian.zfs_packages();
        assert!(debian_packages.contains(&"zfsutils-linux"));
    }
}
