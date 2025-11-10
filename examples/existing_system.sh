#!/bin/bash
################################################################################
# Example: Existing system ZFSBootMenu installation
#
# This example demonstrates installing ZFSBootMenu on an existing Linux system
# Suitable for: Converting existing installations to ZFS
################################################################################

# IMPORTANT: Before running this, ensure you have:
# 1. Backed up all important data
# 2. Identified the target drive(s)
# 3. Understand that this will repartition the drives

# Run the installation script in existing mode
sudo ./zbm_install.sh \
  --mode existing \
  --drives sda \
  --pool zroot \
  --efi-size 1G \
  --swap-size 8G

# Note: This mode assumes you want to convert an existing system
# The script will handle the migration, but manual steps may be required
