#![no_std]

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemError {
    ResourceNotFound,
    NoResourcesAvailable,
    ResourceInUse,
    ResourceInvalid
}