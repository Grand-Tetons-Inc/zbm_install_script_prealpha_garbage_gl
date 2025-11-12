//! Screen definitions for the TUI

/// Screens in the installer flow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Welcome,
    ModeSelect,
    DeviceDiscovery,
    DeviceSelect,
    RaidConfig,
    Settings,
    PreflightCheck,
    Confirmation,
    Execution,
    Completion,
}

impl Screen {
    /// Get the title for this screen
    pub fn title(&self) -> &'static str {
        match self {
            Self::Welcome => "Welcome to ZFSBootMenu Installer",
            Self::ModeSelect => "Select Installation Mode",
            Self::DeviceDiscovery => "Discovering Devices",
            Self::DeviceSelect => "Select Devices",
            Self::RaidConfig => "RAID Configuration",
            Self::Settings => "Installation Settings",
            Self::PreflightCheck => "Pre-flight Checks",
            Self::Confirmation => "Confirm Installation",
            Self::Execution => "Installing",
            Self::Completion => "Installation Complete",
        }
    }

    /// Get the next screen
    pub fn next(&self) -> Option<Screen> {
        match self {
            Self::Welcome => Some(Self::ModeSelect),
            Self::ModeSelect => Some(Self::DeviceDiscovery),
            Self::DeviceDiscovery => Some(Self::DeviceSelect),
            Self::DeviceSelect => Some(Self::RaidConfig),
            Self::RaidConfig => Some(Self::Settings),
            Self::Settings => Some(Self::PreflightCheck),
            Self::PreflightCheck => Some(Self::Confirmation),
            Self::Confirmation => Some(Self::Execution),
            Self::Execution => Some(Self::Completion),
            Self::Completion => None,
        }
    }

    /// Get the previous screen
    pub fn previous(&self) -> Option<Screen> {
        match self {
            Self::Welcome => None,
            Self::ModeSelect => Some(Self::Welcome),
            Self::DeviceDiscovery => Some(Self::ModeSelect),
            Self::DeviceSelect => Some(Self::DeviceDiscovery),
            Self::RaidConfig => Some(Self::DeviceSelect),
            Self::Settings => Some(Self::RaidConfig),
            Self::PreflightCheck => Some(Self::Settings),
            Self::Confirmation => Some(Self::PreflightCheck),
            Self::Execution => None, // Can't go back during execution
            Self::Completion => None,
        }
    }
}

// TODO: Implement screen renderers using Notcurses
// Each screen would have:
// - render() method using Notcurses primitives
// - handle_input() for keyboard/mouse events
// - update() for dynamic content (e.g., device discovery)
//
// Example:
// ```rust
// pub struct WelcomeScreen {
//     nc: NotcursesContext,
// }
//
// impl WelcomeScreen {
//     pub fn render(&mut self) -> Result<()> {
//         // Draw title
//         // Draw warning about data loss
//         // Draw "Press any key to continue"
//     }
//
//     pub fn handle_input(&mut self) -> Result<ScreenAction> {
//         // Wait for keypress
//         // Return ScreenAction::Next
//     }
// }
// ```
