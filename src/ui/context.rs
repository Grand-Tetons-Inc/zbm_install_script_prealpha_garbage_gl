//! Notcurses context wrapper - safe Rust interface to libnotcurses

#[cfg(feature = "tui")]
use libnotcurses_sys::*;

use crate::error::{InstallerError, Result};
use std::ffi::CString;
use std::ptr;

/// Notcurses context wrapper
#[cfg(feature = "tui")]
pub struct NotcursesContext {
    nc: *mut Notcurses,
    stdplane: *mut NcPlane,
    rows: u32,
    cols: u32,
}

#[cfg(feature = "tui")]
impl NotcursesContext {
    /// Initialize notcurses
    pub fn init() -> Result<Self> {
        unsafe {
            // Create options with sensible defaults
            let opts = NcOptions {
                termtype: ptr::null(),
                loglevel: NCLOGLEVEL_WARNING,
                margin_t: 0,
                margin_r: 0,
                margin_b: 0,
                margin_l: 0,
                flags: NCOPTION_SUPPRESS_BANNERS
                    | NCOPTION_NO_ALTERNATE_SCREEN
                    | NCOPTION_PRESERVE_CURSOR,
            };

            let nc = notcurses_init(&opts, ptr::null_mut());
            if nc.is_null() {
                return Err(InstallerError::UiError(
                    "Failed to initialize notcurses".into(),
                ));
            }

            let stdplane = notcurses_stdplane(nc);
            if stdplane.is_null() {
                notcurses_stop(nc);
                return Err(InstallerError::UiError("Failed to get stdplane".into()));
            }

            let mut rows = 0u32;
            let mut cols = 0u32;
            ncplane_dim_yx(stdplane, &mut rows, &mut cols);

            Ok(Self {
                nc,
                stdplane,
                rows,
                cols,
            })
        }
    }

    /// Get terminal dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.rows, self.cols)
    }

    /// Get standard plane
    pub fn stdplane(&mut self) -> *mut NcPlane {
        self.stdplane
    }

    /// Clear the screen
    pub fn clear(&mut self) -> Result<()> {
        unsafe {
            ncplane_erase(self.stdplane);
        }
        Ok(())
    }

    /// Render the screen
    pub fn render(&mut self) -> Result<()> {
        unsafe {
            if notcurses_render(self.nc) != 0 {
                return Err(InstallerError::UiError("Failed to render".into()));
            }
        }
        Ok(())
    }

    /// Get a character/key input (blocking)
    pub fn get_blocking(&mut self) -> Result<NcInput> {
        unsafe {
            let mut input = NcInput::default();
            let id = notcurses_get_blocking(self.nc, &mut input);
            if id == u32::MAX {
                return Err(InstallerError::UiError("Failed to get input".into()));
            }
            Ok(input)
        }
    }

    /// Get a character/key input (non-blocking)
    pub fn get_nonblocking(&mut self) -> Result<Option<NcInput>> {
        unsafe {
            let mut input = NcInput::default();
            let id = notcurses_get_nblock(self.nc, &mut input);
            if id == 0 {
                Ok(None)
            } else if id == u32::MAX {
                Err(InstallerError::UiError("Failed to get input".into()))
            } else {
                Ok(Some(input))
            }
        }
    }

    /// Put text at a specific position with optional styling
    pub fn putstr_yx(
        &mut self,
        y: u32,
        x: u32,
        text: &str,
        channels: u64,
    ) -> Result<()> {
        unsafe {
            ncplane_set_base(
                self.stdplane,
                " ",
                0,
                channels,
            );
            ncplane_cursor_move_yx(self.stdplane, y as i32, x as i32);

            let c_text = CString::new(text)
                .map_err(|e| InstallerError::UiError(format!("Invalid string: {}", e)))?;

            ncplane_set_channels(self.stdplane, channels);
            ncplane_putstr_yx(self.stdplane, y as i32, x as i32, c_text.as_ptr());
        }
        Ok(())
    }

    /// Draw a box with optional title
    pub fn draw_box(
        &mut self,
        y: u32,
        x: u32,
        height: u32,
        width: u32,
        title: Option<&str>,
        channels: u64,
    ) -> Result<()> {
        unsafe {
            // Draw corners and edges
            let ul = "┌";
            let ur = "┐";
            let ll = "└";
            let lr = "┘";
            let hl = "─";
            let vl = "│";

            ncplane_set_channels(self.stdplane, channels);

            // Top border
            self.putstr_yx(y, x, ul, channels)?;
            for i in 1..width - 1 {
                self.putstr_yx(y, x + i, hl, channels)?;
            }
            self.putstr_yx(y, x + width - 1, ur, channels)?;

            // Title (if provided)
            if let Some(title) = title {
                let title_x = x + (width - title.len() as u32) / 2;
                self.putstr_yx(y, title_x - 1, " ", channels)?;
                self.putstr_yx(y, title_x, title, channels)?;
                self.putstr_yx(y, title_x + title.len() as u32, " ", channels)?;
            }

            // Sides
            for i in 1..height - 1 {
                self.putstr_yx(y + i, x, vl, channels)?;
                self.putstr_yx(y + i, x + width - 1, vl, channels)?;
            }

            // Bottom border
            self.putstr_yx(y + height - 1, x, ll, channels)?;
            for i in 1..width - 1 {
                self.putstr_yx(y + height - 1, x + i, hl, channels)?;
            }
            self.putstr_yx(y + height - 1, x + width - 1, lr, channels)?;
        }
        Ok(())
    }

    /// Draw a progress bar
    pub fn draw_progress_bar(
        &mut self,
        y: u32,
        x: u32,
        width: u32,
        progress: f32,
        label: Option<&str>,
        fg_channels: u64,
        bg_channels: u64,
    ) -> Result<()> {
        let filled = (width as f32 * progress.clamp(0.0, 1.0)) as u32;

        unsafe {
            // Draw filled portion
            ncplane_set_channels(self.stdplane, fg_channels);
            for i in 0..filled {
                self.putstr_yx(y, x + i, "█", fg_channels)?;
            }

            // Draw empty portion
            ncplane_set_channels(self.stdplane, bg_channels);
            for i in filled..width {
                self.putstr_yx(y, x + i, "░", bg_channels)?;
            }
        }

        // Draw label if provided
        if let Some(label) = label {
            let label_x = x + (width - label.len() as u32) / 2;
            self.putstr_yx(y, label_x, label, fg_channels)?;
        }

        Ok(())
    }
}

#[cfg(feature = "tui")]
impl Drop for NotcursesContext {
    fn drop(&mut self) {
        unsafe {
            if !self.nc.is_null() {
                notcurses_stop(self.nc);
            }
        }
    }
}

// Mock implementation when tui feature is disabled
#[cfg(not(feature = "tui"))]
pub struct NotcursesContext;

#[cfg(not(feature = "tui"))]
impl NotcursesContext {
    pub fn init() -> Result<Self> {
        Err(InstallerError::UiError(
            "TUI support not compiled in. Rebuild with --features tui".into(),
        ))
    }
}

/// Helper functions for creating channel values (color pairs)
#[cfg(feature = "tui")]
pub mod channels {
    use libnotcurses_sys::*;

    /// Create a channel with RGB colors
    pub fn from_rgb(fg_r: u8, fg_g: u8, fg_b: u8, bg_r: u8, bg_g: u8, bg_b: u8) -> u64 {
        let fg = ((fg_r as u32) << 16) | ((fg_g as u32) << 8) | (fg_b as u32);
        let bg = ((bg_r as u32) << 16) | ((bg_g as u32) << 8) | (bg_b as u32);
        ((fg as u64) << 32) | (bg as u64)
    }

    /// Create a channel with only foreground RGB
    pub fn fg_rgb(r: u8, g: u8, b: u8) -> u64 {
        from_rgb(r, g, b, 0, 0, 0)
    }

    /// Common color presets
    pub const WHITE_ON_BLACK: u64 = 0xFFFFFF_000000;
    pub const BLACK_ON_WHITE: u64 = 0x000000_FFFFFF;
    pub const GREEN_ON_BLACK: u64 = 0x00FF00_000000;
    pub const RED_ON_BLACK: u64 = 0xFF0000_000000;
    pub const YELLOW_ON_BLACK: u64 = 0xFFFF00_000000;
    pub const BLUE_ON_BLACK: u64 = 0x0088FF_000000;
    pub const CYAN_ON_BLACK: u64 = 0x00FFFF_000000;
    pub const MAGENTA_ON_BLACK: u64 = 0xFF00FF_000000;
}

#[cfg(not(feature = "tui"))]
pub mod channels {
    pub const WHITE_ON_BLACK: u64 = 0;
    pub const BLACK_ON_WHITE: u64 = 0;
    pub const GREEN_ON_BLACK: u64 = 0;
    pub const RED_ON_BLACK: u64 = 0;
    pub const YELLOW_ON_BLACK: u64 = 0;
    pub const BLUE_ON_BLACK: u64 = 0;
    pub const CYAN_ON_BLACK: u64 = 0;
    pub const MAGENTA_ON_BLACK: u64 = 0;
}
