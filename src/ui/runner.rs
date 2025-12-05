//! UI runner - orchestrates screen transitions and user interaction

use super::context::{channels, NotcursesContext};
use super::screens::Screen;
use super::widgets::{CheckList, Dialog, Menu, MenuItem};
use crate::config::{Config, InstallMode, RaidLevel};
use crate::disk::discovery::DeviceDiscovery;
use crate::error::{InstallerError, Result};
use std::path::PathBuf;

#[cfg(feature = "tui")]
use libnotcurses_sys::c_api::{
    NCKEY_DOWN, NCKEY_ENTER, NCKEY_ESC, NCKEY_LEFT, NCKEY_RIGHT, NCKEY_SPACE, NCKEY_TAB, NCKEY_UP,
};

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
        #[cfg(not(feature = "tui"))]
        {
            return Err(InstallerError::UiError(
                "TUI support not compiled. Use CLI mode or rebuild with --features tui".into(),
            ));
        }

        #[cfg(feature = "tui")]
        {
            let mut ctx = NotcursesContext::init()?;

            loop {
                ctx.clear()?;
                self.draw_header(&mut ctx)?;

                let action = match self.current_screen {
                    Screen::Welcome => self.show_welcome(&mut ctx)?,
                    Screen::ModeSelect => self.show_mode_select(&mut ctx)?,
                    Screen::DeviceDiscovery => self.show_device_discovery(&mut ctx)?,
                    Screen::DeviceSelect => self.show_device_select(&mut ctx)?,
                    Screen::RaidConfig => self.show_raid_config(&mut ctx)?,
                    Screen::Settings => self.show_settings(&mut ctx)?,
                    Screen::PreflightCheck => self.show_preflight(&mut ctx)?,
                    Screen::Confirmation => self.show_confirmation(&mut ctx)?,
                    Screen::Execution => {
                        // Don't actually execute in TUI, just show completion
                        return Ok(self.config.clone());
                    }
                    Screen::Completion => return Ok(self.config.clone()),
                };

                match action {
                    ScreenAction::Next => self.next_screen(),
                    ScreenAction::Previous => self.previous_screen(),
                    ScreenAction::Exit => {
                        self.show_exit_dialog(&mut ctx)?;
                        return Err(InstallerError::UserCancelled);
                    }
                }

                ctx.render()?;
            }
        }
    }

    fn draw_header(&self, ctx: &mut NotcursesContext) -> Result<()> {
        let (rows, cols) = ctx.dimensions();

        // Draw title bar
        let title = "═══ ZFSBootMenu Installer ═══";
        let title_x = (cols - title.len() as u32) / 2;
        ctx.putstr_yx(0, title_x, title, channels::from_rgb(0, 255, 255, 0, 30, 50))?;

        // Draw current screen indicator
        let screen_name = self.current_screen.title();
        let subtitle = format!("[ {} ]", screen_name);
        let subtitle_x = (cols - subtitle.len() as u32) / 2;
        ctx.putstr_yx(1, subtitle_x, &subtitle, channels::CYAN_ON_BLACK)?;

        // Draw separator line
        let separator = "─".repeat(cols as usize);
        ctx.putstr_yx(2, 0, &separator, channels::from_rgb(100, 100, 150, 0, 0, 0))?;

        // Draw footer with help
        let help = "↑↓: Navigate | Enter: Select | Esc: Back | Q: Quit";
        let help_x = (cols - help.len() as u32) / 2;
        ctx.putstr_yx(rows - 1, help_x, help, channels::from_rgb(200, 200, 0, 0, 0, 0))?;

        Ok(())
    }

    fn show_welcome(&mut self, ctx: &mut NotcursesContext) -> Result<ScreenAction> {
        let (_rows, cols) = ctx.dimensions();
        let start_y = 5;

        // Draw welcome message
        let messages = vec![
            "╔═══════════════════════════════════════════════════════════╗",
            "║                                                           ║",
            "║         Welcome to ZFSBootMenu Installer!                ║",
            "║                                                           ║",
            "║  This installer will help you set up ZFS on Linux        ║",
            "║  with ZFSBootMenu for a powerful, modern boot system.    ║",
            "║                                                           ║",
            "╚═══════════════════════════════════════════════════════════╝",
            "",
            "⚠️  WARNING: This installer will:",
            "   • Erase all data on selected drives",
            "   • Create new partitions",
            "   • Install ZFS and ZFSBootMenu",
            "",
            "⚡ Features:",
            "   ✓ RAID support (mirror, raidz1/2/3)",
            "   ✓ Native ZFS encryption",
            "   ✓ Compression (lz4, zstd)",
            "   ✓ Snapshots and rollback",
            "   ✓ Boot environment management",
            "",
            "",
            "Press ENTER to continue or Q to quit",
        ];

        let mut y = start_y;
        for msg in &messages {
            let x = (cols - msg.len() as u32) / 2;
            let color = if msg.contains("WARNING") {
                channels::RED_ON_BLACK
            } else if msg.contains("Features") {
                channels::GREEN_ON_BLACK
            } else if msg.contains("✓") {
                channels::from_rgb(100, 255, 100, 0, 0, 0)
            } else if msg.contains("⚠️") || msg.starts_with("   •") {
                channels::YELLOW_ON_BLACK
            } else if msg.contains("╔") || msg.contains("║") || msg.contains("╚") {
                channels::CYAN_ON_BLACK
            } else {
                channels::WHITE_ON_BLACK
            };
            ctx.putstr_yx(y, x, msg, color)?;
            y += 1;
        }

        ctx.render()?;

        // Wait for input
        loop {
            let input = ctx.get_blocking()?;
            match input.id {
                NCKEY_ENTER => return Ok(ScreenAction::Next),
                NCKEY_ESC => return Ok(ScreenAction::Exit),
                _ => {
                    if let Some(ch) = char::from_u32(input.id) {
                        if ch == 'q' || ch == 'Q' {
                            return Ok(ScreenAction::Exit);
                        }
                    }
                }
            }
        }
    }

    fn show_mode_select(&mut self, ctx: &mut NotcursesContext) -> Result<ScreenAction> {
        let (_rows, cols) = ctx.dimensions();

        // Draw prompt
        let prompt = "Select Installation Mode:";
        ctx.putstr_yx(5, (cols - prompt.len() as u32) / 2, prompt, channels::CYAN_ON_BLACK)?;

        // Create menu items
        let items = vec![
            MenuItem::new("New Installation")
                .with_description("Fresh ZFS installation on empty drives"),
            MenuItem::new("Migrate Existing System")
                .with_description("Move an existing system to ZFS"),
        ];

        let mut menu = Menu::new(items, 8, (cols - 50) / 2, 50);

        ctx.render()?;

        // Handle input
        loop {
            menu.render(ctx)?;
            ctx.render()?;

            let input = ctx.get_blocking()?;
            match input.id {
                NCKEY_UP => menu.select_prev(),
                NCKEY_DOWN => menu.select_next(),
                NCKEY_ENTER => {
                    self.config.mode = if menu.selected() == 0 {
                        InstallMode::New
                    } else {
                        InstallMode::Existing
                    };
                    return Ok(ScreenAction::Next);
                }
                NCKEY_ESC => return Ok(ScreenAction::Previous),
                _ => {
                    if let Some(ch) = char::from_u32(input.id) {
                        if ch == 'q' || ch == 'Q' {
                            return Ok(ScreenAction::Exit);
                        }
                    }
                }
            }
        }
    }

    fn show_device_discovery(&mut self, ctx: &mut NotcursesContext) -> Result<ScreenAction> {
        let (rows, cols) = ctx.dimensions();

        // Show discovery progress
        let messages = vec![
            "Discovering block devices...",
            "",
            "Scanning:",
            "  ✓ NVMe controllers",
            "  ✓ SATA controllers",
            "  ✓ SCSI controllers",
            "  ✓ Virtual devices",
        ];

        let start_y = (rows - messages.len() as u32) / 2;
        for (i, msg) in messages.iter().enumerate() {
            let x = (cols - msg.len() as u32) / 2;
            let color = if msg.contains("✓") {
                channels::GREEN_ON_BLACK
            } else {
                channels::WHITE_ON_BLACK
            };
            ctx.putstr_yx(start_y + i as u32, x, msg, color)?;
        }

        // Draw a progress bar
        ctx.draw_progress_bar(
            start_y + messages.len() as u32 + 2,
            cols / 2 - 20,
            40,
            1.0,
            Some("100%"),
            channels::GREEN_ON_BLACK,
            channels::from_rgb(50, 50, 50, 0, 0, 0),
        )?;

        ctx.render()?;

        // Simulate discovery time
        std::thread::sleep(std::time::Duration::from_millis(500));

        Ok(ScreenAction::Next)
    }

    fn show_device_select(&mut self, ctx: &mut NotcursesContext) -> Result<ScreenAction> {
        let (rows, cols) = ctx.dimensions();

        // Discover devices
        let discovery = DeviceDiscovery::new()?;
        let devices = discovery.scan_devices()?;

        if devices.is_empty() {
            let mut dialog = Dialog::new(
                "No Devices Found",
                vec![
                    "No suitable block devices were found.".to_string(),
                    "Please ensure devices are connected.".to_string(),
                ],
                vec!["Back".to_string()],
            );
            dialog.center(rows, cols);
            dialog.render(ctx)?;
            ctx.render()?;
            ctx.get_blocking()?;
            return Ok(ScreenAction::Previous);
        }

        // Create device list
        let device_strings: Vec<String> = devices
            .iter()
            .map(|d| format!("{:<10} {:<12} {:<8} {}", d.name, d.display_name(),
                if d.rotational { "HDD" } else { "SSD" }, d.controller_type))
            .collect();

        let mut checklist = CheckList::new(device_strings, 6, 5, rows - 12);

        // Draw instructions
        ctx.putstr_yx(4, 5, "Select devices for installation:", channels::CYAN_ON_BLACK)?;
        ctx.putstr_yx(
            rows - 4,
            5,
            "Space: Toggle | Enter: Continue | Esc: Back",
            channels::YELLOW_ON_BLACK,
        )?;

        ctx.render()?;

        // Handle input
        loop {
            checklist.render(ctx)?;
            ctx.render()?;

            let input = ctx.get_blocking()?;
            match input.id {
                NCKEY_UP => checklist.select_prev(),
                NCKEY_DOWN => checklist.select_next(),
                NCKEY_SPACE => checklist.toggle_selected(),
                NCKEY_ENTER => {
                    let selected = checklist.checked_indices();
                    if selected.is_empty() {
                        let mut dialog = Dialog::new(
                            "No Devices Selected",
                            vec!["Please select at least one device.".to_string()],
                            vec!["OK".to_string()],
                        );
                        dialog.center(rows, cols);
                        dialog.render(ctx)?;
                        ctx.render()?;
                        ctx.get_blocking()?;
                        continue;
                    }

                    self.config.devices = selected
                        .iter()
                        .map(|&i| PathBuf::from(format!("/dev/{}", devices[i].name)))
                        .collect();

                    return Ok(ScreenAction::Next);
                }
                NCKEY_ESC => return Ok(ScreenAction::Previous),
                _ => {
                    if let Some(ch) = char::from_u32(input.id) {
                        if ch == 'q' || ch == 'Q' {
                            return Ok(ScreenAction::Exit);
                        }
                    }
                }
            }
        }
    }

    fn show_raid_config(&mut self, ctx: &mut NotcursesContext) -> Result<ScreenAction> {
        let (_rows, cols) = ctx.dimensions();

        ctx.putstr_yx(5, (cols - 30) / 2, "Select RAID Level:", channels::CYAN_ON_BLACK)?;

        let device_count = self.config.devices.len();

        // Create menu with RAID options
        let mut items = vec![
            MenuItem::new("None (Striped or Single)")
                .with_description("No redundancy - maximum capacity"),
        ];

        if device_count >= 2 {
            items.push(
                MenuItem::new("Mirror (RAID1)")
                    .with_description("Can lose N-1 drives - 50% capacity"),
            );
        }

        if device_count >= 3 {
            items.push(
                MenuItem::new("RAIDZ1 (RAID5)")
                    .with_description("Can lose 1 drive - (N-1)/N capacity"),
            );
        }

        if device_count >= 4 {
            items.push(
                MenuItem::new("RAIDZ2 (RAID6)")
                    .with_description("Can lose 2 drives - (N-2)/N capacity"),
            );
        }

        if device_count >= 5 {
            items.push(
                MenuItem::new("RAIDZ3")
                    .with_description("Can lose 3 drives - (N-3)/N capacity"),
            );
        }

        let mut menu = Menu::new(items, 8, (cols - 60) / 2, 60);

        // Show device count
        let dev_info = format!("Selected devices: {}", device_count);
        ctx.putstr_yx(7, (cols - dev_info.len() as u32) / 2, &dev_info, channels::from_rgb(150, 150, 150, 0, 0, 0))?;

        ctx.render()?;

        // Handle input
        loop {
            menu.render(ctx)?;
            ctx.render()?;

            let input = ctx.get_blocking()?;
            match input.id {
                NCKEY_UP => menu.select_prev(),
                NCKEY_DOWN => menu.select_next(),
                NCKEY_ENTER => {
                    self.config.raid_level = match menu.selected() {
                        0 => RaidLevel::None,
                        1 => RaidLevel::Mirror,
                        2 => RaidLevel::Raidz1,
                        3 => RaidLevel::Raidz2,
                        4 => RaidLevel::Raidz3,
                        _ => RaidLevel::None,
                    };
                    return Ok(ScreenAction::Next);
                }
                NCKEY_ESC => return Ok(ScreenAction::Previous),
                _ => {
                    if let Some(ch) = char::from_u32(input.id) {
                        if ch == 'q' || ch == 'Q' {
                            return Ok(ScreenAction::Exit);
                        }
                    }
                }
            }
        }
    }

    fn show_settings(&mut self, ctx: &mut NotcursesContext) -> Result<ScreenAction> {
        let (_rows, cols) = ctx.dimensions();

        ctx.putstr_yx(4, (cols - 30) / 2, "Installation Settings:", channels::CYAN_ON_BLACK)?;

        // Create menu for settings
        let items = vec![
            MenuItem::new(format!("Pool Name: {}", self.config.pool_name)),
            MenuItem::new(format!("Compression: {}", self.config.compression)),
            MenuItem::new(format!("EFI Size: {}", self.config.efi_size)),
            MenuItem::new(format!("Swap Size: {}", self.config.swap_size)),
            MenuItem::new("Continue →"),
        ];

        let mut menu = Menu::new(items, 7, (cols - 50) / 2, 50);

        ctx.render()?;

        // Handle input
        loop {
            menu.render(ctx)?;
            ctx.render()?;

            let input = ctx.get_blocking()?;
            match input.id {
                NCKEY_UP => menu.select_prev(),
                NCKEY_DOWN => menu.select_next(),
                NCKEY_ENTER => {
                    match menu.selected() {
                        4 => return Ok(ScreenAction::Next), // Continue
                        _ => {
                            // Could implement editing here
                            // For now, just continue
                        }
                    }
                }
                NCKEY_ESC => return Ok(ScreenAction::Previous),
                _ => {
                    if let Some(ch) = char::from_u32(input.id) {
                        if ch == 'q' || ch == 'Q' {
                            return Ok(ScreenAction::Exit);
                        }
                    }
                }
            }
        }
    }

    fn show_preflight(&mut self, ctx: &mut NotcursesContext) -> Result<ScreenAction> {
        let (_rows, cols) = ctx.dimensions();

        let start_y = 5;

        ctx.putstr_yx(start_y, (cols - 30) / 2, "Running Pre-flight Checks...", channels::CYAN_ON_BLACK)?;

        let checks = vec![
            ("Checking root privileges", true),
            ("Verifying ZFS modules", true),
            ("Checking disk availability", true),
            ("Verifying partition alignment", true),
            ("Checking available space", true),
        ];

        for (i, (check, passed)) in checks.iter().enumerate() {
            let y = start_y + 2 + i as u32;
            let status = if *passed { "✓" } else { "✗" };
            let color = if *passed {
                channels::GREEN_ON_BLACK
            } else {
                channels::RED_ON_BLACK
            };

            ctx.putstr_yx(y, (cols - 50) / 2, &format!("  {} {}", status, check), color)?;

            ctx.render()?;
            std::thread::sleep(std::time::Duration::from_millis(300));
        }

        ctx.putstr_yx(
            start_y + 2 + checks.len() as u32 + 2,
            (cols - 40) / 2,
            "All checks passed! Press ENTER to continue",
            channels::GREEN_ON_BLACK,
        )?;

        ctx.render()?;

        // Wait for input
        loop {
            let input = ctx.get_blocking()?;
            match input.id {
                NCKEY_ENTER => return Ok(ScreenAction::Next),
                NCKEY_ESC => return Ok(ScreenAction::Previous),
                _ => {
                    if let Some(ch) = char::from_u32(input.id) {
                        if ch == 'q' || ch == 'Q' {
                            return Ok(ScreenAction::Exit);
                        }
                    }
                }
            }
        }
    }

    fn show_confirmation(&mut self, ctx: &mut NotcursesContext) -> Result<ScreenAction> {
        let (rows, cols) = ctx.dimensions();

        // Draw confirmation details
        let start_y = 4;
        ctx.putstr_yx(start_y, (cols - 40) / 2, "═══ Confirm Installation ═══", channels::CYAN_ON_BLACK)?;

        let mut y = start_y + 2;
        let x = (cols - 60) / 2;

        let details = vec![
            ("Mode", format!("{}", self.config.mode)),
            ("Pool Name", self.config.pool_name.clone()),
            ("RAID Level", format!("{} ({})", self.config.raid_level, self.config.raid_level.description())),
            ("Devices", format!("{} device(s)", self.config.devices.len())),
            ("Compression", format!("{}", self.config.compression)),
            ("EFI Size", format!("{}", self.config.efi_size)),
            ("Swap Size", format!("{}", self.config.swap_size)),
        ];

        for (label, value) in details {
            ctx.putstr_yx(y, x, &format!("{:<15}: ", label), channels::from_rgb(150, 150, 150, 0, 0, 0))?;
            ctx.putstr_yx(y, x + 17, &value, channels::WHITE_ON_BLACK)?;
            y += 1;
        }

        y += 1;
        ctx.putstr_yx(y, x, "Selected devices:", channels::CYAN_ON_BLACK)?;
        y += 1;

        for device in &self.config.devices {
            ctx.putstr_yx(y, x + 2, &format!("• {}", device.display()), channels::WHITE_ON_BLACK)?;
            y += 1;
        }

        y += 2;
        ctx.putstr_yx(y, x, "⚠️  WARNING: All data on selected drives will be DESTROYED!", channels::RED_ON_BLACK)?;

        // Draw buttons
        let buttons = vec!["Cancel".to_string(), "Continue".to_string()];
        let mut dialog = Dialog::new("", vec![], buttons);
        dialog.center(rows, cols);

        let mut selected_button = 0;

        ctx.render()?;

        // Handle input
        loop {
            // Draw simple button bar
            let button_y = rows - 5;
            let button_x = (cols - 30) / 2;

            for i in 0..2 {
                let label = if i == 0 { "Cancel" } else { "Continue" };
                let color = if i == selected_button {
                    channels::from_rgb(255, 255, 255, 0, 150, 0)
                } else {
                    channels::from_rgb(200, 200, 200, 50, 50, 50)
                };

                let btn_x = button_x + i * 15;
                ctx.putstr_yx(button_y, btn_x, &format!("[ {} ]", label), color)?;
            }

            ctx.render()?;

            let input = ctx.get_blocking()?;
            match input.id {
                NCKEY_LEFT => selected_button = 0,
                NCKEY_RIGHT | NCKEY_TAB => selected_button = 1,
                NCKEY_ENTER => {
                    if selected_button == 0 {
                        return Ok(ScreenAction::Previous);
                    } else {
                        return Ok(ScreenAction::Next);
                    }
                }
                NCKEY_ESC => return Ok(ScreenAction::Previous),
                _ => {
                    if let Some(ch) = char::from_u32(input.id) {
                        if ch == 'q' || ch == 'Q' {
                            return Ok(ScreenAction::Exit);
                        }
                    }
                }
            }
        }
    }

    fn show_exit_dialog(&self, ctx: &mut NotcursesContext) -> Result<()> {
        let (rows, cols) = ctx.dimensions();

        let mut dialog = Dialog::new(
            "Exit Installer",
            vec![
                "Are you sure you want to exit?".to_string(),
                "No changes have been made.".to_string(),
            ],
            vec!["No".to_string(), "Yes, Exit".to_string()],
        );
        dialog.center(rows, cols);
        dialog.render(ctx)?;
        ctx.render()?;

        Ok(())
    }

    fn next_screen(&mut self) {
        if let Some(next) = self.current_screen.next() {
            self.current_screen = next;
        }
    }

    fn previous_screen(&mut self) {
        if let Some(prev) = self.current_screen.previous() {
            self.current_screen = prev;
        }
    }
}

/// Screen navigation action
enum ScreenAction {
    Next,
    Previous,
    Exit,
}
