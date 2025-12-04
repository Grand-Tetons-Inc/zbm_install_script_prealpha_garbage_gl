//! TUI framework using Notcurses
//!
//! This module provides a text user interface for the ZBM installer.

pub mod context;
pub mod runner;
pub mod screens;
pub mod widgets;

pub use context::NotcursesContext;
pub use runner::UiRunner;
pub use screens::Screen;

use crate::config::Config;
use crate::error::Result;

/// UI manager
pub struct UiManager {
    config: Config,
}

impl UiManager {
    /// Create a new UI manager
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Run the interactive TUI
    pub fn run(&mut self) -> Result<Config> {
        let mut runner = UiRunner::new(self.config.clone());
        runner.run()
    }
}

// NOTE: Full Notcurses implementation would include:
// - Initialization of notcurses context
// - Screen management and transitions
// - Widget rendering (device lists, progress bars, dialogs)
// - Keyboard/mouse event handling
// - Real-time device discovery updates
// - Progress display during installation
//
// Example structure for future implementation:
//
// ```rust
// use libnotcurses_sys::*;
//
// pub struct NotcursesContext {
//     nc: *mut Notcurses,
//     stdplane: *mut NcPlane,
// }
//
// impl NotcursesContext {
//     pub fn init() -> Result<Self> {
//         unsafe {
//             let mut opts = notcurses_options::default();
//             let nc = notcurses_init(&opts, std::ptr::null_mut());
//             if nc.is_null() {
//                 return Err(InstallerError::UiError("Failed to init notcurses".into()));
//             }
//             let stdplane = notcurses_stdplane(nc);
//             Ok(Self { nc, stdplane })
//         }
//     }
//
//     pub fn render(&mut self) -> Result<()> {
//         unsafe {
//             notcurses_render(self.nc);
//         }
//         Ok(())
//     }
// }
//
// impl Drop for NotcursesContext {
//     fn drop(&mut self) {
//         unsafe {
//             notcurses_stop(self.nc);
//         }
//     }
// }
// ```
