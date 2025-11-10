#!/bin/bash
################################################################################
# Example: Dry run testing
#
# This example demonstrates using dry-run mode to see what would happen
# without making any actual changes
################################################################################

# Run the installation script in dry-run mode
sudo ./zbm_install.sh \
  --mode new \
  --drives sda,sdb \
  --raid mirror \
  --pool zroot \
  --dry-run

# This will:
# - Show all commands that would be executed
# - Validate the configuration
# - Not make any actual changes to the system
