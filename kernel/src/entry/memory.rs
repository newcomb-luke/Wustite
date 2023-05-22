#[derive(Clone, Copy)]
pub struct MemoryRegion {
    pub start: u64,
    pub end: u64,
}

impl MemoryRegion {
    pub const fn zero() -> Self {
        Self { start: 0, end: 0 }
    }

    pub fn read_from(region: *const u64) -> Self {
        unsafe {
            Self {
                start: region.read(),
                end: region.add(1).read(),
            }
        }
    }
}
