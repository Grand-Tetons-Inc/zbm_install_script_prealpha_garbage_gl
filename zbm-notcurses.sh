#!/bin/bash
################################################################################
# ZFSBootMenu Notcurses TUI Launcher
#
# Launches the Python-based notcurses UI and interfaces with bash backend
################################################################################

set -euo pipefail

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Paths
PYTHON_UI="${SCRIPT_DIR}/tui/python/notcurses_ui.py"
CLI_BACKEND="${SCRIPT_DIR}/zbm_install.sh"

# Source state management
source "${SCRIPT_DIR}/tui/lib/tui_state.sh"

################################################################################
# Check dependencies
################################################################################
check_dependencies() {
    local missing=()

    # Check Python 3
    if ! command -v python3 &>/dev/null; then
        missing+=("python3")
    fi

    # Check notcurses Python module
    if ! python3 -c "import notcurses" 2>/dev/null; then
        echo "ERROR: python-notcurses not installed"
        echo ""
        echo "Install notcurses:"
        echo "  Debian/Ubuntu:"
        echo "    sudo apt-get install libnotcurses-dev notcurses-bin"
        echo "    pip3 install notcurses"
        echo ""
        echo "  Fedora:"
        echo "    sudo dnf install notcurses-devel notcurses"
        echo "    pip3 install notcurses"
        echo ""
        echo "Fallback: Use dialog-based TUI instead:"
        echo "  sudo ./zbm-tui.sh"
        exit 1
    fi

    # Check backend exists
    if [[ ! -f "$CLI_BACKEND" ]]; then
        echo "ERROR: Backend CLI not found: $CLI_BACKEND"
        exit 1
    fi

    if [[ ${#missing[@]} -gt 0 ]]; then
        echo "ERROR: Missing required dependencies:"
        for dep in "${missing[@]}"; do
            echo "  - $dep"
        done
        exit 1
    fi
}

################################################################################
# Check if running as root
################################################################################
check_root() {
    if [[ $EUID -ne 0 ]]; then
        echo "ERROR: This TUI must be run as root"
        echo "Please run: sudo $0"
        exit 1
    fi
}

################################################################################
# Prepare system information for Python UI
################################################################################
prepare_system_info() {
    # Initialize TUI state (from tui/lib/tui_state.sh)
    init_tui_state

    # Build JSON with system info and devices
    local json_output
    json_output=$(cat <<EOF
{
  "system_info": {
    "is_efi": $([ "${SYSTEM_INFO[is_efi]}" == "yes" ] && echo "true" || echo "false"),
    "ram_gb": ${SYSTEM_INFO[ram_gb]},
    "cpu_count": ${SYSTEM_INFO[cpu_count]},
    "distro": "${SYSTEM_INFO[distro]}",
    "distro_version": "${SYSTEM_INFO[distro_version]}"
  },
  "devices": {
EOF
)

    # Add device information
    local first_device=true
    for dev in $(get_device_list); do
        if [[ "$first_device" == "false" ]]; then
            json_output+=","
        fi
        first_device=false

        local size="${BLOCK_DEVICES[${dev}:size]}"
        local model="${BLOCK_DEVICES[${dev}:model]}"
        local rotational="${BLOCK_DEVICES[${dev}:rotational]}"

        json_output+=$(cat <<EOF

    "$dev": {
      "size_bytes": $size,
      "model": "$model",
      "rotational": $([ "$rotational" == "1" ] && echo "true" || echo "false")
    }
EOF
)
    done

    json_output+=$(cat <<EOF

  }
}
EOF
)

    echo "$json_output"
}

################################################################################
# Launch Python notcurses UI
################################################################################
launch_ui() {
    echo "Launching Notcurses TUI..."

    # Prepare system info
    local system_json
    system_json=$(prepare_system_info)

    # Launch Python UI, passing system info
    echo "$system_json" | python3 "$PYTHON_UI"

    return $?
}

################################################################################
# Main
################################################################################
main() {
    # Check dependencies and permissions
    check_dependencies
    check_root

    # Launch UI
    launch_ui

    echo ""
    echo "Thank you for using ZFSBootMenu Notcurses TUI!"
}

# Run main
main "$@"
