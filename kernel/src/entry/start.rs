use crate::logln;
use common::BootInfo;
use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn _start(boot_info: *const BootInfo) -> ! {
    initialize_platform();

    let boot_info = unsafe { boot_info.as_ref().unwrap() };

    crate::start_kernel(boot_info);

    kernel::hlt_loop();
}

fn initialize_platform() {
    crate::gdt::init();
    crate::interrupts::init();
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    logln!("ERROR: {}", info);
    kernel::hlt_loop();
}
