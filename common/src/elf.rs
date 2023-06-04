use crate::u16_from_slice;
use crate::u32_from_slice;
use crate::u64_from_slice;
use core::fmt::Display;

pub const ELF_FILE_MAGIC: u32 = 0x464C457F;

const PF_X: u32 = 1;
const PF_W: u32 = 2;
const PF_R: u32 = 4;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InstructionSet {
    X86_64,
    Unknown,
}

impl From<u16> for InstructionSet {
    fn from(value: u16) -> Self {
        match value {
            0x3E => Self::X86_64,
            _ => Self::Unknown,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Endianness {
    Little,
    Big,
    Unknown,
}

impl From<u8> for Endianness {
    fn from(value: u8) -> Self {
        match value {
            0x01 => Self::Little,
            0x02 => Self::Big,
            _ => Self::Unknown,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BitFormat {
    Bits32,
    Bits64,
    Unknown,
}

impl From<u8> for BitFormat {
    fn from(value: u8) -> Self {
        match value {
            0x01 => Self::Bits32,
            0x02 => Self::Bits64,
            _ => Self::Unknown,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OSABI {
    SystemV,
    Linux,
    Unknown,
}

impl From<u8> for OSABI {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::SystemV,
            0x03 => Self::Linux,
            _ => Self::Unknown,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Unknown = 0,
    Rel = 1,
    Exec = 2,
    Dyn = 3,
    Core = 4,
}

impl From<u16> for FileType {
    fn from(value: u16) -> Self {
        match value {
            1 => Self::Rel,
            2 => Self::Exec,
            3 => Self::Dyn,
            4 => Self::Core,
            _ => Self::Unknown,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SegmentType {
    Null,
    Load,
    Dynamic,
    Interp,
    Note,
    ShLib,
    Phdr,
    Tls,
    GNUStack,
    GNUEHFrame,
    Relro,
    Unknown(u32),
}

impl From<u32> for SegmentType {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Null,
            1 => Self::Load,
            2 => Self::Dynamic,
            3 => Self::Interp,
            4 => Self::Note,
            5 => Self::ShLib,
            6 => Self::Phdr,
            7 => Self::Tls,
            0x6474e552 => Self::Relro,
            0x6474e551 => Self::GNUStack,
            0x6474e550 => Self::GNUEHFrame,
            value => Self::Unknown(value),
        }
    }
}

impl Display for SegmentType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            Self::Null => "PT_NULL",
            Self::Load => "PT_LOAD",
            Self::Dynamic => "PT_DYNAMIC",
            Self::Interp => "PT_INTERP",
            Self::Note => "PT_NOTE",
            Self::ShLib => "PT_SHLIB",
            Self::Phdr => "PT_PHDR",
            Self::Tls => "PT_TLS",
            Self::Relro => "GNU RELRO",
            Self::GNUStack => "GNU STACK",
            Self::GNUEHFrame => "GNU EH_FRAME",
            Self::Unknown(value) => {
                return f.write_fmt(format_args!("UNKNOWN ({:08x})", value));
            }
        };

        f.write_str(s)
    }
}

pub struct Elf64Header {
    pub magic: u32,
    pub bit_format: BitFormat,
    pub endianness: Endianness,
    pub header_version: u8,
    pub os_abi: OSABI,
    pub abi_version: u8,
    pub file_type: FileType,
    pub instruction_set: InstructionSet,
    pub elf_version: u32,
    pub entry_point: u64,
    pub program_header_table_offset: u64,
    pub section_header_table_offset: u64,
    pub flags: u32,
    pub header_size: u16,
    pub program_header_table_entry_size: u16,
    pub program_header_table_num_entries: u16,
    pub section_header_table_entry_size: u16,
    pub section_header_table_num_entries: u16,
    pub section_header_string_table_index: u16,
}

impl Elf64Header {
    pub fn program_headers<'a>(&self, file: &'a [u8]) -> Elf64ProgramHeaderIterator<'a> {
        Elf64ProgramHeaderIterator {
            program_header_table_offset: self.program_header_table_offset as usize,
            num_entries: self.program_header_table_num_entries as usize,
            entry_size: self.program_header_table_entry_size as usize,
            file,
            current_entry: 0,
        }
    }
}

impl From<&[u8]> for Elf64Header {
    fn from(value: &[u8]) -> Self {
        let magic = u32_from_slice(&value[0..4]);
        let bit_format = value[4].into();
        let endianness = value[5].into();
        let header_version = value[6];
        let os_abi = value[7].into();
        let abi_version = value[8];
        // 7 bytes of padding
        let file_type = u16_from_slice(&value[16..18]).into();
        let instruction_set = u16_from_slice(&value[18..20]).into();
        let elf_version = u32_from_slice(&value[20..24]);
        let entry_point = u64_from_slice(&value[24..32]);
        let program_header_table_offset = u64_from_slice(&value[32..40]);
        let section_header_table_offset = u64_from_slice(&value[40..48]);
        let flags = u32_from_slice(&value[48..52]);
        let header_size = u16_from_slice(&value[52..54]);
        let program_header_table_entry_size = u16_from_slice(&value[54..56]);
        let program_header_table_num_entries = u16_from_slice(&value[56..58]);
        let section_header_table_entry_size = u16_from_slice(&value[58..60]);
        let section_header_table_num_entries = u16_from_slice(&value[60..62]);
        let section_header_string_table_index = u16_from_slice(&value[62..64]);

        Self {
            magic,
            bit_format,
            endianness,
            header_version,
            os_abi,
            abi_version,
            file_type,
            instruction_set,
            elf_version,
            entry_point,
            program_header_table_offset,
            section_header_table_offset,
            flags,
            header_size,
            program_header_table_entry_size,
            program_header_table_num_entries,
            section_header_table_entry_size,
            section_header_table_num_entries,
            section_header_string_table_index,
        }
    }
}

pub struct Elf64ProgramHeaderEntry {
    pub segment_type: SegmentType,
    pub flags: u32,
    pub offset: u64,
    pub virtual_address: u64,
    pub physical_address: u64,
    pub size_in_file: u64,
    pub size_in_memory: u64,
    pub alignment: u64,
}

impl From<&[u8]> for Elf64ProgramHeaderEntry {
    fn from(value: &[u8]) -> Self {
        let segment_type = u32_from_slice(&value[0..4]).into();
        let flags = u32_from_slice(&value[4..8]);
        let offset = u64_from_slice(&value[8..16]);
        let virtual_address = u64_from_slice(&value[16..24]);
        let physical_address = u64_from_slice(&value[24..32]);
        let size_in_file = u64_from_slice(&value[32..40]);
        let size_in_memory = u64_from_slice(&value[40..48]);
        let alignment = u64_from_slice(&value[48..56]);

        Self {
            segment_type,
            flags,
            offset,
            virtual_address,
            physical_address,
            size_in_file,
            size_in_memory,
            alignment,
        }
    }
}

impl Display for Elf64ProgramHeaderEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("    {}\n", self.segment_type))?;
        f.write_fmt(format_args!("  offset {:08x} ", self.offset))?;
        f.write_fmt(format_args!("vaddr {:08x} ", self.virtual_address))?;
        f.write_fmt(format_args!("paddr {:08x} ", self.physical_address))?;
        f.write_fmt(format_args!(
            "align 2**{}\n",
            self.alignment.checked_ilog2().unwrap_or(0)
        ))?;

        f.write_fmt(format_args!("  filesz {:08x} ", self.size_in_file))?;
        f.write_fmt(format_args!("memsz {:08x} ", self.size_in_memory))?;
        f.write_str("flags ")?;

        f.write_str(if (self.flags & PF_R) != 0 { "r" } else { "-" })?;
        f.write_str(if (self.flags & PF_W) != 0 { "r" } else { "-" })?;
        f.write_str(if (self.flags & PF_X) != 0 { "r" } else { "-" })
    }
}

pub struct Elf64ProgramHeaderIterator<'a> {
    program_header_table_offset: usize,
    num_entries: usize,
    entry_size: usize,
    file: &'a [u8],
    current_entry: usize,
}

impl<'a> Iterator for Elf64ProgramHeaderIterator<'a> {
    type Item = Elf64ProgramHeaderEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_entry >= self.num_entries {
            return None;
        }

        let file_offset = self.program_header_table_offset + (self.current_entry * self.entry_size);

        let entry = Elf64ProgramHeaderEntry::from(&self.file[file_offset..]);

        self.current_entry += 1;

        Some(entry)
    }
}
