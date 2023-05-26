use alloc::vec::Vec;
use x86_64::VirtAddr;

use crate::{kprint, kprintln};

const EBDA_START: usize = 0x00080000;
const EBDA_END: usize = 0x0009FFFF;

const BIOS_BELOW_START: usize = 0x000E0000;
const BIOS_BELOW_END: usize = 0x000FFFFF;

pub struct ACPIReader {
    rsdp: ACPIRsdp,
    rsdt: ACPIRsdt,
}

impl ACPIReader {
    pub fn read(physical_memory_offset: VirtAddr) -> Option<Self> {
        let rsdp = ACPIRsdp::find(physical_memory_offset)?;

        kprintln!("RSDP revision: {}", rsdp.revision());

        if rsdp.revision() != 0 {
            return None;
        }

        let rsdt = ACPIRsdt::parse(physical_memory_offset, rsdp.rsdt_address)?;

        Some(Self { rsdp, rsdt })
    }
}

/// Root system description pointer
struct ACPIRsdp {
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
}

impl ACPIRsdp {
    pub fn find(physical_memory_offset: VirtAddr) -> Option<Self> {
        unsafe {
            let ptr_offset: *const u8 = physical_memory_offset.as_ptr();

            let rsdp_ptr = Self::search_for_self(
                ptr_offset.add(BIOS_BELOW_START),
                ptr_offset.add(BIOS_BELOW_END),
            )
            .or_else(|| {
                Self::search_for_self(ptr_offset.add(EBDA_START), ptr_offset.add(EBDA_END))
            })?;

            let mut buffer: [u8; 20] = [0; 20];
            rsdp_ptr.copy_to(buffer.as_mut_ptr(), 20);

            if !validate_checksum(&buffer) {
                return None;
            }

            let checksum = buffer[8];

            let mut oem_id: [u8; 6] = [0; 6];
            oem_id.copy_from_slice(&buffer[9..15]);

            let revision = buffer[15];

            let rsdt_address = u32_from_slice(&buffer[16..]);

            Some(Self {
                checksum,
                oem_id,
                revision,
                rsdt_address,
            })
        }
    }

    pub fn oem_id(&self) -> &str {
        core::str::from_utf8(&self.oem_id).unwrap()
    }

    pub fn revision(&self) -> u8 {
        self.revision
    }

    pub fn rsdt_address(&self) -> u32 {
        self.rsdt_address
    }

    fn search_for_self(start: *const u8, end: *const u8) -> Option<*const u8> {
        const RSDP_SIGNATURE: &str = "RSD PTR ";

        let mut buffer: [u8; 8] = [0; 8];

        // It will always be on a 16-byte boundary
        let mut addr = start;

        while addr < end {
            unsafe {
                addr.copy_to(buffer.as_mut_ptr(), 8);
            }

            if RSDP_SIGNATURE.as_bytes() == buffer {
                return Some(addr as *const u8);
            }

            addr = unsafe { addr.add(16) };
        }

        None
    }
}

/// Generic ACPI system description table header
struct ACPISDTHeader {
    signature: [u8; 4],
    length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    creator_id: u32,
    creator_revision: u32,
}

impl ACPISDTHeader {
    const SIZE_IN_MEMORY: usize = 36;

    fn parse(ptr: *const u8) -> Self {
        unsafe {
            let mut buffer: [u8; Self::SIZE_IN_MEMORY] = [0; Self::SIZE_IN_MEMORY];
            ptr.copy_to(buffer.as_mut_ptr(), Self::SIZE_IN_MEMORY);

            let mut signature: [u8; 4] = [0; 4];
            signature.copy_from_slice(&buffer[0..4]);

            let length = u32_from_slice(&buffer[4..8]);
            let revision = buffer[8];
            let checksum = buffer[9];

            let mut oem_id: [u8; 6] = [0; 6];
            oem_id.copy_from_slice(&buffer[10..16]);

            let mut oem_table_id: [u8; 8] = [0; 8];
            oem_table_id.copy_from_slice(&buffer[16..24]);

            let oem_revision = u32_from_slice(&buffer[24..28]);
            let creator_id = u32_from_slice(&buffer[28..32]);
            let creator_revision = u32_from_slice(&buffer[32..]);

            Self {
                signature,
                length,
                revision,
                checksum,
                oem_id,
                oem_table_id,
                oem_revision,
                creator_id,
                creator_revision,
            }
        }
    }

    pub fn signature(&self) -> &str {
        core::str::from_utf8(&self.signature).unwrap()
    }
}

/// Root system description table
struct ACPIRsdt {
    header: ACPISDTHeader,
    sdt_pointers: Vec<u32>,
}

impl ACPIRsdt {
    fn parse(physical_memory_offset: VirtAddr, address: u32) -> Option<Self> {
        const RSDT_SIGNATURE: &str = "RSDT";

        let ptr_offset: *const u8 = physical_memory_offset.as_ptr();
        let rsdt_ptr = unsafe { ptr_offset.add(address as usize) };

        kprintln!("rsdt: {:?}", rsdt_ptr);

        for i in 0..16 {
            for j in 0..8 {
                kprint!("{:02x} ", unsafe { rsdt_ptr.add(i * 16 + j).read() });
            }
            kprintln!();
        }

        let header = ACPISDTHeader::parse(rsdt_ptr);

        unsafe {
            if RSDT_SIGNATURE != header.signature()
                || !validate_checksum_ptr(rsdt_ptr, header.length as usize)
            {
                return None;
            }

            let num_sdt_pointers = (header.length as usize - ACPISDTHeader::SIZE_IN_MEMORY) / 4;

            let mut sdt_pointers = Vec::with_capacity(num_sdt_pointers);

            let first_sdt_pointer = rsdt_ptr.add(ACPISDTHeader::SIZE_IN_MEMORY);

            for i in 0..num_sdt_pointers {
                let sdt_pointer = first_sdt_pointer.add(i * 4);

                let mut buffer: [u8; 4] = [0; 4];
                sdt_pointer.copy_to(buffer.as_mut_ptr(), 4);

                let value = u32::from_ne_bytes(buffer);

                let header = ACPISDTHeader::parse(ptr_offset.add(value as usize));

                kprintln!("Found {}", header.signature());

                sdt_pointers.push(value);
            }

            Some(Self {
                header,
                sdt_pointers,
            })
        }
    }
}

fn u32_from_slice(buffer: &[u8]) -> u32 {
    let mut bytes: [u8; 4] = [0; 4];
    bytes.copy_from_slice(buffer);
    u32::from_ne_bytes(bytes)
}

fn validate_checksum_ptr(ptr: *const u8, length: usize) -> bool {
    let mut sum: u32 = 0;

    for i in 0..length {
        unsafe {
            sum += ptr.add(i).read() as u32;
        }
    }

    (sum % 0x100) == 0
}

fn validate_checksum(bytes: &[u8]) -> bool {
    let sum: u32 = bytes.iter().map(|b| *b as u32).sum();
    (sum % 0x100) == 0
}
