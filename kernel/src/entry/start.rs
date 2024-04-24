use crate::{eprintln, hlt_loop};
use common::BootInfo;
use core::{fmt::Write, panic::PanicInfo};
use crate::SERIAL0;

#[no_mangle]
pub unsafe extern "C" fn _start(boot_info: BootInfo) -> ! {
    {
        let mut serial = SERIAL0.lock();
        serial.initialize();
        serial.write_str("Hello from kernel land!");
    }

    loop {}

    crate::initialize_platform();

    crate::initialize_kernel(&boot_info);

    crate::start_kernel();

    hlt_loop();
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    eprintln!("{}", info);
    hlt_loop();
}
