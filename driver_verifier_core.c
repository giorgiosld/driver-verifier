#include <linux/init.h>
#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/device.h>
#include <linux/input.h>

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Giorgio Saldana");
MODULE_DESCRIPTION("A kernel module to verify driver functionality");

// Function declarations for Rust code
extern int rust_init(void);
extern void rust_exit(void);
extern int rust_scan_devices(void);
extern int rust_verify_touchpad(void);

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

static void __exit driver_verifier_exit(void)
{
    printk(KERN_INFO "Driver Verifier: cleaning up\n");
    rust_exit();
    printk(KERN_INFO "Driver Verifier: Module unloaded\n");
}

module_init(driver_verifier_init);
module_exit(driver_verifier_exit);
