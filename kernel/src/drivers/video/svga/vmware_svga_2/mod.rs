use crate::{
    drivers::{
        pci::{
            PCIGeneralDevice, BUS_MASTER_ENABLE, IO_SPACE_ENABLE, MEMORY_SPACE_ENABLE,
            PCI_SUBSYSTEM,
        },
        read_io_port_u32, write_io_port_u32,
    },
    logln,
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

const SVGA_CAP_NONE: u32 = 0x00000000;
const SVGA_CAP_RECT_COPY: u32 = 0x00000002;
const SVGA_CAP_CURSOR: u32 = 0x00000020;
const SVGA_CAP_CUSOR_BYPASS: u32 = 0x00000040;
const SVGA_CAP_CUSOR_BYPASS_2: u32 = 0x00000080;
const SVGA_CAP_8BIT_EMULATION: u32 = 0x00000100;
const SVGA_CAP_ALPHA_CURSOR: u32 = 0x00000200;
const SVGA_CAP_3D: u32 = 0x00004000;
const SVGA_CAP_EXTENDED_FIFO: u32 = 0x00008000;
const SVGA_CAP_MULTIMON: u32 = 0x00010000;
const SVGA_CAP_PITCHLOCK: u32 = 0x00020000;
const SVGA_CAP_IRQMASK: u32 = 0x00040000;
const SVGA_CAP_DISPLAY_TOPOLOGY: u32 = 0x00080000;
const SVGA_CAP_GMR: u32 = 0x00100000;
const SVGA_CAP_TRACES: u32 = 0x00200000;
const SVGA_CAP_GMR2: u32 = 0x00400000;
const SVGA_CAP_SCREEN_OBJECT_2: u32 = 0x00800000;

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

        logln!("SVGA base port: 0x{:04x}", base_port);
        logln!("Framebuffer start: 0x{:04x}", driver.read_framebuffer_start());
        logln!(
            "Framebuffer offset: 0x{:04x}",
            driver.read_framebuffer_offset()
        );
        logln!("Framebuffer size: 0x{:04x}", driver.read_framebuffer_size());
        logln!("FIFO start: 0x{:04x}", driver.read_fifo_start());
        logln!("FIFO size: 0x{:04x}", driver.read_fifo_size());

        logln!(
            "Max dimensions: {}x{}",
            driver.read_max_width(),
            driver.read_max_height()
        );

        logln!(
            "Current dimensions: {}x{}",
            driver.read_width(),
            driver.read_height()
        );

        let fb_start = driver.read_framebuffer_start();
        let fb_width = driver.read_width() as usize;
        let fb_height = driver.read_height() as usize;
        let fb = (fb_start as usize) as *mut u64;

        let color: u64 = 0xff0000ffff0000ff;

        unsafe {
            for r in 0..fb_height {
                for c in 0..(fb_width / 2) {
                    fb.add((r * (fb_width / 2)) + c).write_volatile(color);
                }
            }
        }

        driver.write_width(1920);
        driver.write_height(1080);

        let fb_offset = driver.read_framebuffer_offset();

        logln!("fb_offset: {fb_offset:08x}");

        driver.write_enable(true);

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

    fn write_enable(&mut self, enable: bool) {
        unsafe { self.write_register(SVGA_REG_ENABLE, if enable { 1 } else { 0 }) }
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

    fn write_width(&mut self, value: u32) {
        unsafe { self.write_register(SVGA_REG_WIDTH, value) }
    }

    fn write_height(&mut self, value: u32) {
        unsafe { self.write_register(SVGA_REG_HEIGHT, value) }
    }

    fn read_max_width(&mut self) -> u32 {
        unsafe { self.read_register(SVGA_REG_MAX_WIDTH) }
    }

    fn read_max_height(&mut self) -> u32 {
        unsafe { self.read_register(SVGA_REG_MAX_HEIGHT) }
    }

    fn read_bits_per_pixel(&mut self) -> u32 {
        unsafe { self.read_register(SVGA_REG_BPP) }
    }
}
