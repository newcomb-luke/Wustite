use alloc::collections::VecDeque;
use lazy_static::lazy_static;
use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1, layouts};
use spin::Mutex;
use x86_64::instructions::port::Port;

const SCANCODE_PORT: u16 = 0x60;

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
        Mutex::new(Keyboard::new(
            ScancodeSet1::new(),
            layouts::Us104Key,
            HandleControl::Ignore
        ));
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

pub fn handle_keyboard_interrupt() {
    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(SCANCODE_PORT);

    let scancode: u8 = unsafe { port.read() };

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(_character) => {
                    // KEYBOARD_BUFFER.put_char(character);
                }
                _ => {} // DecodedKey::RawKey(key) => kprint!("{:?}", key),
            }
        }
    }
}
