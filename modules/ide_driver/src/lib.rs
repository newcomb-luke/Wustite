#![no_std]
#![no_main]

use core::panic::PanicInfo;

use modules_common::_log_str;

#[no_mangle]
pub fn _start() {
    let _funcs: &[*const extern "C" fn()] = &[module_init as _];
}

#[no_mangle]
pub unsafe fn module_init() {
    _log_str("Hello!\n");
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
