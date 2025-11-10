#!/bin/bash
################################################################################
# Example: Single drive ZFSBootMenu installation
#
# This example demonstrates installing ZFSBootMenu on a single drive
# Suitable for: Simple installations, VMs, testing
################################################################################

# Run the installation script with single drive configuration
sudo ./zbm_install.sh \
  --mode new \
  --drives sda \
  --pool zroot \
  --efi-size 1G \
  --swap-size 8G

# Note: Replace 'sda' with your actual drive identifier
# Use 'lsblk' to list available drives
