use crate::u16_from_slice;
use crate::u32_from_slice;
use crate::u64_from_slice;
use core::fmt::Display;

pub const ELF_FILE_MAGIC: u32 = 0x464C457F;

const PF_X: u32 = 1;
const PF_W: u32 = 2;
const PF_R: u32 = 4;

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum RelocationType {
    R_X84_64_RELATIVE,
    Unknown,
}

impl From<u32> for RelocationType {
    fn from(value: u32) -> Self {
        match value {
            8 => Self::R_X84_64_RELATIVE,
            _ => Self::Unknown,
        }
    }
}

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

    pub fn get_maximum_process_image_size(&self, bytes: &[u8]) -> u64 {
        self.program_headers(bytes)
            .map(|e| e.virtual_address + e.size_in_memory)
            .max()
            .unwrap()
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

#[derive(Debug, Clone, Copy)]
pub enum ElfValidateError {
    NotElfFileError,
    BitFormat32UnsupportedError,
    UnknownBitFormatError,
    BigEndianUnsupportedError,
    NotX86InstructionSetError,
    ABIUnsupportedError,
    UnknownProgramSectionType(u32),
}

/// Represents an ELF file in memory, tied to a buffer of
/// immutable bytes
pub struct ElfFile<'a> {
    header: Elf64Header,
    bytes: &'a [u8],
}

impl<'a> ElfFile<'a> {
    pub fn new_validated(bytes: &'a [u8]) -> Result<Self, ElfValidateError> {
        let header = Elf64Header::from(bytes);

        if header.magic != ELF_FILE_MAGIC {
            return Err(ElfValidateError::NotElfFileError);
        }

        if header.bit_format != BitFormat::Bits64 {
            if header.bit_format == BitFormat::Bits32 {
                return Err(ElfValidateError::BitFormat32UnsupportedError);
            }

            return Err(ElfValidateError::UnknownBitFormatError);
        }

        if header.endianness != Endianness::Little {
            return Err(ElfValidateError::BigEndianUnsupportedError);
        }

        if header.instruction_set != InstructionSet::X86_64 {
            return Err(ElfValidateError::NotX86InstructionSetError);
        }

        if header.os_abi != OSABI::SystemV {
            return Err(ElfValidateError::ABIUnsupportedError);
        }

        for entry in header.program_headers(bytes) {
            if let SegmentType::Unknown(value) = entry.segment_type {
                return Err(ElfValidateError::UnknownProgramSectionType(value));
            }
        }

        Ok(Self { header, bytes })
    }

    pub fn file_type(&self) -> FileType {
        self.header.file_type
    }

    pub fn program_headers(&self) -> Elf64ProgramHeaderIterator {
        self.header.program_headers(self.bytes)
    }

    pub fn get_dynamic_section(&self) -> Option<Elf64DynamicSectionIterator> {
        let dynamic_header = self
            .header
            .program_headers(self.bytes)
            .find(|e| e.segment_type == SegmentType::Dynamic)?;

        let section_slice = &self.bytes[dynamic_header.offset as usize..];

        let num_entries = dynamic_header.size_in_file as usize / Dyn64Entry::SIZE_IN_BYTES;

        Some(Elf64DynamicSectionIterator::new(section_slice, num_entries))
    }

    pub fn get_maximum_process_image_size(&self) -> u64 {
        self.header.get_maximum_process_image_size(self.bytes)
    }

    pub fn entry_point(&self) -> u64 {
        self.header.entry_point
    }

    /// The caller MUST guarantee that this memory has nothing else in it, and that
    /// nothing else is using it.
    ///
    /// This pointer MUST be 4 KiB page-aligned.
    pub unsafe fn load_dynamic_file(
        &self,
        destination_buffer: &mut [u8],
    ) -> Result<(), Elf64ProcessImageLoadingError> {
        if destination_buffer.len() < self.get_maximum_process_image_size() as usize {
            return Err(Elf64ProcessImageLoadingError::InsufficientMemoryError);
        }

        let destination_address = destination_buffer.as_mut_ptr();
        let file_start_address = self.bytes.as_ptr();

        let mut rela_section_address = None;
        let mut rela_section_size = None;

        for entry in self
            .get_dynamic_section()
            .ok_or(Elf64ProcessImageLoadingError::MissingDynamicSectionError)?
        {
            match entry.tag() {
                Dyn64EntryTag::Rela => {
                    rela_section_address = Some(entry.value);
                }
                Dyn64EntryTag::RelaSz => {
                    rela_section_size = Some(entry.value);
                }
                _ => {}
            }
        }

        let rela_section_memory_offset = rela_section_address
            .ok_or(Elf64ProcessImageLoadingError::MissingRelaSectionError)?
            as usize;
        let rela_section_address = destination_address.add(rela_section_memory_offset);
        let rela_section_size = rela_section_size
            .ok_or(Elf64ProcessImageLoadingError::MissingRelaSizeSectionError)?
            as usize;

        for segment in self.program_headers() {
            if segment.segment_type == SegmentType::Load {
                Self::load_program_segment(file_start_address, destination_address, segment)?;
            }
        }

        let rela_section_slice =
            core::slice::from_raw_parts_mut(rela_section_address, rela_section_size);
        let num_rela_entries = rela_section_size / Rela64Entry::SIZE_IN_BYTES;

        for entry in Elf64RelaSectionIterator::new(rela_section_slice, num_rela_entries) {
            Self::perform_relocation(destination_address, entry)?;
        }

        Ok(())
    }

    unsafe fn load_program_segment(
        file_start_address: *const u8,
        virtual_memory_offset: *mut u8,
        segment: Elf64ProgramHeaderEntry,
    ) -> Result<(), Elf64ProcessImageLoadingError> {
        if segment.segment_type != SegmentType::Load {
            return Err(Elf64ProcessImageLoadingError::LoadNonLoadableSegmentError);
        }

        let segment_file_ptr = file_start_address.add(segment.offset as usize);
        let segment_memory_ptr = virtual_memory_offset.add(segment.virtual_address as usize);
        let num_bytes_to_copy = segment.size_in_file as usize;
        let memset_start_ptr = segment_memory_ptr.add(num_bytes_to_copy);
        let num_bytes_to_memset = (segment.size_in_memory - segment.size_in_file) as usize;

        // Copy the bytes that are actually in the file
        segment_file_ptr.copy_to_nonoverlapping(segment_memory_ptr, num_bytes_to_copy);

        // Memset the other bytes to zero
        memset_start_ptr.write_bytes(0, num_bytes_to_memset);

        // Hopefully everything went well

        Ok(())
    }

    unsafe fn perform_relocation(
        virtual_memory_offset: *mut u8,
        rela_entry: Rela64Entry,
    ) -> Result<(), Elf64ProcessImageLoadingError> {
        // This is the only type of relocation we support right now, it is the only
        // type I have seen in the wild so far
        if rela_entry.ty() != Rela64EntryType::Relative {
            return Err(
                Elf64ProcessImageLoadingError::UnsupportedRelocationTypeError(rela_entry.ty()),
            );
        }

        let entry_position_in_memory =
            virtual_memory_offset.add(rela_entry.offset as usize) as *mut u64;

        // R_X86_64_RELATIVE relocations are just B + A
        // Where B is the virtual memory offset it is loaded at, and A is the entry's addend value
        // They are 64-bit values.

        let data_to_store = virtual_memory_offset as u64 + rela_entry.addend;

        entry_position_in_memory.write(data_to_store);

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Elf64ProcessImageLoadingError {
    InsufficientMemoryError,
    MissingDynamicSectionError,
    MissingRelaSectionError,
    MissingRelaSizeSectionError,
    LoadNonLoadableSegmentError,
    UnsupportedRelocationTypeError(Rela64EntryType),
}

pub struct Elf64DynamicSectionIterator<'a> {
    section_start: &'a [u8],
    num_entries: usize,
    current_entry: usize,
}

impl<'a> Elf64DynamicSectionIterator<'a> {
    fn new(section_start: &'a [u8], num_entries: usize) -> Self {
        Self {
            section_start,
            num_entries,
            current_entry: 0,
        }
    }
}

impl<'a> Iterator for Elf64DynamicSectionIterator<'a> {
    type Item = Dyn64Entry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_entry >= self.num_entries {
            return None;
        }

        let slice_offset = self.current_entry * Dyn64Entry::SIZE_IN_BYTES;
        let entry_slice = &self.section_start[slice_offset..];

        self.current_entry += 1;

        Some(Dyn64Entry::read(entry_slice))
    }
}

/// Known in the ELF docs as an Elf64_Dyn d_tag
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Dyn64EntryTag {
    Null,
    Needed,
    PltRelSz,
    PltGot,
    Hash,
    StrTab,
    SymTab,
    Rela,
    RelaSz,
    RelaEnt,
    StrSz,
    SymEnt,
    Init,
    Fini,
    SoName,
    RPath,
    Symbolic,
    Rel,
    RelSz,
    RelEnt,
    PltRel,
    Debug,
    TextRel,
    JmpRel,
    BindNow,
    Unknown(u64),
}

impl From<u64> for Dyn64EntryTag {
    fn from(value: u64) -> Self {
        match value {
            0 => Self::Null,
            1 => Self::Needed,
            2 => Self::PltRelSz,
            3 => Self::PltGot,
            4 => Self::Hash,
            5 => Self::StrTab,
            6 => Self::SymTab,
            7 => Self::Rela,
            8 => Self::RelaSz,
            9 => Self::RelaEnt,
            10 => Self::StrSz,
            11 => Self::SymEnt,
            12 => Self::Init,
            13 => Self::Fini,
            14 => Self::SoName,
            15 => Self::RPath,
            16 => Self::Symbolic,
            17 => Self::Rel,
            18 => Self::RelSz,
            19 => Self::RelEnt,
            20 => Self::PltRel,
            21 => Self::Debug,
            22 => Self::TextRel,
            23 => Self::JmpRel,
            24 => Self::BindNow,
            v => Self::Unknown(v),
        }
    }
}

impl Display for Dyn64EntryTag {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            Self::Null => "NULL",
            Self::Needed => "NEEDED",
            Self::PltRelSz => "PLTRELSZ",
            Self::PltGot => "PLTGOT",
            Self::Hash => "HASH",
            Self::StrTab => "STRTAB",
            Self::SymTab => "SYMTAB",
            Self::Rela => "RELA",
            Self::RelaSz => "RELASZ",
            Self::RelaEnt => "RELAENT",
            Self::StrSz => "STRSZ",
            Self::SymEnt => "SYMENT",
            Self::Init => "INIT",
            Self::Fini => "FINI",
            Self::SoName => "SONAME",
            Self::RPath => "RPATH",
            Self::Symbolic => "SYMBOLIC",
            Self::Rel => "REL",
            Self::RelSz => "RELSZ",
            Self::RelEnt => "RELENT",
            Self::PltRel => "PLTREL",
            Self::Debug => "DEBUG",
            Self::TextRel => "TEXTREL",
            Self::JmpRel => "JMPREL",
            Self::BindNow => "BINDNOW",
            Self::Unknown(v) => {
                return f.write_fmt(format_args!("Unknown ({v})"));
            }
        };

        f.write_str(s)
    }
}

#[derive(Clone, Copy)]
pub struct Dyn64Entry {
    tag: Dyn64EntryTag,
    value: u64,
}

impl Dyn64Entry {
    const SIZE_IN_BYTES: usize = 16;

    fn read(bytes: &[u8]) -> Self {
        let tag = u64_from_slice(&bytes[0..8]);
        let value = u64_from_slice(&bytes[8..16]);

        Self {
            tag: tag.into(),
            value,
        }
    }

    pub fn tag(&self) -> Dyn64EntryTag {
        self.tag
    }

    pub fn value(&self) -> u64 {
        self.value
    }
}

impl Display for Dyn64Entry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{} 0x{:016x}", self.tag, self.value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rela64EntryType {
    None,
    Word64,
    PC32,
    GOT32,
    PLT32,
    Copy,
    GlobDat,
    JumpSlot,
    Relative,
    Unknown(u32),
}

impl From<u32> for Rela64EntryType {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Word64,
            2 => Self::PC32,
            3 => Self::GOT32,
            4 => Self::PLT32,
            5 => Self::Copy,
            6 => Self::GlobDat,
            7 => Self::JumpSlot,
            8 => Self::Relative,
            v => Self::Unknown(v),
        }
    }
}

impl Display for Rela64EntryType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            Self::None => "R_X84_64_NONE",
            Self::Word64 => "R_X84_64_64",
            Self::PC32 => "R_X84_64_PC32",
            Self::GOT32 => "R_X84_64_GOT32",
            Self::PLT32 => "R_X84_64_PLT32",
            Self::Copy => "R_X84_64_COPY",
            Self::GlobDat => "R_X84_64_GLOB_DAT",
            Self::JumpSlot => "R_X84_64_JUMP_SLOT",
            Self::Relative => "R_X84_64_RELATIVE",
            Self::GlobDat => "R_X84_64_GLOB_DAT",
            Self::Unknown(v) => {
                return f.write_fmt(format_args!("Unknown ({v})"));
            }
        };

        f.write_str(s)
    }
}

#[derive(Clone, Copy)]
pub struct Rela64Entry {
    offset: u64,
    info: u64,
    addend: u64,
}

impl Rela64Entry {
    const SIZE_IN_BYTES: usize = 24;

    fn read(bytes: &[u8]) -> Self {
        let offset = u64_from_slice(&bytes[0..8]);
        let info = u64_from_slice(&bytes[8..16]);
        let addend = u64_from_slice(&bytes[16..24]);

        Self {
            offset,
            info,
            addend,
        }
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }

    pub fn symbol_table(&self) -> u32 {
        (self.info >> 32) as u32
    }

    pub fn ty(&self) -> Rela64EntryType {
        ((self.info & 0xffffffff) as u32).into()
    }

    pub fn addend(&self) -> u64 {
        self.addend
    }
}

impl Display for Rela64Entry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "{:016x} 0x{:016x} {}",
            self.offset,
            self.addend,
            self.ty(),
        ))
    }
}

pub struct Elf64RelaSectionIterator<'a> {
    section_start: &'a [u8],
    num_entries: usize,
    current_entry: usize,
}

impl<'a> Elf64RelaSectionIterator<'a> {
    fn new(section_start: &'a [u8], num_entries: usize) -> Self {
        Self {
            section_start,
            num_entries,
            current_entry: 0,
        }
    }
}

impl<'a> Iterator for Elf64RelaSectionIterator<'a> {
    type Item = Rela64Entry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_entry >= self.num_entries {
            return None;
        }

        let slice_offset = self.current_entry * Rela64Entry::SIZE_IN_BYTES;
        let entry_slice = &self.section_start[slice_offset..];

        self.current_entry += 1;

        Some(Rela64Entry::read(entry_slice))
    }
}
