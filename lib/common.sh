#!/bin/bash
################################################################################
# Common utility functions for ZFSBootMenu installation
################################################################################

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

################################################################################
# Logging functions
################################################################################

log_info() {
    echo -e "${BLUE}[INFO]${NC} $*" | tee -a "$LOG_FILE"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*" | tee -a "$LOG_FILE"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*" | tee -a "$LOG_FILE"
}

log_step() {
    echo -e "\n${GREEN}=== $* ===${NC}\n" | tee -a "$LOG_FILE"
}

################################################################################
# Distribution detection
################################################################################

detect_distribution() {
    if [[ -f /etc/os-release ]]; then
        # shellcheck source=/dev/null
        source /etc/os-release
        DETECTED_DISTRO="$ID"
        DETECTED_VERSION="${VERSION_ID:-unknown}"
        
        log_info "Detected distribution: $DETECTED_DISTRO $DETECTED_VERSION"
        
        # Validate supported distributions
        case "$DETECTED_DISTRO" in
            fedora)
                if [[ ! "$DETECTED_VERSION" =~ ^(42|43)$ ]]; then
                    log_warn "Fedora version $DETECTED_VERSION may not be fully tested (tested: 42, 43)"
                fi
                ;;
            debian)
                if [[ "$DETECTED_VERSION" != "13" ]]; then
                    log_warn "Debian version $DETECTED_VERSION may not be fully tested (tested: 13)"
                fi
                ;;
            mx)
                if [[ ! "$DETECTED_VERSION" =~ ^25 ]]; then
                    log_warn "MX Linux version $DETECTED_VERSION may not be fully tested (tested: 25)"
                fi
                ;;
            *)
                log_warn "Distribution $DETECTED_DISTRO is not explicitly supported, proceeding anyway"
                ;;
        esac
    else
        log_error "Cannot detect distribution: /etc/os-release not found"
        exit 1
    fi
}

################################################################################
# Execute command with dry-run support
################################################################################

execute_cmd() {
    local cmd="$*"
    log_info "Executing: $cmd"
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would execute: $cmd"
        return 0
    else
        if eval "$cmd" 2>&1 | tee -a "$LOG_FILE"; then
            return 0
        else
            local exit_code=$?
            log_error "Command failed with exit code $exit_code: $cmd"
            return $exit_code
        fi
    fi
}

################################################################################
# Check if package is installed
################################################################################

is_package_installed() {
    local package="$1"
    
    case "$DETECTED_DISTRO" in
        fedora)
            rpm -q "$package" &>/dev/null
            ;;
        debian|mx)
            dpkg -l "$package" 2>/dev/null | grep -q "^ii"
            ;;
        *)
            log_warn "Unknown package manager for distribution: $DETECTED_DISTRO"
            return 1
            ;;
    esac
}

################################################################################
# Install package
################################################################################

install_package() {
    local package="$1"
    
    log_info "Installing package: $package"
    
    if is_package_installed "$package"; then
        log_info "Package $package is already installed"
        return 0
    fi
    
    case "$DETECTED_DISTRO" in
        fedora)
            execute_cmd "dnf install -y $package"
            ;;
        debian|mx)
            execute_cmd "apt-get update && apt-get install -y $package"
            ;;
        *)
            log_error "Unknown package manager for distribution: $DETECTED_DISTRO"
            return 1
            ;;
    esac
}

################################################################################
# Check if command exists
################################################################################

command_exists() {
    command -v "$1" &>/dev/null
}

################################################################################
# Wait for user confirmation
################################################################################

confirm() {
    local message="$1"
    
    if [[ "$FORCE" == "true" ]]; then
        return 0
    fi
    
    read -p "$message (yes/no): " -r
    if [[ $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
        return 0
    else
        return 1
    fi
}

################################################################################
# Convert size to bytes
################################################################################

size_to_bytes() {
    local size="$1"
    local number="${size//[^0-9]/}"
    local unit="${size//[0-9]/}"
    
    case "${unit^^}" in
        K|KB)
            echo $((number * 1024))
            ;;
        M|MB)
            echo $((number * 1024 * 1024))
            ;;
        G|GB)
            echo $((number * 1024 * 1024 * 1024))
            ;;
        T|TB)
            echo $((number * 1024 * 1024 * 1024 * 1024))
            ;;
        *)
            echo "$number"
            ;;
    esac
}

################################################################################
# Check if module is loaded
################################################################################

is_module_loaded() {
    local module="$1"
    lsmod | grep -q "^$module"
}

################################################################################
# Load kernel module
################################################################################

load_module() {
    local module="$1"
    
    if is_module_loaded "$module"; then
        log_info "Module $module is already loaded"
        return 0
    fi
    
    log_info "Loading kernel module: $module"
    execute_cmd "modprobe $module"
}

################################################################################
# Cleanup on error
################################################################################

cleanup_on_error() {
    log_error "An error occurred, performing cleanup..."
    
    # Export pool if it exists
    if zpool list "$POOL_NAME" &>/dev/null; then
        log_info "Exporting ZFS pool: $POOL_NAME"
        zpool export "$POOL_NAME" 2>/dev/null || true
    fi
    
    # Unmount any mounted filesystems
    for drive in "${DRIVES[@]}"; do
        umount "/dev/${drive}"* 2>/dev/null || true
    done
}

# Set up error trap
trap cleanup_on_error ERR
