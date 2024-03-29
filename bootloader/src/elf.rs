#![allow(dead_code)]

use crate::{print, println};

const ELF_FILE_MAGIC: u32 = 0x464C457F;
const ELF_FILE_32_BIT: u8 = 1;
const ELF_FILE_64_BIT: u8 = 2;
const ELF_FILE_LITTLE_ENDIAN: u8 = 1;
const ELF_FILE_X86_64_INSTRUCTION_SET: u16 = 0x3E;

const PF_X: u32 = 1;
const PF_W: u32 = 2;
const PF_R: u32 = 4;

#[repr(C, packed)]
struct Elf64Header {
    magic: u32,
    bit_format: u8,
    endianness: u8,
    header_version: u8,
    os_abi: u8,
    abi_version: u8,
    __padding: [u8; 7],
    file_type: u16,
    instruction_set: u16,
    elf_version: u32,
    entry_point: u64,
    program_header_table_offset: u64,
    section_header_table_offset: u64,
    flags: u32,
    header_size: u16,
    program_header_table_entry_size: u16,
    program_header_table_num_entries: u16,
    section_header_table_entry_size: u16,
    section_header_table_num_entries: u16,
    section_header_string_table_index: u16,
}

#[derive(Clone, Copy, PartialEq)]
#[repr(u32)]
enum SegmentType {
    Null = 0,
    Load = 1,
    Dynamic = 2,
    Interp = 3,
    Note = 4,
    ShLib = 5,
    Phdr = 6,
    Tls = 7,
    GNUStack = 0x6474e551,
    GNUEHFrame = 0x6474e550,
}

impl SegmentType {
    fn try_from(value: u32) -> Result<Self, ()> {
        match value {
            0 => Ok(Self::Null),
            1 => Ok(Self::Load),
            2 => Ok(Self::Dynamic),
            3 => Ok(Self::Interp),
            4 => Ok(Self::Note),
            5 => Ok(Self::ShLib),
            6 => Ok(Self::Phdr),
            7 => Ok(Self::Tls),
            0x6474e551 => Ok(Self::GNUStack),
            0x6474e550 => Ok(Self::GNUEHFrame),
            _ => Err(()),
        }
    }
}

impl SegmentType {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Null => "PT_NULL",
            Self::Load => "PT_LOAD",
            Self::Dynamic => "PT_DYNAMIC",
            Self::Interp => "PT_INTERP",
            Self::Note => "PT_NOTE",
            Self::ShLib => "PT_SHLIB",
            Self::Phdr => "PT_PHDR",
            Self::Tls => "PT_TLS",
            Self::GNUStack => "PT_GNU_STACK",
            Self::GNUEHFrame => "PH_GNU_EH_FRAME",
        }
    }
}

#[repr(C, packed)]
struct Elf64ProgramHeaderEntry {
    segment_type: u32,
    flags: u32,
    offset: u64,
    virtual_address: u64,
    physical_address: u64,
    size_in_file: u64,
    size_in_memory: u64,
    alignment: u64,
}

impl Elf64ProgramHeaderEntry {
    fn segment_type(&self) -> Option<SegmentType> {
        SegmentType::try_from(unsafe { core::ptr::addr_of!(self.segment_type).read_unaligned() })
            .ok()
    }

    fn flags(&self) -> u32 {
        unsafe { core::ptr::addr_of!(self.flags).read_unaligned() }
    }

    fn offset(&self) -> u64 {
        unsafe { core::ptr::addr_of!(self.offset).read_unaligned() }
    }

    fn virtual_address(&self) -> u64 {
        unsafe { core::ptr::addr_of!(self.virtual_address).read_unaligned() }
    }

    fn physical_address(&self) -> u64 {
        unsafe { core::ptr::addr_of!(self.physical_address).read_unaligned() }
    }

    fn size_in_file(&self) -> u64 {
        unsafe { core::ptr::addr_of!(self.size_in_file).read_unaligned() }
    }

    fn size_in_memory(&self) -> u64 {
        unsafe { core::ptr::addr_of!(self.size_in_memory).read_unaligned() }
    }

    fn alignment(&self) -> u64 {
        unsafe { core::ptr::addr_of!(self.alignment).read_unaligned() }
    }

    fn print(&self) {
        let segment_str = if let Some(segment_type) = self.segment_type() {
            segment_type.as_str()
        } else {
            "UNKNOWN"
        };

        println!("    {}", segment_str);

        print!("  offset {:08x} ", self.offset());

        print!("vaddr {:08x} ", self.virtual_address());

        print!("paddr {:08x} ", self.physical_address());

        println!("align 2**{}", self.alignment().checked_ilog2().unwrap_or(0));

        print!("  filesz {:08x} ", self.size_in_file());

        print!("memsz {:08x} ", self.size_in_memory());

        print!("flags ");

        if (self.flags() & PF_R) != 0 {
            print!("r");
        } else {
            print!("-");
        }

        if (self.flags() & PF_W) != 0 {
            print!("w");
        } else {
            print!("-");
        }

        if (self.flags() & PF_X) != 0 {
            println!("x");
        } else {
            println!("-");
        }
    }

    fn load(&self, bytes: *const u8) -> Result<(), ElfLoadError> {
        if let Some(seg_type) = self.segment_type() {
            if seg_type != SegmentType::Load {
                return Ok(());
            }
        } else {
            return Err(ElfLoadError::UnknownSegmentTypeError);
        }

        let virtual_address = self.virtual_address() as *mut u8;
        let segment_address = unsafe { bytes.offset(self.offset() as isize) };

        unsafe {
            segment_address.copy_to_nonoverlapping(virtual_address, self.size_in_file() as usize)
        };

        let bytes_to_zero = self.size_in_memory() - self.size_in_file();
        let zero_start = unsafe { virtual_address.offset(self.size_in_file() as isize) };

        unsafe {
            zero_start.write_bytes(0, bytes_to_zero as usize);
        }

        // println!(
        //     "Loaded {:08x} bytes into memory at address {:?}",
        //     self.size_in_file(),
        //     virtual_address
        // );

        // println!(
        //     "Zeroed {:08x} remaining bytes in memory at address {:?}",
        //     bytes_to_zero, virtual_address
        // );

        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq)]
#[repr(u16)]
enum FileType {
    Unknown = 0,
    Rel = 1,
    Exec = 2,
    Dyn = 3,
    Core = 4,
}

impl Elf64Header {
    fn magic(&self) -> u32 {
        unsafe { core::ptr::addr_of!(self.magic).read_unaligned() }
    }

    fn bit_format(&self) -> u8 {
        self.bit_format
    }

    fn endianness(&self) -> u8 {
        self.endianness
    }

    fn file_type(&self) -> FileType {
        let raw = unsafe { core::ptr::addr_of!(self.file_type).read_unaligned() };

        match raw {
            1 => FileType::Rel,
            2 => FileType::Exec,
            3 => FileType::Dyn,
            4 => FileType::Core,
            _ => FileType::Unknown,
        }
    }

    fn instruction_set(&self) -> u16 {
        unsafe { core::ptr::addr_of!(self.instruction_set).read_unaligned() }
    }

    fn entry_point(&self) -> u64 {
        unsafe { core::ptr::addr_of!(self.entry_point).read_unaligned() }
    }

    fn program_header_table_offset(&self) -> u64 {
        unsafe { core::ptr::addr_of!(self.program_header_table_offset).read_unaligned() }
    }

    fn program_header_table_num_entries(&self) -> u16 {
        unsafe { core::ptr::addr_of!(self.program_header_table_num_entries).read_unaligned() }
    }

    fn print_program_header_table(&self, bytes: *const u8) {
        let entry_ptr = unsafe {
            (bytes.offset(self.program_header_table_offset() as isize))
                as *const Elf64ProgramHeaderEntry
        };

        for i in 0..self.program_header_table_num_entries() as isize {
            let entry = unsafe { entry_ptr.offset(i).as_ref().unwrap() };
            entry.print();
        }
    }

    fn load_program_headers(&self, bytes: *const u8) -> Result<(), ElfLoadError> {
        let entry_ptr = unsafe {
            (bytes.offset(self.program_header_table_offset() as isize))
                as *const Elf64ProgramHeaderEntry
        };

        for i in 0..self.program_header_table_num_entries() as isize {
            let entry = unsafe { entry_ptr.offset(i).as_ref().unwrap() };
            entry.load(bytes)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ElfReadError {
    NotElfFileError,
    BitFormat32UnsupportedError,
    UnknownBitFormatError,
    BigEndianUnsupportedError,
    NotX86InstructionSetError,
    NotExecutableFileError,
}

pub fn validate_elf(bytes: *const u8) -> Result<(), ElfReadError> {
    let header = unsafe { (bytes as *const Elf64Header).as_ref().unwrap() };

    if header.magic() != ELF_FILE_MAGIC {
        return Err(ElfReadError::NotElfFileError);
    }

    if header.bit_format() != ELF_FILE_64_BIT {
        if header.bit_format() == ELF_FILE_32_BIT {
            return Err(ElfReadError::BitFormat32UnsupportedError);
        }

        return Err(ElfReadError::UnknownBitFormatError);
    }

    if header.endianness() != ELF_FILE_LITTLE_ENDIAN {
        return Err(ElfReadError::BigEndianUnsupportedError);
    }

    if header.instruction_set() != ELF_FILE_X86_64_INSTRUCTION_SET {
        return Err(ElfReadError::NotX86InstructionSetError);
    }

    if header.file_type() != FileType::Exec {
        return Err(ElfReadError::NotExecutableFileError);
    }

    // header.print_program_header_table(bytes);

    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub enum ElfLoadError {
    ElfReadError(ElfReadError),
    UnknownSegmentTypeError,
}

/// This function attempts to load an ELF file into memory, and then returns
/// the entry point of the executable
pub fn load_elf(bytes: *const u8) -> Result<u64, ElfLoadError> {
    let header = unsafe { (bytes as *const Elf64Header).as_ref().unwrap() };

    validate_elf(bytes).map_err(|e| ElfLoadError::ElfReadError(e))?;

    header.load_program_headers(bytes)?;

    Ok(header.entry_point())
}
