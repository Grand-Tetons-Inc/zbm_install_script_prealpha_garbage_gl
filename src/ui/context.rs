//! Notcurses context wrapper - safe Rust interface to libnotcurses

#[cfg(feature = "tui")]
use libnotcurses_sys::{Nc, NcInput, NcPlane, NcReceived};

use crate::error::{InstallerError, Result};

/// Notcurses context wrapper
#[cfg(feature = "tui")]
pub struct NotcursesContext {
    nc: &'static mut Nc,
    rows: u32,
    cols: u32,
}

#[cfg(feature = "tui")]
impl NotcursesContext {
    /// Initialize notcurses
    pub fn init() -> Result<Self> {
        let nc = unsafe { Nc::new() }.map_err(|e| {
            InstallerError::UiError(format!("Failed to initialize notcurses: {:?}", e))
        })?;

        let stdplane = unsafe { nc.stdplane() };
        let (rows, cols) = stdplane.dim_yx();

        Ok(Self {
            nc,
            rows,
            cols,
        })
    }

    /// Get terminal dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.rows, self.cols)
    }

    /// Get standard plane
    pub fn stdplane(&mut self) -> &mut NcPlane {
        unsafe { self.nc.stdplane() }
    }

    /// Clear the screen
    pub fn clear(&mut self) -> Result<()> {
        let plane = unsafe { self.nc.stdplane() };
        plane.erase();
        Ok(())
    }

    /// Render the screen
    pub fn render(&mut self) -> Result<()> {
        self.nc.render().map_err(|e| {
            InstallerError::UiError(format!("Failed to render: {:?}", e))
        })
    }

    /// Get a character/key input (blocking)
    pub fn get_blocking(&mut self) -> Result<NcInput> {
        let mut input = NcInput::default();
        self.nc.get_blocking(Some(&mut input)).map_err(|e| {
            InstallerError::UiError(format!("Failed to get input: {:?}", e))
        })?;
        Ok(input)
    }

    /// Get a character/key input (non-blocking)
    pub fn get_nonblocking(&mut self) -> Result<Option<NcInput>> {
        let mut input = NcInput::default();
        let result = self.nc.get_nblock(Some(&mut input)).map_err(|e| {
            InstallerError::UiError(format!("Failed to get input: {:?}", e))
        })?;

        match result {
            NcReceived::NoInput => Ok(None),
            _ => Ok(Some(input)),
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
        let plane = unsafe { self.nc.stdplane() };
        plane.set_channels(channels);
        let _ = plane.cursor_move_yx(y, x);
        let _ = plane.putstr_yx(Some(y), Some(x), text);
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
        // Draw corners and edges
        let ul = "┌";
        let ur = "┐";
        let ll = "└";
        let lr = "┘";
        let hl = "─";
        let vl = "│";

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

        // Draw filled portion
        for i in 0..filled {
            self.putstr_yx(y, x + i, "█", fg_channels)?;
        }

        // Draw empty portion
        for i in filled..width {
            self.putstr_yx(y, x + i, "░", bg_channels)?;
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
        let _ = unsafe { self.nc.stop() };
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
    /// Create a channel with RGB colors
    pub fn from_rgb(fg_r: u8, fg_g: u8, fg_b: u8, bg_r: u8, bg_g: u8, bg_b: u8) -> u64 {
        let fg = ((fg_r as u32) << 16) | ((fg_g as u32) << 8) | (fg_b as u32);
        let bg = ((bg_r as u32) << 16) | ((bg_g as u32) << 8) | (bg_b as u32);
        // Set the "not default color" bit for both fg and bg
        let fg_channel = (fg as u64) | 0x4000_0000; // NC_BGDEFAULT_MASK inverted for fg
        let bg_channel = (bg as u64) | 0x4000_0000; // NC_BGDEFAULT_MASK for bg
        (fg_channel << 32) | bg_channel
    }

    /// Create a channel with only foreground RGB
    pub fn fg_rgb(r: u8, g: u8, b: u8) -> u64 {
        from_rgb(r, g, b, 0, 0, 0)
    }

    /// Common color presets
    pub const WHITE_ON_BLACK: u64 = 0x40FFFFFF_40000000;
    pub const BLACK_ON_WHITE: u64 = 0x40000000_40FFFFFF;
    pub const GREEN_ON_BLACK: u64 = 0x4000FF00_40000000;
    pub const RED_ON_BLACK: u64 = 0x40FF0000_40000000;
    pub const YELLOW_ON_BLACK: u64 = 0x40FFFF00_40000000;
    pub const BLUE_ON_BLACK: u64 = 0x400088FF_40000000;
    pub const CYAN_ON_BLACK: u64 = 0x4000FFFF_40000000;
    pub const MAGENTA_ON_BLACK: u64 = 0x40FF00FF_40000000;
}

#[cfg(not(feature = "tui"))]
pub mod channels {
    pub fn from_rgb(_fg_r: u8, _fg_g: u8, _fg_b: u8, _bg_r: u8, _bg_g: u8, _bg_b: u8) -> u64 {
        0
    }

    pub const WHITE_ON_BLACK: u64 = 0;
    pub const BLACK_ON_WHITE: u64 = 0;
    pub const GREEN_ON_BLACK: u64 = 0;
    pub const RED_ON_BLACK: u64 = 0;
    pub const YELLOW_ON_BLACK: u64 = 0;
    pub const BLUE_ON_BLACK: u64 = 0;
    pub const CYAN_ON_BLACK: u64 = 0;
    pub const MAGENTA_ON_BLACK: u64 = 0;
}
