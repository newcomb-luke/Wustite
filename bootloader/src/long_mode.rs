use core::arch::asm;

pub fn is_cpuid_available() -> bool {
    let value: u32;

    unsafe {
        asm!(
            "push ecx",
            "pushfd",
            "pop eax",
            "mov ecx, eax",
            // Flip "ID" bit to 1
            "xor eax, 0x200000",
            "push eax",
            "popfd",
            "pushfd",
            // These should not be the same, if they are
            // then our change didn't stick: We don't have
            // cpuid support.
            "pop eax",
            "cmp ecx, eax",
            "je 3f",
            "xor eax, eax",
            "jmp 4f",
            "3: mov eax, 1",
            "4: pop ecx",
            lateout("eax") value
        );
    }

    value == 0
}

pub fn is_extended_cpuid_available() -> bool {
    let value: u32;

    unsafe {
        asm!(
            "mov eax, 0x80000000",
            "cpuid",
            "cmp eax, 0x80000001",
            "jb 3f",
            "xor eax, eax",
            "jmp 4f",
            "3: mov eax, 1",
            "4: nop",
            lateout("eax") value
        );
    }

    value == 0
}
