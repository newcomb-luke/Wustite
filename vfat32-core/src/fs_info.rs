use bin_tools::read_u32_le;

const FS_INFO_FIRST_SIGNATURE: u32 = 0x41615252;
const FS_INFO_SECOND_SIGNATURE: u32 = 0x61417272;
const FS_INFO_END_SIGNATURE: u32 = 0xAA550000;

#[derive(Debug, Clone, Copy)]
pub struct FSInfo {
    /// offset 0x000
    first_signature: u32,
    /// offset 0x1E4
    second_signature: u32,
    /// offset 0x1E8
    last_free_cluster_count: u32,
    /// offset 0x1EC
    next_available_cluster: u32,
    /// offset 0x1FC
    end_signature: u32,
}

impl FSInfo {
    pub fn read(buffer: &[u8]) -> Self {
        Self {
            first_signature: read_u32_le(buffer, 0x000),
            second_signature: read_u32_le(buffer, 0x1E4),
            last_free_cluster_count: read_u32_le(buffer, 0x1E8),
            next_available_cluster: read_u32_le(buffer, 0x1EC),
            end_signature: read_u32_le(buffer, 0x1FC),
        }
    }

    pub fn is_valid(&self) -> bool {
        (self.first_signature == FS_INFO_FIRST_SIGNATURE)
            & (self.second_signature == FS_INFO_SECOND_SIGNATURE)
            & (self.end_signature == FS_INFO_END_SIGNATURE)
    }
}
