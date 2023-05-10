use core::arch::asm;

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
            // Odd megabyte address
            "mov edi, 0x112345",
            // Even megabyte address
            "mov esi, 0x012345",
            "mov [esi], esi",
            "mov [edi], edi",
            "mov ebx, [esi]",
            "mov edx, [edi]",
            "cmp ebx, edx",
            "jne 2f",
            "    mov {0:x}, 1",
            "    jmp 3f",
            "2:",
            "    xor {0:x}, {0:x}",
            "3: nop",
            out(reg) value
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
