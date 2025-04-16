use core::{fmt::Display, sync::atomic::AtomicU64};

pub mod timer;

static SYSTEM_CLOCK_NANOS: AtomicU64 = AtomicU64::new(0);

const NANOSECONDS_PER_SECOND: u64 = 1_000_000_000;

pub fn get_system_clock() -> u64 {
    SYSTEM_CLOCK_NANOS.load(core::sync::atomic::Ordering::SeqCst)
}

pub fn get_system_time() -> Time {
    let clock = get_system_clock();
    Time::from_timestamp(clock)
}

pub fn increment_system_clock(value: u64) {
    SYSTEM_CLOCK_NANOS.fetch_add(value, core::sync::atomic::Ordering::SeqCst);
}

pub struct Time {
    seconds: u32,
    nanoseconds: u32,
}

impl Time {
    pub fn new(seconds: u32, nanoseconds: u32) -> Self {
        Self {
            seconds,
            nanoseconds,
        }
    }

    pub fn from_timestamp(timestamp: u64) -> Self {
        let seconds = (timestamp / NANOSECONDS_PER_SECOND) as u32;
        let nanoseconds = (timestamp % NANOSECONDS_PER_SECOND) as u32;

        Self {
            seconds,
            nanoseconds,
        }
    }

    pub fn seconds(&self) -> u32 {
        self.seconds
    }

    pub fn nanoseconds(&self) -> u32 {
        self.nanoseconds
    }
}
