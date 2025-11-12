//! Disk management module
//!
//! Provides device discovery, representation, and operations inspired by Growlight.

pub mod block_device;
pub mod discovery;
pub mod operations;

pub use block_device::{BlockDevice, ControllerType, Partition};
pub use discovery::DeviceDiscovery;
pub use operations::{DiskOperations, PartitionSpec, ZbmPartitions};
