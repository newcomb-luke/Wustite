use core::sync::atomic::AtomicU64;

mod timer;

static SYSTEM_CLOCK: AtomicU64 = AtomicU64::new(0);

pub fn get_system_clock() -> u64 {
    SYSTEM_CLOCK.load(core::sync::atomic::Ordering::SeqCst)
}
