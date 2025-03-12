# Rust Kernel Driver Verifier
A Rust-based Linux kernel driver for diagnosing and verifying hardware driver functionality, with a focus on input devices like touchpads. This project aims to provide low-level debugging capabilities for troubleshooting driver issues in Linux systems.

## Overview

The Driver Verifier is designed to detect and diagnose driver-related issues on Linux systems, particularly for input devices like touchpads on laptops. It runs as a kernel module, interfacing directly with hardware to verify proper functionality of device drivers.

### Key Features

- **Device Discovery**: Automatically enumerates and identifies input devices through sysfs
- **Touchpad Verification**: Checks if touchpad drivers are loaded and functioning correctly
- **Input Event Monitoring**: Verifies if the device can generate proper input events
- **Hybrid C/Rust Architecture**: Combines the safety of Rust with C's kernel integration capabilities
- **Diagnostic Outputs**: Provides detailed kernel log information for debugging

### Target Hardware

Specifically optimized for:
- Acer Nitro 5 laptops (with ELAN/Synaptics touchpads)
- Parrot OS 6.3 (Linux kernel 6.11+

## Project Structure

- `src/` - Rust source code
  - `lib.rs` - Main Rust entry point with FFI exports
  - `input_verifier.rs` - Core verification logic for input devices
- `driver_verifier_core.c` - C wrapper for kernel module integration
- `Kbuild` - Kernel build configuration
- `Makefile` - Build orchestration

## Technical Details

The module operates by:

1. Enumerating input devices from `/sys/class/input/`
2. Identifying touchpad devices using multiple detection methods:
   - Device name matching (ELAN, Synaptics, etc.)
   - Capability-based detection (multi-touch support)
   - Vendor-specific identifiers
3. Verifying driver functionality:
   - Checking required kernel modules are loaded
   - Testing device node responsiveness
   - Monitoring input event generation)

## Known Limitations

- The module requires specific kernel headers to compile
- Capability detection requires appropriate permissions
- Some hardware-specific optimizations may not work on all devices
- The module does not modify or fix driver issues, it only reports them

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## Author

- Giorgio Saldana ([@giorgiosld](https://github.com/giorgiosld))
