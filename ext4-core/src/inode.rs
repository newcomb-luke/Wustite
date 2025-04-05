use core::fmt::Debug;

use bin_tools::{read_i32_le, read_u16_le, read_u32_le};

#[derive(Debug, Clone, Copy)]
pub struct Inode {
    /// offset 0x00
    mode: Mode,
    /// offset 0x02
    uid: u16,
    /// offset 0x04
    size: i32,
    /// offset 0x08
    atime: u32,
    /// offset 0x0c
    ctime: u32,
    /// offset 0x10
    mtime: u32,
    /// offset 0x14
    dtime: u32,
    /// offset 0x18
    gid: u16,
    /// offset 0x1a
    links_count: u16,
    /// offset 0x1c
    blocks_count: u32,
    /// offset 0x20
    flags: Flags,
    /// offset 0x24
    osd_1: u32,
    /// offset 0x28
    blocks: Blocks,
    /// offset 0x64
    generation: u32,
    /// offset 0x68
    file_acl: u32,
    /// offset 0x6c
    dir_acl: u32,
    /// offset 0x70
    faddr: u32,
    /// offset 0x74
    osd_2_1: u32,
    /// offset 0x78
    osd_2_2: u32,
    /// offset 0x7c
    osd_2_3: u32
}

impl Inode {
    pub fn read(buffer: &[u8]) -> Self {
        Self {
            mode: Mode::from(read_u16_le(buffer, 0x00)),
            uid: read_u16_le(buffer, 0x02),
            size: read_i32_le(buffer, 0x04),
            atime: read_u32_le(buffer, 0x08),
            ctime: read_u32_le(buffer, 0x0c),
            mtime: read_u32_le(buffer, 0x10),
            dtime: read_u32_le(buffer, 0x14),
            gid: read_u16_le(buffer, 0x18),
            links_count: read_u16_le(buffer, 0x1a),
            blocks_count: read_u32_le(buffer, 0x1c),
            flags: Flags::from(read_u32_le(buffer, 0x20)),
            osd_1: read_u32_le(buffer, 0x24),
            blocks: Blocks::read(&buffer[0x28..]),
            generation: read_u32_le(buffer, 0x64),
            file_acl: read_u32_le(buffer, 0x68),
            dir_acl: read_u32_le(buffer, 0x6c),
            faddr: read_u32_le(buffer, 0x70),
            osd_2_1: read_u32_le(buffer, 0x74),
            osd_2_2: read_u32_le(buffer, 0x78),
            osd_2_3: read_u32_le(buffer, 0x7c),
        }
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Mode(u16);

impl From<u16> for Mode {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl Debug for Mode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}{}{}{}{}{}{} {}{}{} {}{}{}{}{}{}{}{}{}",
               if self.is_socket() { 's' } else { '-' },
               if self.is_symbolic_link() { 'l' } else { '-' },
               if self.is_regular_file() { 'f' } else { '-' },
               if self.is_block_device() { 'b' } else { '-' },
               if self.is_directory() { 'd' } else { '-' },
               if self.is_character_device() { 'c' } else { '-' },
               if self.is_fifo() { 'o' } else { '-' },
               if self.is_set_uid() { 'u' } else { '-' },
               if self.is_set_gid() { 'g' } else { '-' },
               if self.is_sticky_bit_set() { 's' } else { '-' },
               if self.user_read() { 'r' } else { '-' },
               if self.user_write() { 'w' } else { '-' },
               if self.user_execute() { 'x' } else { '-' },
               if self.group_read() { 'r' } else { '-' },
               if self.group_write() { 'w' } else { '-' },
               if self.group_execute() { 'x' } else { '-' },
               if self.others_read() { 'r' } else { '-' },
               if self.others_write() { 'w' } else { '-' },
               if self.others_execute() { 'x' } else { '-' },
        )
    }
}

impl Mode {
    pub fn is_socket(&self) -> bool {
        (0xC000 & self.0) != 0
    }

    pub fn is_symbolic_link(&self) -> bool {
        (0xA000 & self.0) != 0
    }

    pub fn is_regular_file(&self) -> bool {
        (0x8000 & self.0) != 0
    }

    pub fn is_block_device(&self) -> bool {
        (0x6000 & self.0) != 0
    }

    pub fn is_directory(&self) -> bool {
        (0x4000 & self.0) != 0
    }

    pub fn is_character_device(&self) -> bool {
        (0x2000 & self.0) != 0
    }

    pub fn is_fifo(&self) -> bool {
        (0x1000 & self.0) != 0
    }

    pub fn is_set_uid(&self) -> bool {
        (0x0800 & self.0) != 0
    }

    pub fn is_set_gid(&self) -> bool {
        (0x0400 & self.0) != 0
    }

    pub fn is_sticky_bit_set(&self) -> bool {
        (0x0200 & self.0) != 0
    }

    pub fn user_read(&self) -> bool {
        (0x0100 & self.0) != 0
    }

    pub fn user_write(&self) -> bool {
        (0x0080 & self.0) != 0
    }

    pub fn user_execute(&self) -> bool {
        (0x0040 & self.0) != 0
    }

    pub fn group_read(&self) -> bool {
        (0x0020 & self.0) != 0
    }

    pub fn group_write(&self) -> bool {
        (0x0010 & self.0) != 0
    }

    pub fn group_execute(&self) -> bool {
        (0x0008 & self.0) != 0
    }

    pub fn others_read(&self) -> bool {
        (0x0004 & self.0) != 0
    }

    pub fn others_write(&self) -> bool {
        (0x0002 & self.0) != 0
    }

    pub fn others_execute(&self) -> bool {
        (0x0001 & self.0) != 0
    }

    pub fn raw_value(&self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Flags(u32);

impl From<u32> for Flags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl Debug for Flags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}{}{}{} {}{}{}{} {}{}{}{} {}{}{}{}",
               if self.secure_deletion() { 's' } else { '-' },
               if self.record_for_undelete() { 'd' } else { '-' },
               if self.compress_file() { 'c' } else { '-' },
               if self.synchronous_updates() { 'u' } else { '-' },
               if self.immutable_file() { 'i' } else { '-' },
               if self.append_only() { 'a' } else { '-' },
               if self.no_dump_file() { 'd' } else { '-' },
               if self.no_atime() { 't' } else { '-' },
               if self.dirty() { 'd' } else { '-' },
               if self.compressed_blocks() { 'b' } else { '-' },
               if self.no_compress() { 'n' } else { '-' },
               if self.compression_error() { 'e' } else { '-' },
               if self.btree_directory() { 'b' } else { '-' },
               if self.hash_directory() { 'h' } else { '-' },
               if self.afs_directory() { 'a' } else { '-' },
               if self.journal_data() { 'j' } else { '-' },
               )
    }
}

impl Flags {
    pub fn secure_deletion(&self) -> bool {
        (0x00000001 & self.0) != 0
    }

    pub fn record_for_undelete(&self) -> bool {
        (0x00000002 & self.0) != 0
    }

    pub fn compress_file(&self) -> bool {
        (0x00000004 & self.0) != 0
    }

    pub fn synchronous_updates(&self) -> bool {
        (0x00000008 & self.0) != 0
    }

    pub fn immutable_file(&self) -> bool {
        (0x00000010 & self.0) != 0
    }

    pub fn append_only(&self) -> bool {
        (0x00000020 & self.0) != 0
    }

    pub fn no_dump_file(&self) -> bool {
        (0x00000040 & self.0) != 0
    }

    pub fn no_atime(&self) -> bool {
        (0x00000080 & self.0) != 0
    }

    pub fn dirty(&self) -> bool {
        (0x00000100 & self.0) != 0
    }

    pub fn compressed_blocks(&self) -> bool {
        (0x00000200 & self.0) != 0
    }

    pub fn no_compress(&self) -> bool {
        (0x00000400 & self.0) != 0
    }

    pub fn compression_error(&self) -> bool {
        (0x00000800 & self.0) != 0
    }

    pub fn btree_directory(&self) -> bool {
        (0x00001000 & self.0) != 0
    }

    pub fn hash_directory(&self) -> bool {
        (0x00002000 & self.0) != 0
    }

    pub fn afs_directory(&self) -> bool {
        (0x00004000 & self.0) != 0
    }

    pub fn journal_data(&self) -> bool {
        (0x00008000 & self.0) != 0
    }

    pub fn reserved(&self) -> bool {
        (0x80000000 & self.0) != 0
    }

    pub fn raw_value(&self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Blocks([u32; 15]);

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct BlockNumber(u32);

impl From<u32> for BlockNumber {
    fn from(value: u32) -> Self {
        BlockNumber(value)
    }
}

impl From<BlockNumber> for u32 {
    fn from(value: BlockNumber) -> Self {
        value.0
    }
}

impl Blocks {
    pub fn read(buffer: &[u8]) -> Self {
        let mut data: [u32; 15] = [0; 15];
        
        for i in 0..data.len() {
            data[i] = read_u32_le(buffer, i * 4);
        }

        Self(data)
    }

    pub fn blocks(&self) -> &[BlockNumber; 12] {
        // SAFETY: https://users.rust-lang.org/t/unsafe-conversion-from-slice-to-array-reference/88910/11
        unsafe {
            &*(&self.0[0..12]).as_ptr().cast()
        }
    }

    pub fn indirect_block(&self) -> BlockNumber {
        BlockNumber::from(self.0[12])
    }

    pub fn doubly_indirect_block(&self) -> BlockNumber {
        BlockNumber::from(self.0[13])
    }

    pub fn triply_indirect_block(&self) -> BlockNumber {
        BlockNumber::from(self.0[14])
    }
}
