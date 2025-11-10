#!/bin/bash
################################################################################
# Example: RAIDZ1 ZFSBootMenu installation
#
# This example demonstrates installing ZFSBootMenu on three+ drives with RAIDZ1
# Suitable for: Large storage with single drive redundancy
################################################################################

# Run the installation script with RAIDZ1 configuration
sudo ./zbm_install.sh \
  --mode new \
  --drives sda,sdb,sdc \
  --raid raidz1 \
  --pool zroot \
  --efi-size 1G \
  --swap-size 16G

# Note: Replace 'sda,sdb,sdc' with your actual drive identifiers
# RAIDZ1 requires at least 3 drives
# Can tolerate loss of 1 drive
