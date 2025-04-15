use spin::mutex::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortError {
    PortInUse,
    PortOutOfBounds
}

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

    pub fn request_port(&self, port: u16) -> Result<(), PortError> {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let inner = self.mapping.lock();

            if let Some(in_use) = inner.get(port as usize) {
                if !in_use {
                    Ok(())
                } else {
                    Err(PortError::PortInUse)
                }
            } else {
                Err(PortError::PortOutOfBounds)
            }
        })
    }
}