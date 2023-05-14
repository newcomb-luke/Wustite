use core::arch::asm;

pub const PRESENT: u8 = 0b10000000;
pub const RING0: u8 = 0;
const CODE: u8 = 0b00011000;
const DATA: u8 = 0b00010000;
const READABLE: u8 = 0b00000010;
const WRITABLE: u8 = 0b00000010;
const CONFORMING: u8 = 0b00000100;

const GRANULARITY_4KB: u8 = 0b10000000;
const BITS_32: u8 = 0b01000000;
const BITS_64: u8 = 0b00100000;

#[repr(C, packed)]
struct Descriptor {
    // limit (bits 0-15)
    limit_0_15: u16,
    // base (bits 0-15)
    base_0_15: u16,
    // base (bits 16-23)
    base_16_23: u8,
    flags1: u8,
    // limit (bits 16-19)
    flags_and_limit: u8,
    // base (bits 24-31)
    base_24_31: u8,
}

impl Descriptor {
    const fn null() -> Self {
        Self {
            limit_0_15: 0,
            base_0_15: 0,
            base_16_23: 0,
            flags1: 0,
            flags_and_limit: 0,
            base_24_31: 0,
        }
    }

    const fn code(base: u32, limit: u32, size_flags: u8) -> Self {
        Self {
            limit_0_15: limit as u16,
            base_0_15: base as u16,
            base_16_23: (base >> 16) as u8,
            flags1: PRESENT | RING0 | CODE | READABLE,
            flags_and_limit: size_flags | (limit >> 16) as u8,
            base_24_31: (base >> 24) as u8,
        }
    }

    const fn data(base: u32, limit: u32, size_flags: u8) -> Self {
        Self {
            limit_0_15: limit as u16,
            base_0_15: base as u16,
            base_16_23: (base >> 16) as u8,
            flags1: PRESENT | RING0 | DATA | WRITABLE,
            flags_and_limit: size_flags | (limit >> 16) as u8,
            base_24_31: (base >> 24) as u8,
        }
    }
}

#[repr(C, packed)]
pub struct GlobalDescriptorTable {
    descriptors: [Descriptor; 3],
}

#[repr(C, packed)]
struct GDTDescriptor {
    // Size in bytes - 1
    size: u16,
    pointer: *const GlobalDescriptorTable,
}

impl GlobalDescriptorTable {
    pub const fn unreal() -> Self {
        let descriptors = [
            Descriptor::null(),
            Descriptor::code(0, 0xffff, 0),
            Descriptor::data(0, 0xffffff, BITS_32 | GRANULARITY_4KB),
        ];

        Self { descriptors }
    }

    pub fn load(&self) {
        let descriptor = GDTDescriptor {
            size: (3 * 64) - 1,
            pointer: self,
        };

        unsafe {
            asm!("cli", "lgdt [{0}]", "sti", in(reg) &descriptor);
        }
    }
}
