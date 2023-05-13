use core::arch::asm;

const KEYBOARD_DATA_PORT: u8 = 0x60;
const KEYBOARD_STATUS_PORT: u8 = 0x64;
const KEYBOARD_CMD_PORT: u8 = 0x64;

const INPUT_BUFFER_STATUS_BIT: u8 = 2;

pub fn enable_a20() -> Result<(), ()> {
    // Is a20 already enabled?
    if test_a20() {
        return Ok(());
    }

    bios_method();

    if test_a20() {
        return Ok(());
    }

    // Other methods should be added

    Err(())
}

fn test_a20() -> bool {
    let value: u16;

    unsafe {
        asm!(
            "push edi",
            "push esi",
            "push ebx",
            "push edx",
            // Odd megabyte address
            "mov edi, 0x312345",
            // Even megabyte address
            "mov esi, 0x212345",
            "mov [esi], esi",
            "mov [edi], edi",
            "mov ebx, [esi]",
            "mov edx, [edi]",
            "cmp ebx, edx",
            "jne 2f",
            "    mov ax, 1",
            "    jmp 3f",
            "2:",
            "    xor ax, ax",
            "3: pop edx",
            "pop ebx",
            "pop esi",
            "pop edi",
            lateout("ax") value
        );
    }

    value == 0
}

fn bios_method() {
    // Uses a BIOS interrupt to enable a20
    unsafe {
        asm!("mov ax, 0x2401", "int 0x15");
    }
}

fn wait_controller_ready_for_input() {
    unsafe {
        asm!(
            "push ax",
            "2: in al, {0}",
            "test al, {1}",
            "jnz 2b",
            "pop ax",
            in(reg_byte) KEYBOARD_STATUS_PORT,
            in(reg_byte) INPUT_BUFFER_STATUS_BIT
        );
    }
}
