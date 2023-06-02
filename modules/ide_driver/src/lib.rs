#![no_std]
#![no_main]

use core::panic::PanicInfo;

use modules_common::_log_str;

#[no_mangle]
pub fn _start() {
    let funcs: &[*const extern "C" fn()] = &[module_init as _];
    core::mem::forget(core::hint::black_box(funcs));
}

#[no_mangle]
pub unsafe fn module_init() {
    _log_str("Hello!\n");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}
