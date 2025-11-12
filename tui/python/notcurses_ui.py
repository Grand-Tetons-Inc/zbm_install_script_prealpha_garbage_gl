#!/usr/bin/env python3
"""
ZFSBootMenu Notcurses TUI - Main Entry Point

Beautiful, interactive installer using notcurses for rich terminal graphics.
Provides:
- 24-bit true color graphics
- Real-time progress visualization
- System monitoring dashboard
- Animated transitions
- Mouse support
"""

import sys
import json
import signal
from typing import Dict, List, Any, Optional
from notcurses import Notcurses, NCScale, NCBlitter

# Try to import notcurses, fallback gracefully
try:
    from notcurses import Notcurses
    NOTCURSES_AVAILABLE = True
except ImportError:
    NOTCURSES_AVAILABLE = False
    print("ERROR: python-notcurses not installed", file=sys.stderr)
    print("Install: pip3 install notcurses", file=sys.stderr)
    sys.exit(1)

from notcurses_components import (
    WelcomeScreen,
    ModeSelectionScreen,
    DeviceSelectionScreen,
    SettingsScreen,
    ValidationScreen,
    ConfirmationScreen,
    InstallationScreen,
    CompletionScreen
)


class ZBMNotcursesTUI:
    """Main TUI application using notcurses"""

    def __init__(self):
        self.nc: Optional[Notcurses] = None
        self.current_screen = "welcome"
        self.state: Dict[str, Any] = {
            "mode": None,
            "drives": [],
            "pool_name": "zroot",
            "raid_level": "none",
            "bootloader": "zbm",
            "compression": "zstd",
            "ashift": "",
            "efi_size": "1G",
            "swap_size": "8G",
            "hostname": "",
            "source_root": "/",
            "copy_home": True,
            "exclude_paths": []
        }
        self.system_info: Dict[str, Any] = {}
        self.devices: Dict[str, Any] = {}

    def init_notcurses(self) -> bool:
        """Initialize notcurses"""
        try:
            self.nc = Notcurses()
            # Enable mouse support
            self.nc.mouse_enable()
            return True
        except Exception as e:
            print(f"Failed to initialize notcurses: {e}", file=sys.stderr)
            return False

    def cleanup(self):
        """Cleanup notcurses"""
        if self.nc:
            self.nc.mouse_disable()
            self.nc.stop()

    def load_system_info(self):
        """Load system information from bash backend"""
        # Read JSON from stdin (sent by bash wrapper)
        try:
            data = json.loads(sys.stdin.readline())
            self.system_info = data.get("system_info", {})
            self.devices = data.get("devices", {})
        except json.JSONDecodeError:
            # Fallback: detect directly
            self.detect_system_info()

    def detect_system_info(self):
        """Detect system info directly"""
        import os

        # Check EFI
        self.system_info["is_efi"] = os.path.isdir("/sys/firmware/efi")

        # Get RAM
        try:
            with open("/proc/meminfo") as f:
                for line in f:
                    if line.startswith("MemTotal:"):
                        kb = int(line.split()[1])
                        self.system_info["ram_gb"] = kb // (1024 * 1024)
                        break
        except:
            self.system_info["ram_gb"] = 0

        # Get CPU count
        try:
            with open("/proc/cpuinfo") as f:
                self.system_info["cpu_count"] = sum(1 for line in f if line.startswith("processor"))
        except:
            self.system_info["cpu_count"] = 0

        # Detect distro
        try:
            with open("/etc/os-release") as f:
                for line in f:
                    if line.startswith("ID="):
                        self.system_info["distro"] = line.split("=")[1].strip().strip('"')
                    elif line.startswith("VERSION_ID="):
                        self.system_info["distro_version"] = line.split("=")[1].strip().strip('"')
        except:
            pass

    def detect_devices(self):
        """Detect block devices from /sys/block"""
        import os

        self.devices = {}

        try:
            for device in os.listdir("/sys/block"):
                # Skip loop, ram, etc.
                if device.startswith(("loop", "ram", "dm-")):
                    continue

                dev_path = f"/sys/block/{device}"

                # Get size
                try:
                    with open(f"{dev_path}/size") as f:
                        sectors = int(f.read().strip())
                        size_bytes = sectors * 512
                except:
                    size_bytes = 0

                # Get model
                try:
                    with open(f"{dev_path}/device/model") as f:
                        model = f.read().strip()
                except:
                    model = "Unknown"

                # Get rotational
                try:
                    with open(f"{dev_path}/queue/rotational") as f:
                        rotational = int(f.read().strip()) == 1
                except:
                    rotational = False

                # Determine type
                if device.startswith("nvme"):
                    dev_type = "NVMe"
                elif rotational:
                    dev_type = "HDD"
                else:
                    dev_type = "SSD"

                self.devices[device] = {
                    "size_bytes": size_bytes,
                    "size_gb": size_bytes // (1024**3),
                    "model": model,
                    "type": dev_type,
                    "rotational": rotational
                }
        except Exception as e:
            print(f"Error detecting devices: {e}", file=sys.stderr)

    def run(self) -> int:
        """Main application loop"""
        # Initialize
        if not self.init_notcurses():
            return 1

        # Load system info
        self.load_system_info()

        # If no devices loaded, detect them
        if not self.devices:
            self.detect_devices()

        # Main loop
        try:
            while True:
                if self.current_screen == "welcome":
                    screen = WelcomeScreen(self.nc, self.system_info)
                    result = screen.show()
                    if result == "quit":
                        break
                    self.current_screen = "mode_select"

                elif self.current_screen == "mode_select":
                    screen = ModeSelectionScreen(self.nc)
                    result = screen.show()
                    if result == "back":
                        self.current_screen = "welcome"
                    elif result in ("new", "existing"):
                        self.state["mode"] = result
                        self.current_screen = "device_select"
                    elif result == "quit":
                        break

                elif self.current_screen == "device_select":
                    screen = DeviceSelectionScreen(self.nc, self.devices, self.state["drives"])
                    result = screen.show()
                    if result == "back":
                        self.current_screen = "mode_select"
                    elif result == "quit":
                        break
                    elif isinstance(result, list):
                        self.state["drives"] = result
                        self.current_screen = "settings"

                elif self.current_screen == "settings":
                    screen = SettingsScreen(self.nc, self.state)
                    result = screen.show()
                    if result == "back":
                        self.current_screen = "device_select"
                    elif result == "quit":
                        break
                    elif result == "next":
                        self.current_screen = "validation"

                elif self.current_screen == "validation":
                    screen = ValidationScreen(self.nc, self.state, self.system_info)
                    result = screen.show()
                    if result == "back":
                        self.current_screen = "settings"
                    elif result == "quit":
                        break
                    elif result == "valid":
                        self.current_screen = "confirm"

                elif self.current_screen == "confirm":
                    screen = ConfirmationScreen(self.nc, self.state)
                    result = screen.show()
                    if result == "back":
                        self.current_screen = "validation"
                    elif result == "quit":
                        break
                    elif result == "proceed":
                        self.current_screen = "install"

                elif self.current_screen == "install":
                    screen = InstallationScreen(self.nc, self.state)
                    result = screen.show()
                    if result == "success":
                        self.current_screen = "complete"
                    elif result == "failed":
                        self.current_screen = "settings"
                    elif result == "quit":
                        break

                elif self.current_screen == "complete":
                    screen = CompletionScreen(self.nc, self.state)
                    result = screen.show()
                    break

                else:
                    break

        except KeyboardInterrupt:
            pass
        finally:
            self.cleanup()

        return 0


def signal_handler(sig, frame):
    """Handle Ctrl+C gracefully"""
    sys.exit(0)


def main():
    """Main entry point"""
    signal.signal(signal.SIGINT, signal_handler)

    app = ZBMNotcursesTUI()
    return app.run()


if __name__ == "__main__":
    sys.exit(main())
