#!/bin/bash
################################################################################
# Example: Mirrored (RAID1) ZFSBootMenu installation
#
# This example demonstrates installing ZFSBootMenu on two mirrored drives
# Suitable for: Production systems requiring redundancy
################################################################################

# Run the installation script with mirror configuration
sudo ./zbm_install.sh \
  --mode new \
  --drives sda,sdb \
  --raid mirror \
  --pool zroot \
  --efi-size 1G \
  --swap-size 16G

# Note: Replace 'sda,sdb' with your actual drive identifiers
# Both drives should be of similar size for optimal use
