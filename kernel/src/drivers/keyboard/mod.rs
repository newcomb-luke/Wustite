use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use spin::Mutex;
use x86_64::instructions::port::Port;
use crate::kprint;
use lazy_static::lazy_static;

const SCANCODE_PORT: u16 = 0x60;

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
        Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1,
            HandleControl::Ignore)
        );
}

pub fn handle_keyboard_interrupt() {
    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(SCANCODE_PORT);

    let scancode: u8 = unsafe { port.read() };

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => kprint!("{}", character),
                DecodedKey::RawKey(key) => kprint!("{:?}", key),
            }
        }
    }
}