# Contributing to ZFSBootMenu Installation Script

Thank you for your interest in contributing to this project! This document provides guidelines for contributing.

## Code of Conduct

- Be respectful and inclusive
- Test your changes thoroughly before submitting
- Follow the existing code style and conventions

## Development Setup

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/zbm_install_script_prealpha_garbage.git
   cd zbm_install_script_prealpha_garbage
   ```

3. Create a branch for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   ```

## Code Style

### BASH Script Guidelines

- Use 4 spaces for indentation (no tabs)
- Always use `set -e -u -o pipefail` at the top of scripts
- Quote all variables: `"$variable"` not `$variable`
- Use meaningful function and variable names
- Add comments for complex logic
- Keep functions focused on a single task
- Use `local` for function variables

### Required Tools

Before submitting, ensure your code passes:

```bash
# Shellcheck validation
shellcheck -x zbm_install.sh lib/*.sh examples/*.sh

# Basic syntax check
bash -n zbm_install.sh lib/*.sh
```

## Testing

### Manual Testing

Always test your changes in a safe environment:

1. Use a virtual machine for testing
2. Test with the `--dry-run` flag first
3. Test on multiple distributions if possible:
   - Fedora 42/43
   - Debian 13
   - MX Linux 25

### Test Checklist

- [ ] Script passes shellcheck without errors
- [ ] Dry-run mode works correctly
- [ ] Help text is accurate
- [ ] Error handling works as expected
- [ ] Changes don't break existing functionality
- [ ] Documentation is updated

## Making Changes

### Adding New Features

1. Update the main script or appropriate library module
2. Add appropriate error handling
3. Update documentation in README.md
4. Add example usage if applicable
5. Test thoroughly

### Fixing Bugs

1. Identify the root cause
2. Add a test case if possible
3. Implement the fix
4. Verify the fix doesn't break other functionality
5. Update changelog/documentation

## Submitting Changes

1. Commit your changes with clear, descriptive messages:
   ```bash
   git commit -m "Add support for XYZ feature"
   ```

2. Push to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```

3. Create a Pull Request with:
   - Clear description of changes
   - Why the changes are needed
   - How you tested the changes
   - Any breaking changes or special considerations

## Pull Request Guidelines

### PR Description Should Include

- Summary of changes
- Motivation for changes
- Testing performed
- Screenshots (if UI/output changes)
- Breaking changes (if any)

### PR Checklist

- [ ] Code follows project style guidelines
- [ ] All scripts pass shellcheck
- [ ] Changes have been tested
- [ ] Documentation has been updated
- [ ] Examples have been added/updated (if needed)
- [ ] Commit messages are clear and descriptive

## Project Structure

```
zbm_install_script_prealpha_garbage/
├── zbm_install.sh       # Main script - orchestrates installation
├── lib/
│   ├── common.sh       # Logging, utilities, distribution detection
│   ├── disk.sh         # Disk operations and partitioning
│   ├── zfs.sh          # ZFS pool and dataset management
│   └── bootloader.sh   # Bootloader installation and config
├── examples/           # Usage examples
├── README.md          # User documentation
└── CONTRIBUTING.md    # This file
```

## Adding New Distribution Support

To add support for a new Linux distribution:

1. Update `detect_distribution()` in `lib/common.sh`
2. Add package installation logic in `install_package()` in `lib/common.sh`
3. Add distribution-specific ZFS installation in `lib/zfs.sh`
4. Add ZFSBootMenu package installation in `lib/bootloader.sh`
5. Test thoroughly on the new distribution
6. Update README.md with the new distribution

## Questions?

If you have questions about contributing:

1. Check existing issues and pull requests
2. Review the ZFSBootMenu documentation: https://docs.zfsbootmenu.org/
3. Open a new issue for discussion

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
