#![no_std]
#![feature(allocator_api)]

use core::panic::PanicInfo;

mod input_verifier;

// Static mutable global for our verifier
static mut VERIFIER: Option<input_verifier::InputDeviceVerifier> = None;

// Panic handler for no_std
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// FFI functions to be called from C

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

#[no_mangle]
pub extern "C" fn rust_exit() {
    unsafe {
        VERIFIER = None;
    }
}

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
