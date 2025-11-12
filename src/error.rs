//! Error types for the ZBM installer
//!
//! Provides comprehensive error handling using thiserror for ergonomic error definitions.

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for installer operations
pub type Result<T> = std::result::Result<T, InstallerError>;

/// Main error type for the installer
#[derive(Error, Debug)]
pub enum InstallerError {
    /// Device or file not found
    #[error("Device not found: {0}")]
    DeviceNotFound(PathBuf),

    /// Device is currently in use
    #[error("Device {0} is currently in use (mounted or part of active pool)")]
    DeviceInUse(PathBuf),

    /// Invalid device for operation
    #[error("Invalid device {path}: {reason}")]
    InvalidDevice { path: PathBuf, reason: String },

    /// ZFS command failed
    #[error("ZFS operation failed: {operation}\nDetails: {details}")]
    ZfsError { operation: String, details: String },

    /// Disk operation failed
    #[error("Disk operation failed: {operation}\nDetails: {details}")]
    DiskError { operation: String, details: String },

    /// Bootloader operation failed
    #[error("Bootloader operation failed: {0}")]
    BootloaderError(String),

    /// Validation error
    #[error("Validation failed: {0}")]
    ValidationError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// UI error
    #[error("UI error: {0}")]
    UiError(String),

    /// Insufficient permissions
    #[error("Insufficient permissions: {0}. This program must be run as root.")]
    PermissionDenied(String),

    /// Command execution failed
    #[error("Command '{cmd}' failed with exit code {code}: {stderr}")]
    CommandFailed {
        cmd: String,
        code: i32,
        stderr: String,
    },

    /// System error
    #[error("System error: {0}")]
    SystemError(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Unsupported operation
    #[error("Unsupported: {0}")]
    Unsupported(String),

    /// Generic error with context
    #[error("{0}")]
    Other(String),
}

impl InstallerError {
    /// Create a validation error
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::ValidationError(msg.into())
    }

    /// Create a config error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        Self::ConfigError(msg.into())
    }

    /// Create a ZFS error
    pub fn zfs<S: Into<String>>(operation: S, details: S) -> Self {
        Self::ZfsError {
            operation: operation.into(),
            details: details.into(),
        }
    }

    /// Create a disk error
    pub fn disk<S: Into<String>>(operation: S, details: S) -> Self {
        Self::DiskError {
            operation: operation.into(),
            details: details.into(),
        }
    }

    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::ValidationError(_) | Self::ConfigError(_) | Self::UiError(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = InstallerError::validation("test");
        assert!(matches!(err, InstallerError::ValidationError(_)));
        assert!(err.is_recoverable());
    }

    #[test]
    fn test_device_not_found() {
        let err = InstallerError::DeviceNotFound(PathBuf::from("/dev/sda"));
        assert!(err.to_string().contains("/dev/sda"));
    }

    #[test]
    fn test_command_failed() {
        let err = InstallerError::CommandFailed {
            cmd: "zpool create".to_string(),
            code: 1,
            stderr: "pool already exists".to_string(),
        };
        assert!(err.to_string().contains("zpool create"));
    }
}
