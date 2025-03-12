/// InputDeviceVerifier module provides functionality to verify input devices on Linux
/// with a particular focus on touchpad detection and functionality verification.
use core::fmt;
use alloc::vec::Vec;
use alloc::string::{String, ToString};

/// Type of input device
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Touchpad,
    Keyboard,
    Mouse,
    Unknown,
}

/// Input device information
pub struct DeviceInfo {
    pub name: String,
    pub path: String,
    pub device_type: DeviceType,
}

/// Represents a verifier for Linux input devices with focus on touchpad verification.
/// 
/// This struct maintains state about discovered input devices and their functionality,
/// particularly focused on touchpad devices for debugging purposes.
pub struct InputDeviceVerifier {
    touchpad_found: bool,
    touchpad_working: bool,
    touchpad_path: Option<String>,
    touchpad_name: Option<String>,
}

impl InputDeviceVerifier {
    /// Creates a new instance of the InputDeviceVerifier.
    ///
    /// Initializes the verifier with default state (no devices found or verified).
    /// Logs initialization to the kernel log buffer.
    ///
    /// # Returns
    ///
    /// * `Result<Self, ()>` - A new verifier instance wrapped in Ok, or Err if initialization fails
    pub fn new() -> Result<Self, ()> {
        kprint!("Initializing InputDeviceVerifier\n");
        
        Ok(Self {
            touchpad_found: false,
            touchpad_working: false,
            touchpad_path: None,
            touchpad_name: None,
        })
    }
    
    /// Scans the system for input devices with focus on touchpad devices.
    ///
    /// Performs a system scan to detect input devices connected to the system.
    /// Updates the internal state to track discovered devices.
    ///
    /// # Returns
    ///
    /// * `Result<(), ()>` - Ok if the scan completes successfully, Err otherwise
    pub fn scan_devices(&mut self) -> Result<(), ()> {
        kprint!("Scanning for input devices...\n");
        
        let input_devices = self.read_input_devices()?;
        
        match self.identify_touchpad(&input_devices) {
            Ok((found, path, name)) => {
                self.touchpad_found = found;
                self.touchpad_path = path;
                self.touchpad_name = name;
                
                if self.touchpad_found {
                    kprint!("Touchpad device found: {}\n", self.touchpad_name.as_ref().unwrap());
                    kprint!("Touchpad path: {}\n", self.touchpad_path.as_ref().unwrap());
                } else {
                    kprint!("No touchpad device identified\n");
                }
                
                kprint!("Input device scan complete\n");
                Ok(())
            },
            Err(_) => {
                kprint!("Failed to identify touchpad\n");
                Err(())
            }
        }
    }

    /// Reads input devices from sysfs and proc.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<DeviceInfo>, ()>` - Vector of input device info or error
    fn read_input_devices(&self) -> Result<Vec<DeviceInfo>, ()> {
        let mut devices = Vec::new();
        
        // Call the kernel FFI function to get input devices
        let device_entries = self.read_sysfs_directory("/sys/class/input")?;
        
        for entry in device_entries {
            if !entry.starts_with("event") {
                continue;
            }
            
            let device_path = alloc::format!("/dev/input/{}", entry);
            let sys_path = alloc::format!("/sys/class/input/{}", entry);
            
            match self.read_device_name(&sys_path) {
                Ok(name) => {
                    kprint!("Found input device: {} at {}\n", name, device_path);
                    
                    let device_type = if self.is_touchpad_by_name(&name) {
                        DeviceType::Touchpad
                    } else if name.contains("keyboard") || name.contains("Keyboard") {
                        DeviceType::Keyboard
                    } else if name.contains("mouse") || name.contains("Mouse") {
                        DeviceType::Mouse
                    } else {
                        self.determine_device_type(&device_path).unwrap_or(DeviceType::Unknown)
                    };
                    
                    devices.push(DeviceInfo {
                        name,
                        path: device_path,
                        device_type,
                    });
                },
                Err(_) => continue, 
            }
        }
        
        Ok(devices)
    }

    /// Reads the name of an input device from sysfs.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to device directory in sysfs
    ///
    /// # Returns
    ///
    /// * `Result<String, ()>` - Device name or error
    fn read_device_name(&self, path: &str) -> Result<String, ()> {
        let name_path = alloc::format!("{}/device/name", path);
        self.read_file_contents(&name_path)
    }

    /// Determines device type based on capabilities in /proc/bus/input/devices.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to device node
    ///
    /// # Returns
    ///
    /// * `Result<DeviceType, ()>` - Device type or error
    fn determine_device_type(&self, path: &str) -> Result<DeviceType, ()> {
        unsafe {
            extern "C" {
                fn kernel_get_device_capabilities(
                    path: *const u8,
                    path_len: usize,
                    abs_support: *mut u32,     // For EV_ABS support
                    rel_support: *mut u32,     // For EV_REL support
                    key_support: *mut u32      // For EV_KEY support
                ) -> i32;
            }
            
            let path_bytes = path.as_bytes();
            let mut abs_support: u32 = 0;
            let mut rel_support: u32 = 0;
            let mut key_support: u32 = 0;
            
            let result = kernel_get_device_capabilities(
                path_bytes.as_ptr(),
                path_bytes.len(),
                &mut abs_support,
                &mut rel_support,
                &mut key_support
            );
            
            if result < 0 {
                return Err(());
            }
            
            // Check for ABS_MT_POSITION_X (0x35) and ABS_MT_POSITION_Y (0x36) due them absolute
            // positioning
            if (abs_support & (1 << 0x35)) != 0 && (abs_support & (1 << 0x36)) != 0 {
                return Ok(DeviceType::Touchpad);
            }
            
            // Check for REL_X (0x00) and REL_Y (0x01) due the possibility to have a relative positioning
            if (rel_support & (1 << 0x00)) != 0 && (rel_support & (1 << 0x01)) != 0 {
                return Ok(DeviceType::Mouse);
            }
            
            // Keyboard check might involve KEY_A through KEY_Z
            let has_letter_keys = (0x04..=0x1D).any(|key_code| (key_support & (1 << key_code)) != 0);
            if has_letter_keys {
                return Ok(DeviceType::Keyboard);
            }
            
            Ok(DeviceType::Unknown)
        }
    }

    /// Reads sysfs directory entries.
    ///
    /// # Arguments
    ///
    /// * `path` - Directory path
    ///
    /// # Returns
    ///
    /// * `Result<Vec<String>, ()>` - Directory entries or error
    fn read_sysfs_directory(&self, path: &str) -> Result<Vec<String>, ()> {
        unsafe {
            extern "C" {
                fn kernel_read_directory(
                    path: *const u8,
                    path_len: usize,
                    callback: unsafe extern "C" fn(*const u8, usize, *mut Vec<String>) -> i32,
                    output: *mut Vec<String>
                ) -> i32;
            }
            
            unsafe extern "C" fn dir_callback(entry: *const u8, entry_len: usize, output: *mut Vec<String>) -> i32 {
                let entry_slice = core::slice::from_raw_parts(entry, entry_len);
                if let Ok(entry_str) = core::str::from_utf8(entry_slice) {
                    if !entry_str.starts_with(".") {  // Skip hidden files
                        (*output).push(entry_str.to_string());
                    }
                }
                0  
            }
            
            let mut entries = Vec::new();
            let path_bytes = path.as_bytes();
            
            let result = kernel_read_directory(
                path_bytes.as_ptr(),
                path_bytes.len(),
                dir_callback,
                &mut entries as *mut Vec<String>
            );
            
            if result < 0 {
                kprint!("Failed to read directory: {}\n", path);
                return Err(());
            }
            
            Ok(entries)
        }
    }
    
    /// Reads file contents from sysfs or proc.
    ///
    /// # Arguments
    ///
    /// * `path` - File path
    ///
    /// # Returns
    ///
    /// * `Result<String, ()>` - File contents or error
    fn read_file_contents(&self, path: &str) -> Result<String, ()> {
        unsafe {
            extern "C" {
                fn kernel_read_file(
                    path: *const u8,
                    path_len: usize,
                    buffer: *mut u8,
                    buffer_size: usize,
                    bytes_read: *mut usize
                ) -> i32;
            }
            
            let path_bytes = path.as_bytes();
            let mut buffer = alloc::vec![0u8; 256];  
            let mut bytes_read: usize = 0;
            
            let result = kernel_read_file(
                path_bytes.as_ptr(),
                path_bytes.len(),
                buffer.as_mut_ptr(),
                buffer.len(),
                &mut bytes_read
            );
            
            if result < 0 || bytes_read == 0 {
                return Err(());
            }
            
            // Truncate buffer to actual size and remove any trailing whitespace
            buffer.truncate(bytes_read);
            while buffer.last() == Some(&b'\n') || buffer.last() == Some(&b'\r') || buffer.last() == Some(&b' ') {
                buffer.pop();
            }
            
            match String::from_utf8(buffer) {
                Ok(contents) => Ok(contents),
                Err(_) => Err(())
            }
        }
    }

    /// Identifies a touchpad device from a list of input devices.
    ///
    /// # Arguments
    ///
    /// * `devices` - List of device information structures
    ///
    /// # Returns
    ///
    /// * `Result<(bool, Option<String>, Option<String>), ()>` - Tuple with: 
    ///   - found flag
    ///   - optional device path
    ///   - optional device name
    fn identify_touchpad(&self, devices: &[DeviceInfo]) -> Result<(bool, Option<String>, Option<String>), ()> {
        // First check for devices already identified as touchpads
        if let Some(device) = devices.iter().find(|dev| dev.device_type == DeviceType::Touchpad) {
            kprint!("Found explicit touchpad device: {}\n", device.name);
            return Ok((true, Some(device.path.clone()), Some(device.name.clone())));
        }
        
        // If not found by type check for it indicators in name
        for device in devices {
            if self.is_touchpad_by_name(&device.name) {
                kprint!("Identified touchpad by name: {}\n", device.name);
                return Ok((true, Some(device.path.clone()), Some(device.name.clone())));
            }
        }
        
        Ok((false, None, None))
    }
    
    /// Checks if a device is a touchpad based on its name.
    ///
    /// # Arguments
    ///
    /// * `name` - Device name to check
    ///
    /// # Returns
    ///
    /// * `bool` - True if the device name indicates a touchpad
    fn is_touchpad_by_name(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        
        name_lower.contains("touchpad") ||
        name_lower.contains("trackpad") ||
        name_lower.contains("glidepoint") ||
        name_lower.contains("clickpad") ||
        // Specific vendors
        name.contains("ETPS") ||     // Elantech Touchpad
        name.contains("ELAN") ||     // ELAN Touchpad (common in Acer laptops)
        name.contains("04F3") ||     // ELAN Vendor ID
        name.contains("Synaptics") || // Synaptics Touchpad
        name.contains("ALPS") ||     // ALPS Touchpad
        // Specific to Acer Nitro 5
        name.contains("MSFT0001") || // Microsoft Precision Touchpad
        name.contains("1A58:0271")   // Another common Acer Nitro 5 touchpad ID
    }

    /// Verifies if the touchpad is functioning correctly.
    ///
    /// This function checks if a touchpad was found first, then attempts to
    /// verify if it's working correctly by interacting with the device driver.
    ///
    /// # Returns
    ///
    /// * `Result<bool, ()>` - Ok with true if touchpad is working, Ok with false if not working
    ///                       or not found, and Err if verification process fails
    pub fn verify_touchpad(&mut self) -> Result<bool, ()> {
        if !self.touchpad_found {
            kprint!("Touchpad not found, cannot verify\n");
            return Ok(false);
        }
        
        let Some(touchpad_path) = self.touchpad_path.as_ref() else {
            kprint!("Touchpad path not available\n");
            return Ok(false);
        };
        
        kprint!("Verifying touchpad functionality for: {}\n", 
                self.touchpad_name.as_ref().unwrap_or(&"Unknown".to_string()));
        
        // Check if required kernel modules are loaded
        match self.check_touchpad_modules() {
            Ok(true) => kprint!("Touchpad modules are loaded correctly\n"),
            Ok(false) => {
                kprint!("Required touchpad modules not loaded\n");
                self.touchpad_working = false;
                return Ok(false);
            },
            Err(_) => {
                kprint!("Failed to check touchpad modules\n");
                return Err(());
            }
        }
        
        // Verify device node is responsive
        match self.check_device_responsive(touchpad_path) {
            Ok(true) => kprint!("Touchpad device node is responsive\n"),
            Ok(false) => {
                kprint!("Touchpad device node is not responsive\n");
                self.touchpad_working = false;
                return Ok(false);
            },
            Err(_) => {
                kprint!("Failed to check touchpad device node\n");
                return Err(());
            }
        }
        
        // Verify input event capability
        match self.check_input_events(touchpad_path) {
            Ok(true) => {
                kprint!("Touchpad can generate input events\n");
                self.touchpad_working = true;
            },
            Ok(false) => {
                kprint!("Touchpad cannot generate input events\n");
                self.touchpad_working = false;
            },
            Err(_) => {
                kprint!("Failed to check touchpad event generation\n");
                return Err(());
            }
        }

        kprint!("Touchpad verification complete: {}\n", 
               if self.touchpad_working { "working" } else { "not working" });
        
        Ok(self.touchpad_working)
    }
}

/// Kernel print macro that calls into C-based kernel logging functions.
///
/// This macro allows Rust code to interface with the kernel's printing facilities,
/// which is necessary since we can't use std::println! in a kernel context.
///
/// # Parameters
///
/// * Format string and arguments similar to standard Rust format! macro
///
/// # Examples
///
/// ```
/// kprint!("Hello from Rust kernel module\n");
/// kprint!("Value: {}\n", some_value);
/// ```
#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => ({
        extern "C" {
            fn kernel_print(msg: *const u8, len: usize);
        }
        
        let msg = alloc::format!($($arg)*);
        let bytes = msg.as_bytes();
        unsafe {
            kernel_print(bytes.as_ptr(), bytes.len());
        }
    });
}
