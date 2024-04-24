use crate::{logln, hlt_loop};
use common::BootInfo;
use core::{fmt::Write, panic::PanicInfo};

#[no_mangle]
pub unsafe extern "C" fn _start(boot_info: *const BootInfo) -> ! {
    crate::initialize_platform();

    let boot_info = boot_info.as_ref().unwrap();

    crate::initialize_kernel(boot_info);

    crate::start_kernel();

    hlt_loop();
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    logln!("ERROR: {}", info);
    hlt_loop();
}
