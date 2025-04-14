use crate::drivers::write_io_port_u8;

const PIC_1_ADDRESS: u16 = 0x20;
const PIC_2_ADDRESS: u16 = 0xA0;
const PIC_COMMAND_OFFSET: u16 = 0x00;
const PIC_DATA_OFFSET: u16 = 0x01;

const ICW1_ICW4: u8 = 0x01;
const ICW1_INIT: u8 = 0x10;
const ICW4_8086: u8 = 0x01;

const PIC_1_OFFSET: u8 = 32;
const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub fn initialize_legacy_pics() {
    unsafe {
        // Start the initialization sequence in cascade mode
        write_legacy_pic_command(PIC_1_ADDRESS, ICW1_INIT | ICW1_ICW4);
        write_legacy_pic_command(PIC_2_ADDRESS, ICW1_INIT | ICW1_ICW4);
        // Master PIC vector offset
        write_legacy_pic_data(PIC_1_ADDRESS, PIC_1_OFFSET);
        // Slave PIC vector offset
        write_legacy_pic_data(PIC_2_ADDRESS, PIC_2_OFFSET);
        // Tell master PIC that there is a slave PIC at IRQ2
        write_legacy_pic_data(PIC_1_ADDRESS, 0x04);
        // Tell slave PIC its cascade identity
        write_legacy_pic_data(PIC_1_ADDRESS, 0x02);

        // Have the PICS use 8086 mode (as opposed to 8080 mode)
        write_legacy_pic_data(PIC_1_ADDRESS, ICW4_8086);
        write_legacy_pic_data(PIC_2_ADDRESS, ICW4_8086);

        // Now disable them by masking all interrupts
        write_legacy_pic_data(PIC_1_ADDRESS, 0xFF);
        write_legacy_pic_data(PIC_2_ADDRESS, 0xFF);
    }
}

unsafe fn write_legacy_pic_command(address: u16, value: u8) {
    unsafe {
        write_io_port_u8(address + PIC_COMMAND_OFFSET, value);
    }
}

unsafe fn write_legacy_pic_data(address: u16, value: u8) {
    unsafe {
        write_io_port_u8(address + PIC_DATA_OFFSET, value);
    }
}
