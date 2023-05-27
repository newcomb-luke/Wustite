#![allow(dead_code)]

use x86_64::VirtAddr;

const EBDA_START: usize = 0x00080000;
const EBDA_END: usize = 0x0009FFFF;

const BIOS_BELOW_START: usize = 0x000E0000;
const BIOS_BELOW_END: usize = 0x000FFFFF;

pub struct ACPIReader {
    rsdp: ACPIRsdp,
    rsdt: ACPIRsdt,
    fadt: ACPIFadt,
}

impl ACPIReader {
    pub fn read(physical_memory_offset: VirtAddr) -> Option<Self> {
        let rsdp = ACPIRsdp::find(physical_memory_offset)?;

        if rsdp.revision() != 0 {
            return None;
        }

        let rsdt = ACPIRsdt::parse(physical_memory_offset, rsdp.rsdt_address)?;

        let fadt_addr = rsdt.find_table(physical_memory_offset, ACPISDT::Fadt)?;
        let fadt = ACPIFadt::parse(physical_memory_offset, fadt_addr)?;

        Some(Self { rsdp, rsdt, fadt })
    }
}

#[derive(Clone, Copy)]
enum ACPISDT {
    Fadt,
}

impl ACPISDT {
    fn signature(&self) -> &'static str {
        match self {
            Self::Fadt => "FACP",
        }
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
    rsdt_address: u32,
    header: ACPISDTHeader,
}

impl ACPIRsdt {
    fn parse(physical_memory_offset: VirtAddr, address: u32) -> Option<Self> {
        const RSDT_SIGNATURE: &str = "RSDT";

        let ptr_offset: *const u8 = physical_memory_offset.as_ptr();
        let rsdt_ptr = unsafe { ptr_offset.add(address as usize) };

        let header = ACPISDTHeader::parse(rsdt_ptr);

        if RSDT_SIGNATURE != header.signature()
            || !validate_checksum_ptr(rsdt_ptr, header.length as usize)
        {
            return None;
        }

        Some(Self {
            rsdt_address: address,
            header,
        })
    }

    fn find_table(&self, physical_memory_offset: VirtAddr, table: ACPISDT) -> Option<u32> {
        let ptr_offset: *const u8 = physical_memory_offset.as_ptr();
        let rsdt_ptr = unsafe { ptr_offset.add(self.rsdt_address as usize) };
        let num_sdt_pointers = (self.header.length as usize - ACPISDTHeader::SIZE_IN_MEMORY) / 4;

        unsafe {
            let first_sdt_pointer = rsdt_ptr.add(ACPISDTHeader::SIZE_IN_MEMORY);

            for i in 0..num_sdt_pointers {
                let sdt_pointer = first_sdt_pointer.add(i * 4);

                let mut buffer: [u8; 4] = [0; 4];
                sdt_pointer.copy_to(buffer.as_mut_ptr(), 4);
                let address_of_table = u32::from_ne_bytes(buffer);
                // Reuse the buffer
                let ptr_to_table = ptr_offset.add(address_of_table as usize);
                ptr_to_table.copy_to(buffer.as_mut_ptr(), 4);

                if buffer == table.signature().as_bytes() {
                    return Some(address_of_table);
                }
            }
        }

        None
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PreferredPowerManagementProfile {
    Unspecified,
    Desktop,
    Mobile,
    Workstation,
    EnterpriseServer,
    SOHOServer,
    AppliancePC,
    PerformanceServer,
    Tablet,
    Unknown,
}

impl From<u8> for PreferredPowerManagementProfile {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Unspecified,
            1 => Self::Desktop,
            2 => Self::Mobile,
            3 => Self::Workstation,
            4 => Self::EnterpriseServer,
            5 => Self::SOHOServer,
            6 => Self::AppliancePC,
            7 => Self::PerformanceServer,
            8 => Self::Tablet,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ACPIAddressSpace {
    SystemMemory,
    SystemIO,
    PCIConfigurationSpace,
    EmbeddedController,
    SystemManagementBus,
    SystemCMOS,
    PCIDeviceBARTarget,
    IntelligentPlatformManagementInfrastructure,
    GeneralPurposeIO,
    GenericSerialBus,
    PlatformCommunicationChannel,
    Unknown,
}

impl From<u8> for ACPIAddressSpace {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::SystemMemory,
            1 => Self::SystemIO,
            2 => Self::PCIConfigurationSpace,
            3 => Self::EmbeddedController,
            4 => Self::SystemManagementBus,
            5 => Self::SystemCMOS,
            6 => Self::PCIDeviceBARTarget,
            7 => Self::IntelligentPlatformManagementInfrastructure,
            8 => Self::GeneralPurposeIO,
            9 => Self::GenericSerialBus,
            10 => Self::PlatformCommunicationChannel,
            _ => Self::Unknown,
        }
    }
}

struct GenericAddressStructure {
    address_space: ACPIAddressSpace,
    bit_width: u8,
    bit_offset: u8,
    access_size: u8,
    address: u64,
}

/// Fixed ACPI description table
struct ACPIFadt {
    header: ACPISDTHeader,
    firmware_control: u32,
    dsdt_address: u32,
    int_model: u8,
    preferred_power_management_profile: PreferredPowerManagementProfile,
    sci_interrupt: u16,
    smi_command_port: u32,
    acpi_enable: u8,
    acpi_disable: u8,
    s4bios_req: u8,
    pstate_control: u8,
    pm1a_event_block: u32,
    pm1b_event_block: u32,
    pm1a_control_block: u32,
    pm1b_control_block: u32,
    pm2_control_block: u32,
    pm_timer_block: u32,
    gpe0_block: u32,
    gpe1_block: u32,
    pm1_event_length: u8,
    pm1_control_length: u8,
    pm2_control_length: u8,
    pm_timer_length: u8,
    gpe0_length: u8,
    gpe1_length: u8,
    gpe1_base: u8,
    c_state_control: u8,
    worst_c2_latency: u16,
    worst_c3_latency: u16,
    flush_size: u16,
    flush_stride: u16,
    duty_offset: u8,
    duty_width: u8,
    day_alarm: u8,
    month_alarm: u8,
    century: u8,
    boot_architecture_flags: u16,
    _reserved2: u8,
    flags: u32,
}

impl ACPIFadt {
    fn parse(physical_memory_offset: VirtAddr, address: u32) -> Option<Self> {
        const FADT_SIGNATURE: &str = "FACP";

        let ptr_offset: *const u8 = physical_memory_offset.as_ptr();
        let fadt_ptr = unsafe { ptr_offset.add(address as usize) };

        let header = ACPISDTHeader::parse(fadt_ptr);

        unsafe {
            if FADT_SIGNATURE != header.signature()
                || !validate_checksum_ptr(fadt_ptr, header.length as usize)
            {
                return None;
            }

            let data_ptr = fadt_ptr.add(ACPISDTHeader::SIZE_IN_MEMORY);
            let mut buffer: [u8; 116] = [0; 116];
            data_ptr.copy_to(buffer.as_mut_ptr(), 116);

            let firmware_control = u32_from_slice(&buffer[0..4]);
            let dsdt_address = u32_from_slice(&buffer[4..8]);
            let int_model = buffer[8];
            let preferred_power_management_profile = buffer[9].into();
            let sci_interrupt = u16_from_slice(&buffer[10..12]);
            let smi_command_port = u32_from_slice(&buffer[12..16]);
            let acpi_enable = buffer[16];
            let acpi_disable = buffer[17];
            let s4bios_req = buffer[18];
            let pstate_control = buffer[19];
            let pm1a_event_block = u32_from_slice(&buffer[20..24]);
            let pm1b_event_block = u32_from_slice(&buffer[24..28]);
            let pm1a_control_block = u32_from_slice(&buffer[28..32]);
            let pm1b_control_block = u32_from_slice(&buffer[32..36]);
            let pm2_control_block = u32_from_slice(&buffer[36..40]);
            let pm_timer_block = u32_from_slice(&buffer[40..44]);
            let gpe0_block = u32_from_slice(&buffer[44..48]);
            let gpe1_block = u32_from_slice(&buffer[48..52]);
            let pm1_event_length = buffer[52];
            let pm1_control_length = buffer[53];
            let pm2_control_length = buffer[54];
            let pm_timer_length = buffer[55];
            let gpe0_length = buffer[56];
            let gpe1_length = buffer[57];
            let gpe1_base = buffer[58];
            let c_state_control = buffer[59];
            let worst_c2_latency = u16_from_slice(&buffer[60..62]);
            let worst_c3_latency = u16_from_slice(&buffer[62..64]);
            let flush_size = u16_from_slice(&buffer[64..66]);
            let flush_stride = u16_from_slice(&buffer[66..68]);
            let duty_offset = buffer[68];
            let duty_width = buffer[69];
            let day_alarm = buffer[70];
            let month_alarm = buffer[71];
            let century = buffer[72];
            let boot_architecture_flags = u16_from_slice(&buffer[73..75]);
            let _reserved2 = buffer[75];
            let flags = u32_from_slice(&buffer[76..80]);

            Some(Self {
                header,
                firmware_control,
                dsdt_address,
                int_model,
                preferred_power_management_profile,
                sci_interrupt,
                smi_command_port,
                acpi_enable,
                acpi_disable,
                s4bios_req,
                pstate_control,
                pm1a_event_block,
                pm1b_event_block,
                pm1a_control_block,
                pm1b_control_block,
                pm2_control_block,
                pm_timer_block,
                gpe0_block,
                gpe1_block,
                pm1_event_length,
                pm1_control_length,
                pm2_control_length,
                pm_timer_length,
                gpe0_length,
                gpe1_length,
                gpe1_base,
                c_state_control,
                worst_c2_latency,
                worst_c3_latency,
                flush_size,
                flush_stride,
                duty_offset,
                duty_width,
                day_alarm,
                month_alarm,
                century,
                boot_architecture_flags,
                _reserved2,
                flags,
            })
        }
    }
}

fn u32_from_slice(buffer: &[u8]) -> u32 {
    let mut bytes: [u8; 4] = [0; 4];
    bytes.copy_from_slice(buffer);
    u32::from_ne_bytes(bytes)
}

fn u16_from_slice(buffer: &[u8]) -> u16 {
    let mut bytes: [u8; 2] = [0; 2];
    bytes.copy_from_slice(buffer);
    u16::from_ne_bytes(bytes)
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
