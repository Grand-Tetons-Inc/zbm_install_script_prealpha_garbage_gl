#!/usr/bin/env python3
"""
ZFSBootMenu Notcurses UI Components

Beautiful UI components using notcurses for rich terminal graphics.
"""

import time
from typing import Dict, List, Any, Optional, Tuple
from notcurses import Notcurses, Plane, NCAlign


class BaseScreen:
    """Base class for all screens"""

    def __init__(self, nc: Notcurses):
        self.nc = nc
        self.stdplane = nc.stdplane()
        self.width = self.stdplane.dim_x()
        self.height = self.stdplane.dim_y()

    def clear(self):
        """Clear the screen"""
        self.stdplane.erase()

    def render(self):
        """Render the screen"""
        self.nc.render()

    def draw_box(self, y: int, x: int, height: int, width: int, title: str = ""):
        """Draw a box with optional title"""
        # Draw corners and borders using box-drawing characters
        self.stdplane.putstr_yx(y, x, "╔" + "═" * (width - 2) + "╗")
        for i in range(1, height - 1):
            self.stdplane.putstr_yx(y + i, x, "║")
            self.stdplane.putstr_yx(y + i, x + width - 1, "║")
        self.stdplane.putstr_yx(y + height - 1, x, "╚" + "═" * (width - 2) + "╝")

        # Draw title if provided
        if title:
            title_str = f"  {title}  "
            title_x = x + (width - len(title_str)) // 2
            self.stdplane.putstr_yx(y, title_x, title_str)

    def draw_centered_text(self, y: int, text: str, color: int = 0xffffff):
        """Draw centered text"""
        x = (self.width - len(text)) // 2
        self.stdplane.set_fg_rgb8(color >> 16, (color >> 8) & 0xff, color & 0xff)
        self.stdplane.putstr_yx(y, x, text)
        self.stdplane.set_fg_default()

    def wait_for_key(self) -> str:
        """Wait for a key press"""
        ev = self.nc.getc_blocking()
        if ev.is_release:
            return ""
        return ev.id if hasattr(ev, 'id') else ""


class WelcomeScreen(BaseScreen):
    """Welcome screen with system information"""

    def __init__(self, nc: Notcurses, system_info: Dict[str, Any]):
        super().__init__(nc)
        self.system_info = system_info

    def show(self) -> str:
        """Show welcome screen"""
        self.clear()

        # Title
        y = 2
        self.draw_centered_text(y, "╔═══════════════════════════════════════════════════════════╗", 0x00ff00)
        y += 1
        self.draw_centered_text(y, "║        ZFSBootMenu Installation - Notcurses TUI          ║", 0x00ff00)
        y += 1
        self.draw_centered_text(y, "╚═══════════════════════════════════════════════════════════╝", 0x00ff00)

        # System information box
        y += 2
        box_y = y
        box_height = 12
        box_width = 60
        box_x = (self.width - box_width) // 2

        self.draw_box(box_y, box_x, box_height, box_width, "System Information")

        # Display system info
        y = box_y + 2
        info_x = box_x + 4

        is_efi = self.system_info.get("is_efi", False)
        efi_str = "✓ Yes" if is_efi else "✗ No (BIOS not supported!)"
        efi_color = 0x00ff00 if is_efi else 0xff0000

        self.stdplane.putstr_yx(y, info_x, "EFI System:  ")
        self.stdplane.set_fg_rgb8(efi_color >> 16, (efi_color >> 8) & 0xff, efi_color & 0xff)
        self.stdplane.putstr(efi_str)
        self.stdplane.set_fg_default()

        y += 1
        ram_gb = self.system_info.get("ram_gb", 0)
        ram_ok = ram_gb >= 2
        ram_color = 0x00ff00 if ram_ok else 0xff0000
        self.stdplane.putstr_yx(y, info_x, f"RAM:         ")
        self.stdplane.set_fg_rgb8(ram_color >> 16, (ram_color >> 8) & 0xff, ram_color & 0xff)
        self.stdplane.putstr(f"{ram_gb} GB")
        self.stdplane.set_fg_default()

        y += 1
        cpu_count = self.system_info.get("cpu_count", 0)
        self.stdplane.putstr_yx(y, info_x, f"CPU Cores:   {cpu_count}")

        y += 1
        distro = self.system_info.get("distro", "Unknown")
        version = self.system_info.get("distro_version", "")
        self.stdplane.putstr_yx(y, info_x, f"Distro:      {distro} {version}")

        # Instructions
        y = box_y + box_height + 2
        self.draw_centered_text(y, "This installer will guide you through ZFS installation", 0xaaaaaa)
        y += 1
        self.draw_centered_text(y, "with support for RAID, compression, and system migration", 0xaaaaaa)

        # Controls
        y = self.height - 4
        self.draw_centered_text(y, "─" * 60, 0x666666)
        y += 1
        self.draw_centered_text(y, "[ENTER] Continue    [Q] Quit", 0x00ffff)

        self.render()

        # Wait for input
        while True:
            key = self.wait_for_key()
            if key in ('\n', '\r', ' '):
                return "next"
            elif key.lower() == 'q':
                return "quit"


class ModeSelectionScreen(BaseScreen):
    """Installation mode selection"""

    def __init__(self, nc: Notcurses):
        super().__init__(nc)
        self.selected = 0
        self.modes = [
            ("new", "New Installation", "Install ZFS on empty drives (DESTROYS data)"),
            ("existing", "Migrate System", "Copy running system to new ZFS installation")
        ]

    def show(self) -> str:
        """Show mode selection"""
        while True:
            self.clear()

            # Title
            y = 2
            self.draw_centered_text(y, "═══ Select Installation Mode ═══", 0x00ff00)

            # Mode options
            y += 4
            box_width = 70
            box_height = 8
            box_x = (self.width - box_width) // 2

            for i, (mode_id, mode_name, mode_desc) in enumerate(self.modes):
                # Highlight selected
                if i == self.selected:
                    color = 0x00ffff
                    prefix = "►"
                else:
                    color = 0xffffff
                    prefix = " "

                mode_y = y + (i * 4)
                self.stdplane.set_fg_rgb8(color >> 16, (color >> 8) & 0xff, color & 0xff)
                self.stdplane.putstr_yx(mode_y, box_x, f"{prefix} {mode_name}")
                self.stdplane.set_fg_default()

                self.stdplane.set_fg_rgb8(0xaa, 0xaa, 0xaa)
                self.stdplane.putstr_yx(mode_y + 1, box_x + 4, mode_desc)
                self.stdplane.set_fg_default()

            # Controls
            y = self.height - 4
            self.draw_centered_text(y, "─" * 60, 0x666666)
            y += 1
            self.draw_centered_text(y, "[↑/↓] Navigate  [ENTER] Select  [ESC] Back  [Q] Quit", 0x00ffff)

            self.render()

            # Handle input
            key = self.wait_for_key()

            if key == '\n' or key == '\r':
                return self.modes[self.selected][0]
            elif key in ('j', 'down'):
                self.selected = (self.selected + 1) % len(self.modes)
            elif key in ('k', 'up'):
                self.selected = (self.selected - 1) % len(self.modes)
            elif key == 'escape':
                return "back"
            elif key.lower() == 'q':
                return "quit"


class DeviceSelectionScreen(BaseScreen):
    """Device selection with checkbox list"""

    def __init__(self, nc: Notcurses, devices: Dict[str, Any], selected_drives: List[str]):
        super().__init__(nc)
        self.devices = devices
        self.selected_drives = set(selected_drives)
        self.cursor = 0
        self.device_list = sorted(devices.keys())

    def format_size(self, size_bytes: int) -> str:
        """Format size in human-readable form"""
        gb = size_bytes // (1024**3)
        if gb > 1024:
            return f"{gb // 1024}TB"
        return f"{gb}GB"

    def show(self) -> Any:
        """Show device selection"""
        while True:
            self.clear()

            # Title
            y = 2
            self.draw_centered_text(y, "═══ Select Target Drives ═══", 0x00ff00)
            y += 1
            self.draw_centered_text(y, "⚠ WARNING: Selected drives will be WIPED! ⚠", 0xff0000)

            # Device list
            y += 3
            if not self.device_list:
                self.draw_centered_text(y, "No drives detected!", 0xff0000)
            else:
                for i, dev in enumerate(self.device_list):
                    dev_info = self.devices[dev]
                    size_str = self.format_size(dev_info["size_bytes"])
                    model = dev_info["model"]
                    dev_type = dev_info["type"]

                    # Highlight cursor position
                    if i == self.cursor:
                        color = 0x00ffff
                        cursor = "►"
                    else:
                        color = 0xffffff
                        cursor = " "

                    # Checkbox
                    checked = "☑" if dev in self.selected_drives else "☐"

                    line = f"{cursor} {checked} {dev:<12} ({size_str:>6}  {model[:20]:<20}  {dev_type})"
                    x = (self.width - len(line)) // 2

                    self.stdplane.set_fg_rgb8(color >> 16, (color >> 8) & 0xff, color & 0xff)
                    self.stdplane.putstr_yx(y + i, x, line)
                    self.stdplane.set_fg_default()

            # Selected count
            y = self.height - 7
            count_text = f"Selected: {len(self.selected_drives)} drive(s)"
            self.draw_centered_text(y, count_text, 0x00ff00)

            # Controls
            y = self.height - 4
            self.draw_centered_text(y, "─" * 80, 0x666666)
            y += 1
            self.draw_centered_text(y, "[↑/↓] Navigate  [SPACE] Toggle  [ENTER] Continue  [ESC] Back  [Q] Quit", 0x00ffff)

            self.render()

            # Handle input
            key = self.wait_for_key()

            if key == '\n' or key == '\r':
                if len(self.selected_drives) > 0:
                    return list(self.selected_drives)
            elif key == ' ':
                if self.device_list:
                    dev = self.device_list[self.cursor]
                    if dev in self.selected_drives:
                        self.selected_drives.remove(dev)
                    else:
                        self.selected_drives.add(dev)
            elif key in ('j', 'down'):
                if self.device_list:
                    self.cursor = (self.cursor + 1) % len(self.device_list)
            elif key in ('k', 'up'):
                if self.device_list:
                    self.cursor = (self.cursor - 1) % len(self.device_list)
            elif key == 'escape':
                return "back"
            elif key.lower() == 'q':
                return "quit"


class SettingsScreen(BaseScreen):
    """Configuration settings with editable fields"""

    def __init__(self, nc, state):
        super().__init__(nc)
        self.state = state
        self.cursor = 0

        # Define fields based on mode
        if state.get("mode") == "existing":
            self.fields = [
                ("pool_name", "Pool Name", "text"),
                ("hostname", "Hostname", "text"),
                ("compression", "Compression", "choice", ["lz4", "zstd", "gzip-9", "off"]),
                ("raid_level", "RAID Level", "choice", ["none", "mirror", "raidz1", "raidz2", "raidz3"]),
                ("bootloader", "Bootloader", "choice", ["zbm", "systemd-boot", "refind"]),
                ("ashift", "Ashift", "choice", ["auto", "9", "12", "13"]),
                ("efi_size", "EFI Size", "text"),
                ("swap_size", "Swap Size", "text"),
                ("source_root", "Source Root", "text"),
                ("copy_home", "Copy /home", "bool"),
            ]
        else:
            self.fields = [
                ("pool_name", "Pool Name", "text"),
                ("hostname", "Hostname", "text"),
                ("compression", "Compression", "choice", ["lz4", "zstd", "gzip-9", "off"]),
                ("raid_level", "RAID Level", "choice", ["none", "mirror", "raidz1", "raidz2", "raidz3"]),
                ("bootloader", "Bootloader", "choice", ["zbm", "systemd-boot", "refind"]),
                ("ashift", "Ashift", "choice", ["auto", "9", "12", "13"]),
                ("efi_size", "EFI Size", "text"),
                ("swap_size", "Swap Size", "text"),
            ]

    def get_field_value(self, field_key):
        """Get current value for field"""
        val = self.state.get(field_key, "")
        if isinstance(val, bool):
            return "Yes" if val else "No"
        return str(val)

    def edit_text_field(self, field_key, label):
        """Simple text editing (for demo - real impl would use proper input)"""
        # For now, just cycle through some preset values or allow basic editing
        # In a real implementation, you'd capture character input
        current = self.state.get(field_key, "")

        # Show input prompt
        self.clear()
        y = self.height // 2 - 2
        self.draw_centered_text(y, f"Edit {label}", 0x00ff00)
        y += 2
        self.draw_centered_text(y, f"Current: {current}", 0xffffff)
        y += 2
        self.draw_centered_text(y, "(Press ENTER to keep, ESC to cancel)", 0xaaaaaa)
        self.render()

        key = self.wait_for_key()
        if key == 'escape':
            return None
        return current

    def cycle_choice(self, field_key, choices):
        """Cycle through choice values"""
        current = self.state.get(field_key, choices[0])
        try:
            idx = choices.index(current)
            idx = (idx + 1) % len(choices)
        except ValueError:
            idx = 0
        return choices[idx]

    def toggle_bool(self, field_key):
        """Toggle boolean value"""
        return not self.state.get(field_key, True)

    def show(self):
        """Show settings screen with editable fields"""
        while True:
            self.clear()

            # Title
            y = 2
            mode_str = "System Migration" if self.state.get("mode") == "existing" else "New Installation"
            self.draw_centered_text(y, f"═══ Configure {mode_str} ═══", 0x00ff00)

            # Settings fields
            y += 3
            box_width = 70
            start_x = (self.width - box_width) // 2

            for i, field_info in enumerate(self.fields):
                field_key = field_info[0]
                field_label = field_info[1]
                field_type = field_info[2]

                # Get current value
                value = self.get_field_value(field_key)

                # Highlight cursor
                if i == self.cursor:
                    color = 0x00ffff
                    prefix = "►"
                else:
                    color = 0xffffff
                    prefix = " "

                # Draw field
                self.stdplane.set_fg_rgb8(color >> 16, (color >> 8) & 0xff, color & 0xff)
                line = f"{prefix} {field_label:<20} : {value}"
                self.stdplane.putstr_yx(y + i, start_x, line)
                self.stdplane.set_fg_default()

            # Instructions
            y = self.height - 6
            self.draw_centered_text(y, "Press SPACE or ENTER to edit field", 0xaaaaaa)

            # Controls
            y = self.height - 4
            self.draw_centered_text(y, "─" * 70, 0x666666)
            y += 1
            self.draw_centered_text(y, "[↑/↓] Navigate  [SPACE/ENTER] Edit  [C] Continue  [ESC] Back  [Q] Quit", 0x00ffff)

            self.render()

            # Handle input
            key = self.wait_for_key()

            if key.lower() == 'c':
                return "next"
            elif key == ' ' or key == '\n' or key == '\r':
                # Edit current field
                field_info = self.fields[self.cursor]
                field_key = field_info[0]
                field_label = field_info[1]
                field_type = field_info[2]

                if field_type == "text":
                    result = self.edit_text_field(field_key, field_label)
                    if result is not None:
                        self.state[field_key] = result
                elif field_type == "choice":
                    choices = field_info[3]
                    self.state[field_key] = self.cycle_choice(field_key, choices)
                elif field_type == "bool":
                    self.state[field_key] = self.toggle_bool(field_key)

            elif key in ('j', 'down'):
                self.cursor = (self.cursor + 1) % len(self.fields)
            elif key in ('k', 'up'):
                self.cursor = (self.cursor - 1) % len(self.fields)
            elif key == 'escape':
                return "back"
            elif key.lower() == 'q':
                return "quit"


class ValidationScreen(BaseScreen):
    """Validation results display with pass/fail indicators"""

    def __init__(self, nc, state, system_info):
        super().__init__(nc)
        self.state = state
        self.system_info = system_info
        self.checks = []
        self.all_passed = False

    def run_validations(self):
        """Run all validation checks"""
        self.checks = []

        # Check 1: EFI system
        is_efi = self.system_info.get("is_efi", False)
        self.checks.append({
            "name": "EFI System Check",
            "passed": is_efi,
            "message": "System is EFI" if is_efi else "BIOS not supported",
            "critical": True
        })

        # Check 2: Sufficient RAM
        ram_gb = self.system_info.get("ram_gb", 0)
        ram_ok = ram_gb >= 2
        self.checks.append({
            "name": "RAM Check",
            "passed": ram_ok,
            "message": f"{ram_gb}GB RAM available" if ram_ok else f"Only {ram_gb}GB RAM (need 2GB+)",
            "critical": True
        })

        # Check 3: Drives selected
        drives = self.state.get("drives", [])
        drives_ok = len(drives) > 0
        self.checks.append({
            "name": "Drive Selection",
            "passed": drives_ok,
            "message": f"{len(drives)} drive(s) selected" if drives_ok else "No drives selected",
            "critical": True
        })

        # Check 4: RAID level valid for drive count
        raid_level = self.state.get("raid_level", "none")
        raid_ok = True
        raid_msg = f"RAID level: {raid_level}"

        if raid_level == "mirror" and len(drives) < 2:
            raid_ok = False
            raid_msg = "Mirror requires 2+ drives"
        elif raid_level in ["raidz1", "raidz2", "raidz3"]:
            min_drives = {"raidz1": 3, "raidz2": 4, "raidz3": 5}
            if len(drives) < min_drives[raid_level]:
                raid_ok = False
                raid_msg = f"{raid_level} requires {min_drives[raid_level]}+ drives"

        self.checks.append({
            "name": "RAID Configuration",
            "passed": raid_ok,
            "message": raid_msg,
            "critical": True
        })

        # Check 5: Pool name valid
        pool_name = self.state.get("pool_name", "")
        pool_ok = len(pool_name) > 0 and pool_name.replace("_", "").replace("-", "").isalnum()
        self.checks.append({
            "name": "Pool Name",
            "passed": pool_ok,
            "message": f"Pool name: {pool_name}" if pool_ok else "Invalid pool name",
            "critical": True
        })

        # Check 6: Hostname valid (for migration mode)
        if self.state.get("mode") == "existing":
            hostname = self.state.get("hostname", "")
            hostname_ok = len(hostname) > 0
            self.checks.append({
                "name": "Hostname",
                "passed": hostname_ok,
                "message": f"Hostname: {hostname}" if hostname_ok else "Hostname not set",
                "critical": False
            })

        # Determine if all critical checks passed
        self.all_passed = all(check["passed"] for check in self.checks if check["critical"])

    def show(self):
        """Show validation results"""
        # Run validations
        self.run_validations()

        self.clear()

        # Title
        y = 2
        if self.all_passed:
            self.draw_centered_text(y, "═══ ✓ Validation Passed ═══", 0x00ff00)
        else:
            self.draw_centered_text(y, "═══ ✗ Validation Failed ═══", 0xff0000)

        # Validation results
        y += 3
        box_width = 70
        start_x = (self.width - box_width) // 2

        for check in self.checks:
            if check["passed"]:
                icon = "✓"
                color = 0x00ff00
            else:
                icon = "✗"
                color = 0xff0000 if check["critical"] else 0xffaa00

            self.stdplane.set_fg_rgb8(color >> 16, (color >> 8) & 0xff, color & 0xff)
            self.stdplane.putstr_yx(y, start_x, f"{icon} {check['name']}")
            self.stdplane.set_fg_default()

            # Message
            self.stdplane.set_fg_rgb8(0xaa, 0xaa, 0xaa)
            self.stdplane.putstr_yx(y, start_x + 30, check["message"])
            self.stdplane.set_fg_default()

            y += 1

        # Warning or proceed message
        y = self.height - 6
        if self.all_passed:
            self.draw_centered_text(y, "All checks passed! Ready to proceed.", 0x00ff00)
        else:
            self.draw_centered_text(y, "⚠ Cannot proceed - fix issues and try again ⚠", 0xff0000)

        # Controls
        y = self.height - 4
        self.draw_centered_text(y, "─" * 70, 0x666666)
        y += 1
        if self.all_passed:
            self.draw_centered_text(y, "[ENTER] Continue  [ESC] Back  [Q] Quit", 0x00ffff)
        else:
            self.draw_centered_text(y, "[ESC] Back  [Q] Quit", 0x00ffff)

        self.render()

        while True:
            key = self.wait_for_key()
            if key in ('\n', '\r') and self.all_passed:
                return "valid"
            elif key == 'escape':
                return "back"
            elif key.lower() == 'q':
                return "quit"


class ConfirmationScreen(BaseScreen):
    """Final confirmation with complete configuration summary"""

    def __init__(self, nc, state):
        super().__init__(nc)
        self.state = state

    def show(self):
        """Show configuration summary and final confirmation"""
        self.clear()

        # Title with big warning
        y = 1
        self.draw_centered_text(y, "╔═══════════════════════════════════════════════════════════╗", 0xff0000)
        y += 1
        self.draw_centered_text(y, "║              ⚠  FINAL CONFIRMATION  ⚠                    ║", 0xff0000)
        y += 1
        self.draw_centered_text(y, "╚═══════════════════════════════════════════════════════════╝", 0xff0000)

        # Mode
        y += 2
        mode = self.state.get("mode", "")
        mode_str = "System Migration" if mode == "existing" else "New Installation"
        self.draw_centered_text(y, f"Installation Mode: {mode_str}", 0x00ffff)

        # Configuration summary
        y += 2
        box_width = 70
        start_x = (self.width - box_width) // 2

        # Target drives (with warning)
        y += 1
        self.stdplane.set_fg_rgb8(0xff, 0x00, 0x00)
        self.stdplane.putstr_yx(y, start_x, "⚠ Target Drives (WILL BE WIPED):")
        self.stdplane.set_fg_default()

        drives = self.state.get("drives", [])
        for drive in drives:
            y += 1
            self.stdplane.putstr_yx(y, start_x + 4, f"• /dev/{drive}")

        # ZFS Configuration
        y += 2
        self.stdplane.set_fg_rgb8(0x00, 0xff, 0x00)
        self.stdplane.putstr_yx(y, start_x, "ZFS Configuration:")
        self.stdplane.set_fg_default()

        y += 1
        pool_name = self.state.get("pool_name", "zroot")
        self.stdplane.putstr_yx(y, start_x + 4, f"Pool Name:    {pool_name}")

        y += 1
        raid = self.state.get("raid_level", "none")
        self.stdplane.putstr_yx(y, start_x + 4, f"RAID Level:   {raid}")

        y += 1
        compression = self.state.get("compression", "zstd")
        self.stdplane.putstr_yx(y, start_x + 4, f"Compression:  {compression}")

        y += 1
        ashift = self.state.get("ashift", "auto")
        self.stdplane.putstr_yx(y, start_x + 4, f"Ashift:       {ashift}")

        # Bootloader
        y += 2
        self.stdplane.set_fg_rgb8(0x00, 0xff, 0x00)
        self.stdplane.putstr_yx(y, start_x, "Bootloader:")
        self.stdplane.set_fg_default()

        y += 1
        bootloader = self.state.get("bootloader", "zbm")
        bl_desc = {"zbm": "ZFSBootMenu (standalone)", "systemd-boot": "systemd-boot + ZBM", "refind": "rEFInd + ZBM"}
        self.stdplane.putstr_yx(y, start_x + 4, f"Type:         {bl_desc.get(bootloader, bootloader)}")

        # Partition sizes
        y += 2
        self.stdplane.set_fg_rgb8(0x00, 0xff, 0x00)
        self.stdplane.putstr_yx(y, start_x, "Partitions:")
        self.stdplane.set_fg_default()

        y += 1
        efi_size = self.state.get("efi_size", "1G")
        self.stdplane.putstr_yx(y, start_x + 4, f"EFI Size:     {efi_size}")

        y += 1
        swap_size = self.state.get("swap_size", "8G")
        self.stdplane.putstr_yx(y, start_x + 4, f"Swap Size:    {swap_size}")

        # Migration-specific options
        if mode == "existing":
            y += 2
            self.stdplane.set_fg_rgb8(0x00, 0xff, 0x00)
            self.stdplane.putstr_yx(y, start_x, "Migration Options:")
            self.stdplane.set_fg_default()

            y += 1
            source = self.state.get("source_root", "/")
            self.stdplane.putstr_yx(y, start_x + 4, f"Source Root:  {source}")

            y += 1
            hostname = self.state.get("hostname", "")
            self.stdplane.putstr_yx(y, start_x + 4, f"New Hostname: {hostname}")

            y += 1
            copy_home = self.state.get("copy_home", True)
            copy_home_str = "Yes" if copy_home else "No"
            self.stdplane.putstr_yx(y, start_x + 4, f"Copy /home:   {copy_home_str}")

        # Final warning
        y = self.height - 8
        self.draw_centered_text(y, "═" * 70, 0x666666)
        y += 1
        self.draw_centered_text(y, "⚠  THIS ACTION CANNOT BE UNDONE  ⚠", 0xff0000)
        y += 1
        self.draw_centered_text(y, "All data on target drives will be permanently destroyed!", 0xff0000)

        # Controls
        y = self.height - 4
        self.draw_centered_text(y, "─" * 70, 0x666666)
        y += 1
        self.draw_centered_text(y, "[Y] PROCEED WITH INSTALLATION  [N] Go Back  [Q] Quit", 0x00ffff)

        self.render()

        while True:
            key = self.wait_for_key()
            if key.lower() == 'y':
                return "proceed"
            elif key.lower() == 'n' or key == 'escape':
                return "back"
            elif key.lower() == 'q':
                return "quit"


class InstallationScreen(BaseScreen):
    """Live installation progress display"""

    def __init__(self, nc, state):
        super().__init__(nc)
        self.state = state
        self.steps = []
        self.current_step = 0
        self.log_lines = []
        self.max_log_lines = 10

    def init_steps(self):
        """Initialize installation steps based on mode"""
        mode = self.state.get("mode", "new")

        if mode == "existing":
            self.steps = [
                "Checking device fitness",
                "Preparing target drives",
                "Creating partitions",
                "Setting up ZFS pool",
                "Mounting filesystems",
                "Copying system files",
                "Zapping network configuration",
                "Installing bootloader",
                "Finalizing installation"
            ]
        else:
            self.steps = [
                "Checking device fitness",
                "Preparing target drives",
                "Creating partitions",
                "Setting up ZFS pool",
                "Installing base system",
                "Installing bootloader",
                "Finalizing installation"
            ]

    def add_log(self, message):
        """Add a log line"""
        self.log_lines.append(message)
        if len(self.log_lines) > self.max_log_lines:
            self.log_lines.pop(0)

    def draw_progress_bar(self, y, x, width, percent):
        """Draw a progress bar"""
        filled = int(width * percent / 100)
        bar = "█" * filled + "░" * (width - filled)

        # Color based on progress
        if percent < 30:
            color = 0xff0000
        elif percent < 70:
            color = 0xffff00
        else:
            color = 0x00ff00

        self.stdplane.set_fg_rgb8(color >> 16, (color >> 8) & 0xff, color & 0xff)
        self.stdplane.putstr_yx(y, x, bar)
        self.stdplane.set_fg_default()

        # Percentage
        percent_str = f" {percent}%"
        self.stdplane.putstr_yx(y, x + width + 2, percent_str)

    def update_display(self):
        """Update the installation display"""
        self.clear()

        # Title
        y = 2
        self.draw_centered_text(y, "═══ Installation in Progress ═══", 0x00ff00)

        # Overall progress
        y += 3
        percent = int((self.current_step / len(self.steps)) * 100)
        self.draw_centered_text(y, f"Overall Progress: Step {self.current_step + 1} of {len(self.steps)}", 0xffffff)

        y += 1
        bar_width = 50
        bar_x = (self.width - bar_width) // 2
        self.draw_progress_bar(y, bar_x, bar_width, percent)

        # Steps list
        y += 3
        step_x = (self.width - 60) // 2

        for i, step in enumerate(self.steps):
            if i < self.current_step:
                icon = "✓"
                color = 0x00ff00
            elif i == self.current_step:
                icon = "⟳"
                color = 0x00ffff
            else:
                icon = "○"
                color = 0x666666

            self.stdplane.set_fg_rgb8(color >> 16, (color >> 8) & 0xff, color & 0xff)
            self.stdplane.putstr_yx(y + i, step_x, f"{icon} {step}")
            self.stdplane.set_fg_default()

        # Log output
        y = self.height - 14
        self.draw_centered_text(y, "─" * 70, 0x666666)
        y += 1
        self.draw_centered_text(y, "Log Output", 0xaaaaaa)
        y += 1

        for log_line in self.log_lines:
            self.stdplane.set_fg_rgb8(0xaa, 0xaa, 0xaa)
            log_x = (self.width - 68) // 2
            self.stdplane.putstr_yx(y, log_x, log_line[:68])
            self.stdplane.set_fg_default()
            y += 1

        # Status
        y = self.height - 3
        self.draw_centered_text(y, "Please wait... Do not interrupt!", 0xffff00)

        self.render()

    def show(self):
        """Run installation with live progress"""
        self.init_steps()

        # Simulate installation steps
        for i, step in enumerate(self.steps):
            self.current_step = i
            self.add_log(f"Starting: {step}")
            self.update_display()

            # Simulate work (in real implementation, this would call bash backend)
            time.sleep(0.5)

            self.add_log(f"Completed: {step}")
            self.update_display()
            time.sleep(0.3)

        # Mark complete
        self.current_step = len(self.steps)
        self.update_display()

        time.sleep(1)
        return "success"


class CompletionScreen(BaseScreen):
    def __init__(self, nc, state):
        super().__init__(nc)
        self.state = state

    def show(self):
        self.clear()
        self.draw_centered_text(5, "✓ Installation Complete!", 0x00ff00)
        self.draw_centered_text(7, "Your system is ready to reboot", 0xffffff)
        self.draw_centered_text(self.height - 3, "[ENTER] Exit", 0x00ffff)
        self.render()

        self.wait_for_key()
        return "quit"
