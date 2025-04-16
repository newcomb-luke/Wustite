use crate::{
    acpi::acpi_request_irq,
    drivers::{DriverResult, write_io_port_u8},
    interrupts::{GSI, IrqResult, LogicalIrq},
    log,
    resource::{request_irq, request_port},
    state::increment_system_clock,
};

const CHANNEL0_DATA_PORT: u16 = 0x40;
const COMMAND_REGISTER_PORT: u16 = 0x43;

// The target frequency here is 500 Hz
// Technically 1193182 / 500 = 2386.364
const TIMER_DIVIDER: u16 = 2386;
const SYSTEM_CLOCK_INCREMENT_NANOS: u64 = 2000000;

pub static LEGACY_TIMER_DRIVER: LegacyTimer = LegacyTimer::new();

pub struct LegacyTimer {}

impl LegacyTimer {
    pub const fn new() -> Self {
        Self {}
    }

    pub fn initialize(&self) -> DriverResult {
        request_port(COMMAND_REGISTER_PORT).map_err(|_| ())?;
        request_port(CHANNEL0_DATA_PORT).map_err(|_| ())?;

        acpi_request_irq(
            GSI::from_u8(0),
            &LEGACY_TIMER_DRIVER,
            Self::handle_interrupt,
        )?;

        // This just works, which makes it easy
        //      Channel = 0
        //      Latch count value command = 0
        //      Mode 0 (interrupt on terminal count) = 0
        //      16-bit binary counter = 0
        unsafe {
            write_io_port_u8(COMMAND_REGISTER_PORT, 0);
        }

        unsafe {
            Self::set_pit_count(TIMER_DIVIDER);
        }

        Ok(())
    }

    pub extern "C" fn handle_interrupt(&'static self, _irq: LogicalIrq) -> IrqResult {
        increment_system_clock(SYSTEM_CLOCK_INCREMENT_NANOS);
        IrqResult::Handled
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
}
