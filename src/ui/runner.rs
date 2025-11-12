//! UI runner - orchestrates screen transitions and user interaction

use crate::config::Config;
use crate::error::Result;
use crate::ui::Screen;

/// UI runner
pub struct UiRunner {
    current_screen: Screen,
    config: Config,
}

impl UiRunner {
    /// Create a new UI runner
    pub fn new(config: Config) -> Self {
        Self {
            current_screen: Screen::Welcome,
            config,
        }
    }

    /// Run the TUI workflow
    pub fn run(&mut self) -> Result<Config> {
        // TODO: Implement full TUI workflow with Notcurses
        //
        // Basic flow:
        // 1. Initialize Notcurses
        // 2. Loop through screens
        // 3. Render current screen
        // 4. Handle input
        // 5. Transition to next screen
        // 6. Cleanup on exit
        //
        // Example:
        // ```
        // let mut nc = NotcursesContext::init()?;
        //
        // loop {
        //     match self.current_screen {
        //         Screen::Welcome => self.show_welcome(&mut nc)?,
        //         Screen::DeviceSelect => self.show_device_select(&mut nc)?,
        //         // ... etc
        //         Screen::Completion => break,
        //     }
        //
        //     nc.render()?;
        // }
        // ```

        log::info!("Interactive TUI not yet implemented. Use CLI mode with --mode flag.");
        Ok(self.config.clone())
    }

    /// Get current screen
    pub fn current_screen(&self) -> Screen {
        self.current_screen
    }

    /// Navigate to next screen
    pub fn next_screen(&mut self) {
        if let Some(next) = self.current_screen.next() {
            self.current_screen = next;
        }
    }

    /// Navigate to previous screen
    pub fn previous_screen(&mut self) {
        if let Some(prev) = self.current_screen.previous() {
            self.current_screen = prev;
        }
    }
}

// TODO: Implement screen-specific rendering and interaction
// Each screen would be implemented as a separate function/struct
// that uses Notcurses to render and handle input
//
// Example device selection screen (Growlight-inspired):
// ```
// fn show_device_select(&mut self, nc: &mut NotcursesContext) -> Result<()> {
//     // Create main panel
//     let panel = nc.create_panel(0, 0, height, width)?;
//
//     // Render device tree (controller -> devices -> partitions)
//     let discovery = DeviceDiscovery::new()?;
//     let devices = discovery.scan_devices()?;
//
//     // Group by controller type
//     let mut y = 2;
//     for controller_type in [ControllerType::Nvme, ControllerType::Sata, ...] {
//         panel.putstr(y, 2, &format!("{} Controllers:", controller_type))?;
//         y += 1;
//
//         for device in devices.iter().filter(|d| d.controller_type == controller_type) {
//             let selected = self.config.devices.contains(&device.path);
//             let marker = if selected { "[*]" } else { "[ ]" };
//
//             panel.putstr(y, 4, &format!("{} {} - {}", marker, device.name, device.size_human()))?;
//             y += 1;
//
//             // Show partitions (indented)
//             for part in &device.partitions {
//                 panel.putstr(y, 6, &format!("└─ {} - {}", part.path.display(), bytesize::ByteSize(part.size)))?;
//                 y += 1;
//             }
//         }
//     }
//
//     // Handle keyboard input
//     // - Up/Down: navigate devices
//     // - Space: toggle selection
//     // - Enter: confirm and proceed
//     // - Escape: go back
//
//     Ok(())
// }
// ```
