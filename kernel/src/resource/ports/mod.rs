use kernel::SystemError;
use spin::mutex::Mutex;

pub static PORTS_TABLE: PortsTable = PortsTable::new();

pub struct PortsTable {
    mapping: Mutex<[bool; 256]>
}

impl PortsTable {
    const fn new() -> Self {
        Self {
            mapping: Mutex::new([false; 256])
        }
    }

    pub fn request_port(&self, port: u16) -> Result<(), SystemError> {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let inner = self.mapping.lock();

            if let Some(in_use) = inner.get(port as usize) {
                if !in_use {
                    Ok(())
                } else {
                    Err(SystemError::ResourceInUse)
                }
            } else {
                Err(SystemError::ResourceInvalid)
            }
        })
    }
}