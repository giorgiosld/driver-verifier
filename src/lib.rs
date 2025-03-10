//! Kernel driver module for verifying input devices on Linux
//!
//! This crate provides a Linux kernel module written in Rust that verifies
//! the functionality of input devices, with particular focus on touchpad devices.
//! It serves as both a diagnostic tool and a reference implementation for
//! Rust-based Linux kernel drivers.
#![no_std]
#![feature(allocator_api)]

use core::panic::PanicInfo;

mod input_verifier;

/// Static mutable global instance for our verifier component.
///
/// This global state is necessary for the FFI functions to interact with
/// our Rust-based verifier from C kernel code.
static mut VERIFIER: Option<input_verifier::InputDeviceVerifier> = None;

// Panic handler for no_std
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// FFI functions to be called from C
/// Initializes the Rust module components of the kernel driver.
///
/// This function creates a new InputDeviceVerifier instance and stores it
/// in the global VERIFIER state for later use by other FFI functions.
///
/// # Safety
///
/// This function is unsafe because it modifies global state and is called
/// from C code.
///
/// # Returns
///
/// * `i32` - 0 on success, -1 on error
#[no_mangle]
pub extern "C" fn rust_init() -> i32 {
    match input_verifier::InputDeviceVerifier::new() {
        Ok(verifier) => {
            unsafe {
                VERIFIER = Some(verifier);
            }
            0 
        }
        Err(_) => -1, 
    }
}

/// Cleans up the Rust module components during kernel driver unload.
///
/// This function releases the InputDeviceVerifier instance stored in
/// the global VERIFIER state.
///
/// # Safety
///
/// This function is unsafe because it modifies global state and is called
/// from C code.
#[no_mangle]
pub extern "C" fn rust_exit() {
    unsafe {
        VERIFIER = None;
    }
}

/// Triggers a scan for input devices using the Rust verifier.
///
/// This function calls the scan_devices method on the global VERIFIER
/// instance if it exists.
///
/// # Safety
///
/// This function is unsafe because it accesses global state and is called
/// from C code.
///
/// # Returns
///
/// * `i32` - 0 on success, -1 if VERIFIER is None or scan fails
#[no_mangle]
pub extern "C" fn rust_scan_devices() -> i32 {
    unsafe {
        if let Some(ref mut verifier) = VERIFIER {
            match verifier.scan_devices() {
                Ok(_) => 0, 
                Err(_) => -1, 
            }
        } else {
            -1 
        }
    }
}

/// Verifies touchpad functionality using the Rust verifier.
///
/// This function calls the verify_touchpad method on the global VERIFIER
/// instance if it exists.
///
/// # Safety
///
/// This function is unsafe because it accesses global state and is called
/// from C code.
///
/// # Returns
///
/// * `i32` - 1 if touchpad is working, 0 if not working, -1 on error or if VERIFIER is None
#[no_mangle]
pub extern "C" fn rust_verify_touchpad() -> i32 {
    unsafe {
        if let Some(ref mut verifier) = VERIFIER {
            match verifier.verify_touchpad() {
                Ok(working) => if working { 1 } else { 0 },
                Err(_) => -1, 
            }
        } else {
            -1 
        }
    }
}
