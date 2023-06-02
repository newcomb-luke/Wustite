use core::arch::asm;

#[link(name = "longmode")]
extern "cdecl" {
    fn long_mode_jump(entry_point: u32);
}

pub fn enter_long_mode(entry_point: u64) -> ! {
    unsafe {
        long_mode_jump(entry_point as u32);
    }

    loop {
        unsafe {
            asm!("cli");
        }
    }
}

pub fn is_cpuid_available() -> bool {
    let value: u32;

    unsafe {
        asm!(
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
            "4: nop",
            out("ecx") _,
            lateout("eax") value,
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
            out("ebx") _,
            out("ecx") _,
            out("edx") _,
            lateout("eax") value
        );
    }

    value == 0
}
