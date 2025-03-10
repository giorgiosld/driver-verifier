/**
 * driver_verifier_core.c - C entry point for the hybrid C/Rust kernel driver
 *
 * This file serves as the main entry point for the kernel module, handling
 * initialization, cleanup, and coordination with the Rust components.
 * It implements the standard Linux kernel module structure while delegating
 * the core verification functionality to the Rust implementation.
 */
#include <linux/init.h>
#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/device.h>
#include <linux/input.h>

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Giorgio Saldana");
MODULE_DESCRIPTION("A kernel module to verify driver functionality");

/**
 * External function declarations for Rust code
 *
 * These functions are implemented in the Rust part of the module and
 * compiled into a static library that gets linked with this C code.
 */
extern int rust_init(void);
extern void rust_exit(void);
extern int rust_scan_devices(void);
extern int rust_verify_touchpad(void);

/**
 * driver_verifier_init - Module initialization function
 *
 * Called when the module is loaded into the kernel. This function
 * initializes the Rust component, triggers device scanning, and
 * verifies touchpad functionality.
 *
 * Return: 0 on success, negative error code on failure
 */
static int __init driver_verifier_init(void)
{
    printk(KERN_INFO "Driver Verifier: initializing\n");
    
    int result = rust_init();
    if (result != 0) {
        printk(KERN_ERR "Driver Verifier: Failed to initialize Rust component\n");
        return -EINVAL;
    }
    
    rust_scan_devices();
    
    int touchpad_status = rust_verify_touchpad();
    printk(KERN_INFO "Driver Verifier: Touchpad status: %s\n", 
           touchpad_status ? "working" : "not working or not found");
    
    printk(KERN_INFO "Driver Verifier: Module loaded successfully\n");
    return 0;
}

/**
 * driver_verifier_exit - Module cleanup function
 *
 * Called when the module is unloaded from the kernel. This function
 * performs cleanup by calling into the Rust exit function.
 */
static void __exit driver_verifier_exit(void)
{
    printk(KERN_INFO "Driver Verifier: cleaning up\n");
    rust_exit();
    printk(KERN_INFO "Driver Verifier: Module unloaded\n");
}

module_init(driver_verifier_init);
module_exit(driver_verifier_exit);
