use common::elf::{
    BitFormat, Elf64Header, Endianness, FileType, InstructionSet, SegmentType, ELF_FILE_MAGIC,
    OSABI,
};

#[derive(Debug, Clone, Copy)]
pub enum ElfReadError {
    NotElfFileError,
    BitFormat32UnsupportedError,
    UnknownBitFormatError,
    BigEndianUnsupportedError,
    NotX86InstructionSetError,
    NotExecutableFileError,
    ABIUnsupportedError,
    UnknownProgramSectionType(u32),
}

pub fn validate_elf(bytes: &[u8]) -> Result<(), ElfReadError> {
    let header = Elf64Header::from(bytes);

    if header.magic != ELF_FILE_MAGIC {
        return Err(ElfReadError::NotElfFileError);
    }

    if header.bit_format != BitFormat::Bits64 {
        if header.bit_format == BitFormat::Bits32 {
            return Err(ElfReadError::BitFormat32UnsupportedError);
        }

        return Err(ElfReadError::UnknownBitFormatError);
    }

    if header.endianness != Endianness::Little {
        return Err(ElfReadError::BigEndianUnsupportedError);
    }

    if header.instruction_set != InstructionSet::X86_64 {
        return Err(ElfReadError::NotX86InstructionSetError);
    }

    if header.file_type != FileType::Exec {
        return Err(ElfReadError::NotExecutableFileError);
    }

    if header.os_abi != OSABI::SystemV {
        return Err(ElfReadError::ABIUnsupportedError);
    }

    for entry in header.program_headers(bytes) {
        if let SegmentType::Unknown(value) = entry.segment_type {
            return Err(ElfReadError::UnknownProgramSectionType(value));
        }
    }

    Ok(())
}

///// This function attempts to load an ELF file into memory, and then returns
///// the entry point of the executable
//pub fn load_elf(bytes: *const u8) -> Result<u64, ElfLoadError> {
//    let header = unsafe { (bytes as *const Elf64Header).as_ref().unwrap() };
//
//    validate_elf(bytes).map_err(|e| ElfLoadError::ElfReadError(e))?;
//
//    header.load_program_headers(bytes)?;
//
//    Ok(header.entry_point)
//}
