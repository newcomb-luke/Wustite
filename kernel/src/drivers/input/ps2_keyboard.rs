use alloc::collections::VecDeque;
use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1, layouts};
use spin::{Mutex, Once};

use crate::{
    drivers::{DriverResult, read_io_port_u8},
    interrupts::{GSI, IrqResult, LogicalIrq},
    log,
    resource::{request_irq, request_port},
};

const SCANCODE_PORT: u16 = 0x60;
const IRQ_NUMBER: u8 = 1;

struct PS2KeyboardInner {
    keyboard: Keyboard<layouts::Us104Key, ScancodeSet1>,
}

pub struct PS2KeyboardDriver {
    inner: Mutex<Once<PS2KeyboardInner>>,
}

impl PS2KeyboardDriver {
    const fn new() -> Self {
        Self {
            inner: Mutex::new(Once::new()),
        }
    }

    pub fn init(&'static mut self) -> DriverResult {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let inner = self.inner.lock();

            if inner.is_completed() {
                panic!("Attempted to initialize PS/2 keyboard driver twice");
            }

            request_port(SCANCODE_PORT).map_err(|_| ())?;

            request_irq(GSI::from_u8(IRQ_NUMBER), self, Self::handle_interrupt)?;

            inner.call_once(|| PS2KeyboardInner {
                keyboard: Keyboard::new(
                    ScancodeSet1::new(),
                    layouts::Us104Key,
                    HandleControl::Ignore,
                ),
            });

            Ok(())
        })
    }

    extern "C" fn handle_interrupt(&'static self, _irq: LogicalIrq) -> IrqResult {
        let mut inner = self.inner.lock();
        let keyboard_inner = inner.get_mut().unwrap();
        let keyboard = &mut keyboard_inner.keyboard;

        let scancode = unsafe { read_io_port_u8(SCANCODE_PORT) };

        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(_character) => {
                        log!("{}", _character);
                        // KEYBOARD_BUFFER.put_char(character);
                    }
                    _ => {} // DecodedKey::RawKey(key) => kprint!("{:?}", key),
                }
            }
        }

        IrqResult::Handled
    }
}

pub const BACKSPACE: u8 = 8;

pub static KEYBOARD_BUFFER: KeyboardBuffer = KeyboardBuffer::new();

struct KeyboardBufferInner {
    buffer: VecDeque<char>,
}

impl KeyboardBufferInner {
    fn new() -> Self {
        Self {
            buffer: VecDeque::new(),
        }
    }

    fn get_char(&mut self) -> Option<char> {
        self.buffer.pop_front()
    }

    fn put_char(&mut self, c: char) {
        self.buffer.push_back(c);
    }
}

pub struct KeyboardBuffer {
    inner: Mutex<Option<KeyboardBufferInner>>,
}

impl KeyboardBuffer {
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }

    pub fn init(&self) {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut inner = self.inner.lock();
            *inner = Some(KeyboardBufferInner::new());
        });
    }

    pub fn get_char(&self) -> Option<char> {
        x86_64::instructions::interrupts::without_interrupts(|| {
            let mut inner = self.inner.lock();
            if let Some(buffer) = inner.as_mut() {
                buffer.get_char()
            } else {
                None
            }
        })
    }

    pub fn put_char(&self, c: char) {
        x86_64::instructions::interrupts::without_interrupts(|| {
            if let Some(buffer) = self.inner.lock().as_mut() {
                buffer.put_char(c);
            }
        })
    }
}

pub fn handle_keyboard_interrupt() {}
