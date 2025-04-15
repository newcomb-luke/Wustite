use crate::{drivers::write_io_port_u8, logln};

const CHANNEL0_DATA_PORT: u16 = 0x40;
const COMMAND_REGISTER_PORT: u16 = 0x43;

// The target frequency here is 500 Hz
// Technically 1193182 / 500 = 2386.364
const TIMER_DIVIDER: u16 = 2386;

/// SAFETY: The caller must guarantee that the default ports are the real ports
pub unsafe fn initialize_legacy_timer() {
    // This just works, which makes it easy
    //      Channel = 0
    //      Latch count value command = 0
    //      Mode 0 (interrupt on terminal count) = 0
    //      16-bit binary counter = 0
    unsafe {
        write_io_port_u8(COMMAND_REGISTER_PORT, 0);
    }

    unsafe {
        set_pit_count(TIMER_DIVIDER);
    }
}

unsafe fn set_pit_count(count: u16) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let low = (count & 0xFF) as u8;
        let high = ((count >> 8) & 0xFF) as u8;

        unsafe {
            write_io_port_u8(CHANNEL0_DATA_PORT, low);
            write_io_port_u8(CHANNEL0_DATA_PORT, high);
        }
    });
}
