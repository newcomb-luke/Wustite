use core::fmt::{Debug, Display};

use crate::Error;

use bin_tools::{read_i16_le, read_u16_be, read_u16_le, read_u32_be, read_u32_le, read_u64_le};

#[derive(Debug, Copy, Clone)]
pub struct FileSystemState(u16);

impl FileSystemState {
    pub fn cleanly_unmounted(&self) -> bool {
        (self.0 & 0x0001) != 0
    }

    pub fn errors_detected(&self) -> bool {
        (self.0 & 0x0002) != 0
    }

    pub fn orphans_being_recovered(&self) -> bool {
        (self.0 & 0x0004) != 0
    }

    pub fn raw_value(&self) -> u16 {
        self.0
    }
}

impl From<u16> for FileSystemState {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum SuperBlockErrorPolicy {
    Continue,
    RemountAsReadOnly,
    Panic,
    Unknown,
}

impl From<u16> for SuperBlockErrorPolicy {
    fn from(value: u16) -> Self {
        match value {
            1 => Self::Continue,
            2 => Self::RemountAsReadOnly,
            3 => Self::Panic,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum FileSystemCreatorOS {
    Linux,
    Hurd,
    Masix,
    FreeBSD,
    Lites,
    Wustite,
    Unknown,
}

impl From<u32> for FileSystemCreatorOS {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Linux,
            1 => Self::Hurd,
            2 => Self::Masix,
            3 => Self::FreeBSD,
            4 => Self::Lites,
            5 => Self::Wustite,
            _ => Self::Unknown,
        }
    }
}

impl Display for FileSystemCreatorOS {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Linux => "Linux",
                Self::Hurd => "Hurd",
                Self::Masix => "Masix",
                Self::FreeBSD => "FreeBSD",
                Self::Lites => "Lites",
                Self::Wustite => "Wustite",
                Self::Unknown => "<unknown>",
            }
        )
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Revision {
    Original,
    V2,
    Unknown,
}

impl From<u32> for Revision {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Original,
            1 => Self::V2,
            _ => Self::Unknown,
        }
    }
}

impl Display for Revision {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Original => "0 (original)",
                Self::V2 => "1 (dynamic)",
                Self::Unknown => "<unknown>",
            }
        )
    }
}

#[derive(Copy, Clone)]
pub struct UUID([u8; 16]);

impl PartialEq for UUID {
    fn eq(&self, other: &Self) -> bool {
        for i in 0..self.0.len() {
            if self.0[i] != other.0[i] {
                return false;
            }
        }

        true
    }
}

impl Eq for UUID {}

impl Debug for UUID {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self}")
    }
}

impl Display for UUID {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "{:08x}-{:04x}-{:04x}-{:04x}-{:08x}{:04x}",
            read_u32_be(&self.0, 0),
            read_u16_be(&self.0, 4),
            read_u16_be(&self.0, 6),
            read_u16_be(&self.0, 8),
            read_u32_be(&self.0, 10),
            read_u16_be(&self.0, 14)
        )
    }
}

#[derive(Copy, Clone)]
pub struct VolumeLabel([u8; 16]);

impl VolumeLabel {
    pub fn as_str(&self) -> &str {
        let mut end_pos = 0;

        for i in 0..self.0.len() {
            if self.0[i] == 0 {
                break;
            }
            end_pos += 1;
        }

        unsafe { core::str::from_utf8_unchecked(&self.0[0..end_pos]) }
    }
}

impl Debug for VolumeLabel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Copy, Clone)]
pub struct MountDirectory([u8; 64]);

impl MountDirectory {
    pub fn as_str(&self) -> &str {
        let mut end_pos = 0;

        for i in 0..self.0.len() {
            if self.0[i] == 0 {
                break;
            }
            end_pos += 1;
        }

        unsafe { core::str::from_utf8_unchecked(&self.0[0..end_pos]) }
    }

    pub fn is_empty(&self) -> bool {
        self.0[0] == 0
    }
}

impl Debug for MountDirectory {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Copy, Clone)]
pub struct HashSeed([u32; 4]);

#[derive(Debug, Copy, Clone)]
pub struct JournalInodesBackup([u32; 17]);

#[derive(Copy, Clone)]
pub struct FunctionName([u8; 32]);

impl FunctionName {
    pub fn as_str(&self) -> &str {
        let mut end_pos = 0;

        for i in 0..self.0.len() {
            if self.0[i] == 0 {
                break;
            }
            end_pos += 1;
        }

        unsafe { core::str::from_utf8_unchecked(&self.0[0..end_pos]) }
    }
}

impl Debug for FunctionName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Copy, Clone)]
pub struct MountOptions([u8; 64]);

impl MountOptions {
    pub fn as_str(&self) -> &str {
        let mut end_pos = 0;

        for i in 0..self.0.len() {
            if self.0[i] == 0 {
                break;
            }
            end_pos += 1;
        }

        unsafe { core::str::from_utf8_unchecked(&self.0[0..end_pos]) }
    }
}

impl Debug for MountOptions {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BackupBlockGroups([u32; 2]);

#[derive(Debug, Clone, Copy)]
pub struct EncryptionAlgorithms([u8; 4]);

#[derive(Debug, Clone, Copy)]
pub struct EncryptionSalt([u8; 16]);

#[derive(Debug, Copy, Clone)]
pub struct SuperBlock {
    /// offset 0x0
    inodes_count: u32,
    // offset 0x004 for lo bytes
    // offset 0x150 for hi bytes
    blocks_count: u64,
    // offset 0x008 for lo bytes
    // offset 0x154 for hi bytes
    reserved_blocks_count: u64,
    // offset 0x00c for lo bytes
    // offset 0x158 for hi bytes
    free_blocks_count: u64,
    // offset 0x10
    free_inodes_count: u32,
    // offset 0x14
    first_data_block: u32,
    // offset 0x18
    log_block_size: u32,
    // offset 0x1c
    log_cluster_size: u32,
    // offset 0x20
    blocks_per_group: u32,
    // offset 0x24
    clusters_per_group: u32,
    // offset 0x28
    inodes_per_group: u32,
    // offset 0x02c for lo bytes
    // offset 0x275 for hi byte
    mount_time: u64,
    // offset 0x030 for lo bytes
    // offset 0x274 for hi byte
    write_time: u64,
    // offset 0x34
    mount_count: u16,
    // offset 0x36
    max_mount_count: i16,
    // offset 0x38
    // needs to be 0xEF53
    magic: u16,
    // offset 0x3a
    state: FileSystemState,
    // offset 0x3c
    error_policy: SuperBlockErrorPolicy,
    // offset 0x3e
    minor_revision_level: u16,
    // offset 0x040 for lo bytes
    // offset 0x277 for hi byte
    last_check_time: u64,
    // offset 0x44
    check_interval: u32,
    // offset 0x48
    creator_os: FileSystemCreatorOS,
    // offset 0x4c
    revision_level: Revision,
    // offset 0x50
    default_reserved_uid: u16,
    // offset 0x52
    default_reserved_gid: u16,
    // begin ext4 v2 data
    // offset 0x54
    first_inode: u32,
    // offset 0x58
    inode_size: u16,
    // offset 0x5a
    block_group_number: u16,
    // offset 0x5c
    compatible_features: u32,
    // offset 0x60
    incompatible_features: u32,
    // offset 0x64
    read_only_compatible_features: u32,
    // offset 0x68
    uuid: UUID,
    // offset 0x78
    volume_label: VolumeLabel,
    // offset 0x88
    last_mounted: Option<MountDirectory>,
    // offset 0xc8
    algorithm_usage_bitmap: u32,
    // offset 0xcc
    prealloc_blocks: u8,
    // offset 0xcd
    prealloc_dir_blocks: u8,
    // offset 0xce
    reserved_gdt_blocks: u16,
    // offset 0xd0
    journal_uuid: UUID,
    // offset 0xe0
    journal_inode_number: u32,
    // offset 0xe4
    journal_device: u32,
    // offset 0xe8
    last_orphan: u32,
    // offset 0xec
    hash_seed: HashSeed,
    // offset 0xfc
    default_hash_version: u8,
    // offset 0xfd
    journal_backup_type: u8,
    // offset 0xfe
    group_descriptor_size: u16,
    // offset 0x100
    default_mount_options: u32,
    // offset 0x104
    first_meta_block_group: u32,
    // offset 0x108 for lo bytes
    // offset 0x276 for hi byte
    created_time: u64,
    // offset 0x10c
    journal_inodes_backup: JournalInodesBackup,
    // offset 0x15c
    inode_min_size: u16,
    // offset 0x15e
    inode_new_recommended_size: u16,
    // offset 0x160
    misc_flags: u32,
    // offset 0x164
    raid_stride: u16,
    // offset 0x166
    mmp_interval: u16,
    // offset 0x168
    mmp_block: u64,
    // offset 0x170
    raid_stripe_width: u32,
    // offset 0x174
    log_groups_per_flex: u8,
    // offset 0x175
    checksum_type: u8,
    // offset 0x178
    lifetime_kb_written: u64,
    // offset 0x180
    snapshot_inode_number: u32,
    // offset 0x184
    snapshot_id: u32,
    // offset 0x188
    snapshot_future_blocks: u64,
    // offset 0x190
    snapshot_list_inode_number: u32,
    // offset 0x194
    error_count: u32,
    // offset 0x198 for lo bytes
    // offset 0x278 for hi byte
    first_error_time: u64,
    // offset 0x19c
    first_error_inode: u32,
    // offset 0x1a0
    first_error_block: u64,
    // offset 0x1a8
    first_error_function: FunctionName,
    // offset 0x1c8
    first_error_line: u32,
    // offset 0x1cc for lo bytes
    // offset 0x279 for hi byte
    last_error_time: u64,
    // offset 0x1d0
    last_error_inode: u32,
    // offset 0x1d4
    last_error_line: u32,
    // offset 0x1d8
    last_error_block: u64,
    // offset 0x1e0
    last_error_function: FunctionName,
    // offset 0x200
    mount_options: MountOptions,
    // offset 0x240
    user_quota_inode_number: u32,
    // offset 0x244
    group_quota_inode_number: u32,
    // offset 0x248
    overhead_blocks: u32,
    // offset 0x24c
    backup_block_groups: BackupBlockGroups,
    // offset 0x254
    encryption_algorithms: EncryptionAlgorithms,
    // offset 0x258
    encryption_salt: EncryptionSalt,
    // offset 0x268
    lost_and_found_inode_number: u32,
    // offset 0x26c
    project_quotas_inode_number: u32,
    // offset 0x270
    checksum_seed: u32,
    // offset 0x27c
    filename_encoding: u16,
    // offset 0x27e
    filename_encoding_flags: u16,
    // offset 0x280
    orphan_file_inode_number: u32,
    // offset 0x3fc
    checksum: u32,
}

impl SuperBlock {
    const SUPER_BLOCK_SIZE: usize = 1024;

    pub fn read(buffer: &[u8]) -> Result<Self, Error> {
        if buffer.len() < Self::SUPER_BLOCK_SIZE {
            return Err(Error::BufferSizeTooSmall(buffer.len() as u32));
        }

        Ok(Self {
            inodes_count: read_u32_le(buffer, 0x00),
            blocks_count: read_u32_le(buffer, 0x004) as u64 // lo bytes
                        | ((read_u32_le(buffer, 0x150) as u64) << 32), // hi bytes
            reserved_blocks_count: read_u32_le(buffer, 0x008) as u64 // lo bytes
                                 | ((read_u32_le(buffer, 0x154) as u64) << 32), // hi bytes
            free_blocks_count: read_u32_le(buffer, 0x0c) as u64 // lo bytes
                             | ((read_u32_le(buffer, 0x158) as u64) << 32), // hi bytes
            free_inodes_count: read_u32_le(buffer, 0x10),
            first_data_block: read_u32_le(buffer, 0x14),
            log_block_size: read_u32_le(buffer, 0x18),
            log_cluster_size: read_u32_le(buffer, 0x1c),
            blocks_per_group: read_u32_le(buffer, 0x20),
            clusters_per_group: read_u32_le(buffer, 0x24),
            inodes_per_group: read_u32_le(buffer, 0x28),
            mount_time: read_u32_le(buffer, 0x2c) as u64 // lo bytes
                      | ((buffer[0x275] as u64) << 32), // hi byte
            write_time: read_u32_le(buffer, 0x30) as u64 // lo bytes
                      | ((buffer[0x274] as u64) << 32), // hi byte
            mount_count: read_u16_le(buffer, 0x34),
            max_mount_count: read_i16_le(buffer, 0x36),
            magic: read_u16_le(buffer, 0x38),
            state: FileSystemState::from(read_u16_le(buffer, 0x38)),
            error_policy: SuperBlockErrorPolicy::from(read_u16_le(buffer, 0x3c)),
            minor_revision_level: read_u16_le(buffer, 0x3e),
            last_check_time: read_u32_le(buffer, 0x40) as u64 // lo bytes
                           | ((buffer[0x277] as u64) << 32), // hi byte
            check_interval: read_u32_le(buffer, 0x44),
            creator_os: FileSystemCreatorOS::from(read_u32_le(buffer, 0x48)),
            revision_level: Revision::from(read_u32_le(buffer, 0x4c)),
            default_reserved_uid: read_u16_le(buffer, 0x50),
            default_reserved_gid: read_u16_le(buffer, 0x52),
            first_inode: read_u32_le(buffer, 0x54),
            inode_size: read_u16_le(buffer, 0x58),
            block_group_number: read_u16_le(buffer, 0x5a),
            compatible_features: read_u32_le(buffer, 0x5c),
            incompatible_features: read_u32_le(buffer, 0x60),
            read_only_compatible_features: read_u32_le(buffer, 0x64),
            uuid: read_uuid(buffer, 0x68),
            volume_label: read_label(buffer, 0x78),
            last_mounted: read_mount_directory(buffer, 0x88),
            algorithm_usage_bitmap: read_u32_le(buffer, 0xc8),
            prealloc_blocks: buffer[0xcc],
            prealloc_dir_blocks: buffer[0xcd],
            reserved_gdt_blocks: read_u16_le(buffer, 0xce),
            journal_uuid: read_uuid(buffer, 0xd0),
            journal_inode_number: read_u32_le(buffer, 0xe0),
            journal_device: read_u32_le(buffer, 0xe4),
            last_orphan: read_u32_le(buffer, 0xe8),
            hash_seed: read_hash_seed(buffer, 0xec),
            default_hash_version: buffer[0xfc],
            journal_backup_type: buffer[0xfd],
            group_descriptor_size: read_u16_le(buffer, 0xfe),
            default_mount_options: read_u32_le(buffer, 0x100),
            first_meta_block_group: read_u32_le(buffer, 0x104),
            created_time: read_u32_le(buffer, 0x108) as u64 // lo bytes
                        | ((buffer[0x276] as u64) << 32), // hi byte
            journal_inodes_backup: read_journal_inodes_backup(buffer, 0x10c),
            inode_min_size: read_u16_le(buffer, 0x15c),
            inode_new_recommended_size: read_u16_le(buffer, 0x15e),
            misc_flags: read_u32_le(buffer, 0x160),
            raid_stride: read_u16_le(buffer, 0x164),
            mmp_interval: read_u16_le(buffer, 0x166),
            mmp_block: read_u64_le(buffer, 0x168),
            raid_stripe_width: read_u32_le(buffer, 0x170),
            log_groups_per_flex: buffer[0x174],
            checksum_type: buffer[0x175],
            lifetime_kb_written: read_u64_le(buffer, 0x178),
            snapshot_inode_number: read_u32_le(buffer, 0x180),
            snapshot_id: read_u32_le(buffer, 0x184),
            snapshot_future_blocks: read_u64_le(buffer, 0x188),
            snapshot_list_inode_number: read_u32_le(buffer, 0x190),
            error_count: read_u32_le(buffer, 0x194),
            first_error_time: read_u32_le(buffer, 0x198) as u64 // lo bytes
                            | ((buffer[0x278] as u64) << 32), // hi byte
            first_error_inode: read_u32_le(buffer, 0x19c),
            first_error_block: read_u64_le(buffer, 0x1a0),
            first_error_function: read_function_name(buffer, 0x1a8),
            first_error_line: read_u32_le(buffer, 0x1c8),
            last_error_time: read_u32_le(buffer, 0x1cc) as u64 // lo bytes
                           | ((buffer[0x279] as u64) << 32), // hi byte
            last_error_inode: read_u32_le(buffer, 0x1d0),
            last_error_line: read_u32_le(buffer, 0x1d4),
            last_error_block: read_u64_le(buffer, 0x1d8),
            last_error_function: read_function_name(buffer, 0x1e0),
            mount_options: read_mount_options(buffer, 0x200),
            user_quota_inode_number: read_u32_le(buffer, 0x240),
            group_quota_inode_number: read_u32_le(buffer, 0x244),
            overhead_blocks: read_u32_le(buffer, 0x248),
            backup_block_groups: read_backup_block_groups(buffer, 0x24c),
            encryption_algorithms: read_encryption_algorithms(buffer, 0x254),
            encryption_salt: read_encryption_salt(buffer, 0x258),
            lost_and_found_inode_number: read_u32_le(buffer, 0x268),
            project_quotas_inode_number: read_u32_le(buffer, 0x26c),
            checksum_seed: read_u32_le(buffer, 0x270),
            filename_encoding: read_u16_le(buffer, 0x27c),
            filename_encoding_flags: read_u16_le(buffer, 0x27e),
            orphan_file_inode_number: read_u32_le(buffer, 0x280),
            checksum: read_u32_le(buffer, 0x3fc),
        })
    }

    pub fn magic(&self) -> u16 {
        self.magic
    }

    pub fn volume_label(&self) -> &str {
        self.volume_label.as_str()
    }

    pub fn filesystem_uuid(&self) -> &UUID {
        &self.uuid
    }

    pub fn last_mounted(&self) -> Option<&str> {
        self.last_mounted.as_ref().map(|o| o.as_str())
    }

    pub fn filesystem_revision(&self) -> Revision {
        self.revision_level
    }

    pub fn creator_os(&self) -> FileSystemCreatorOS {
        self.creator_os
    }

    pub fn filesystem_state(&self) -> FileSystemState {
        self.state
    }

    pub fn group_descriptor_size(&self) -> u16 {
        self.group_descriptor_size
    }

    pub fn inode_size(&self) -> u16 {
        self.inode_size
    }
}

fn read_uuid(input: &[u8], offset: usize) -> UUID {
    let mut buffer: [u8; 16] = [0; 16];
    buffer.copy_from_slice(&input[offset..offset + 16]);
    UUID(buffer)
}

fn read_label(input: &[u8], offset: usize) -> VolumeLabel {
    let mut buffer: [u8; 16] = [0; 16];
    buffer.copy_from_slice(&input[offset..offset + 16]);
    VolumeLabel(buffer)
}

fn read_mount_directory(input: &[u8], offset: usize) -> Option<MountDirectory> {
    if input[offset] == 0 {
        return None;
    }

    let mut buffer: [u8; 64] = [0; 64];
    buffer.copy_from_slice(&input[offset..offset + 64]);

    Some(MountDirectory(buffer))
}

fn read_function_name(input: &[u8], offset: usize) -> FunctionName {
    let mut buffer: [u8; 32] = [0; 32];
    buffer.copy_from_slice(&input[offset..offset + 32]);
    FunctionName(buffer)
}

fn read_mount_options(input: &[u8], offset: usize) -> MountOptions {
    let mut buffer: [u8; 64] = [0; 64];
    buffer.copy_from_slice(&input[offset..offset + 64]);
    MountOptions(buffer)
}

fn read_hash_seed(input: &[u8], offset: usize) -> HashSeed {
    let mut numbers: [u32; 4] = [0; 4];

    for i in 0..numbers.len() {
        numbers[i] = read_u32_le(input, offset + i * 4);
    }

    HashSeed(numbers)
}

fn read_journal_inodes_backup(input: &[u8], offset: usize) -> JournalInodesBackup {
    let mut numbers: [u32; 17] = [0; 17];

    for i in 0..numbers.len() {
        numbers[i] = read_u32_le(input, offset + i * 4);
    }

    JournalInodesBackup(numbers)
}

fn read_backup_block_groups(input: &[u8], offset: usize) -> BackupBlockGroups {
    let mut numbers: [u32; 2] = [0; 2];

    for i in 0..numbers.len() {
        numbers[i] = read_u32_le(input, offset + i * 4);
    }

    BackupBlockGroups(numbers)
}

fn read_encryption_algorithms(input: &[u8], offset: usize) -> EncryptionAlgorithms {
    let mut buffer: [u8; 4] = [0; 4];
    buffer.copy_from_slice(&input[offset..offset + 4]);
    EncryptionAlgorithms(buffer)
}

fn read_encryption_salt(input: &[u8], offset: usize) -> EncryptionSalt {
    let mut buffer: [u8; 16] = [0; 16];
    buffer.copy_from_slice(&input[offset..offset + 16]);
    EncryptionSalt(buffer)
}
