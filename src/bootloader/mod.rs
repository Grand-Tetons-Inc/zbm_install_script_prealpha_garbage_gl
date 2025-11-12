//! Bootloader installation and configuration

pub mod systemd_boot;
pub mod zbm;

pub use systemd_boot::SystemdBoot;
pub use zbm::ZbmInstaller;
