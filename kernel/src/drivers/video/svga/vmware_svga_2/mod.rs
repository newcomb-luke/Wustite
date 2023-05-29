use crate::{
    drivers::{
        pci::{
            PCIGeneralDevice, BUS_MASTER_ENABLE, IO_SPACE_ENABLE, MEMORY_SPACE_ENABLE,
            PCI_SUBSYSTEM,
        },
        read_io_port_u32, write_io_port_u32,
    },
    println,
};

const SVGA_INDEX_OFFSET: u16 = 0;
const SVGA_VALUE_OFFSET: u16 = 1;
const SVGA_BIOS_OFFSET: u16 = 2;
const SVGA_IRQSTATUS_OFFSET: u16 = 3;

const SVGA_REG_ID: u32 = 0;
const SVGA_REG_ENABLE: u32 = 1;
const SVGA_REG_WIDTH: u32 = 2;
const SVGA_REG_HEIGHT: u32 = 3;
const SVGA_REG_MAX_WIDTH: u32 = 4;
const SVGA_REG_MAX_HEIGHT: u32 = 5;
const SVGA_REG_BPP: u32 = 7;
const SVGA_REG_FB_START: u32 = 13;
const SVGA_REG_FB_OFFSET: u32 = 14;
const SVGA_REG_VRAM_SIZE: u32 = 15;
const SVGA_REG_FB_SIZE: u32 = 16;
const SVGA_REG_CAPABILITIES: u32 = 17;
const SVGA_REG_FIFO_START: u32 = 18;
const SVGA_REG_FIFO_SIZE: u32 = 19;
const SVGA_REG_CONFIG_DONE: u32 = 20;
const SVGA_REG_SYNC: u32 = 21;
const SVGA_REG_BUSY: u32 = 22;

const DRIVER_SPEC_ID: u32 = 0x90000002;

#[derive(Debug, Clone, Copy)]
pub enum DriverInitError {
    SpecUnsupportedError,
}

pub struct VMWareSVGADriver {
    device: PCIGeneralDevice,
    base_port: u16,
}

impl VMWareSVGADriver {
    pub fn new(mut device: PCIGeneralDevice) -> Result<Self, DriverInitError> {
        PCI_SUBSYSTEM.send_command(
            &mut device,
            BUS_MASTER_ENABLE | MEMORY_SPACE_ENABLE | IO_SPACE_ENABLE,
        );

        let base_port = (device.bar0() - 1) as u16;

        let mut driver = Self { device, base_port };

        driver.write_spec_id_register(DRIVER_SPEC_ID);

        if driver.read_spec_id_register() != DRIVER_SPEC_ID {
            return Err(DriverInitError::SpecUnsupportedError);
        }

        println!("SVGA base port: {:04x}", base_port);
        println!("Framebuffer start: {:04x}", driver.read_framebuffer_start());
        println!(
            "Framebuffer offset: {:04x}",
            driver.read_framebuffer_offset()
        );
        println!("Framebuffer size: {:04x}", driver.read_framebuffer_size());
        println!("FIFO start: {:04x}", driver.read_fifo_start());
        println!("FIFO size: {:04x}", driver.read_fifo_size());

        println!(
            "Max dimensions: {}x{}",
            driver.read_max_width(),
            driver.read_max_height()
        );

        println!(
            "Current dimensions: {}x{}",
            driver.read_width(),
            driver.read_height()
        );

        Ok(driver)
    }

    #[inline]
    unsafe fn write_register(&mut self, register: u32, value: u32) {
        write_io_port_u32(self.base_port + SVGA_INDEX_OFFSET, register);
        write_io_port_u32(self.base_port + SVGA_VALUE_OFFSET, value);
    }

    #[inline]
    unsafe fn read_register(&mut self, register: u32) -> u32 {
        write_io_port_u32(self.base_port + SVGA_INDEX_OFFSET, register);
        read_io_port_u32(self.base_port + SVGA_VALUE_OFFSET)
    }

    fn write_spec_id_register(&mut self, value: u32) {
        unsafe { self.write_register(SVGA_REG_ID, value) }
    }

    fn read_spec_id_register(&mut self) -> u32 {
        unsafe { self.read_register(SVGA_REG_ID) }
    }

    fn read_framebuffer_start(&mut self) -> u32 {
        unsafe { self.read_register(SVGA_REG_FB_START) }
    }

    fn read_framebuffer_offset(&mut self) -> u32 {
        unsafe { self.read_register(SVGA_REG_FB_OFFSET) }
    }

    fn read_framebuffer_size(&mut self) -> u32 {
        unsafe { self.read_register(SVGA_REG_FB_SIZE) }
    }

    fn read_fifo_start(&mut self) -> u32 {
        unsafe { self.read_register(SVGA_REG_FIFO_START) }
    }

    fn read_fifo_size(&mut self) -> u32 {
        unsafe { self.read_register(SVGA_REG_FIFO_SIZE) }
    }

    fn read_width(&mut self) -> u32 {
        unsafe { self.read_register(SVGA_REG_WIDTH) }
    }

    fn read_height(&mut self) -> u32 {
        unsafe { self.read_register(SVGA_REG_HEIGHT) }
    }

    fn read_max_width(&mut self) -> u32 {
        unsafe { self.read_register(SVGA_REG_MAX_WIDTH) }
    }

    fn read_max_height(&mut self) -> u32 {
        unsafe { self.read_register(SVGA_REG_MAX_HEIGHT) }
    }
}
