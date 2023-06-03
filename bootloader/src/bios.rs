use core::arch::asm;

macro_rules! enter_16_bit_real_mode {
    () => {
        // Why we have to use AT&T syntax:
        // https://www.reddit.com/r/rust/comments/o8lrz8/how_do_i_get_a_far_absolute_jump_with_inline/

        // Clear interrupts and enter 16-bit protected mode segment
        asm!(
            ".code32",
            "cli",
            "ljmp $0x18, $2f",
            "2:",
            options(att_syntax)
        );

        // Disable protected mode bit from cr0
        asm!(
            ".code16",
            "mov eax, cr0",
            "and al, ~1",
            "mov cr0, eax",
            out("eax") _
        );

        // Enter 16-bit real mode segment
        asm!(".code16", "ljmp $0x00, $2f", "2:", options(att_syntax));

        // Set up real mode segments and re-enable interrupts
        asm!(
            ".code16",
            "xor ax, ax",
            "mov ds, ax",
            "mov ss, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "sti",
            out("eax") _
        );
    };
}

macro_rules! enter_32_bit_protected_mode {
    () => {
        asm!(
            ".code16",
            "cli",
            "mov eax, cr0",
            "or al, 1",
            "mov cr0, eax",
            out("eax") _
        );

        asm!(".code16", "ljmp $0x08, $2f", "2: ", options(att_syntax));

        asm!(
            ".code32",
            "mov ax, 0x10",
            "mov ds, ax",
            "mov ss, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
        );
    };
}

// It breaks for some reason if we don't do inline(never) :)
#[inline(never)]
// We also add this to the special .bios link section that goes directly after
// the entry point. We do that to allow us to not have to worry about segment
// registers for BIOS calls. The segment will always be 0 because where we are
// will always correspond with the stack segment, if we are low enough
#[link_section = ".bios"]
pub unsafe fn bios_get_next_segment(entry: *mut u8, continuation_id: *mut u32) -> i32 {
    let bytes_read: i32;

    enter_16_bit_real_mode!();

    asm!(
        ".code16",
        "mov ebx, [{1:e}]",
        // A magic number that says, yes, we want this data
        "mov edx, 0x534D4150",
        // Destination address for BIOS to put the data
        "mov edi, {0:e}",
        "mov ecx, 24",
        "mov eax, 0x0000E820",
        "int 0x15",
        "jc 2f",
        // On success, eax is reset to "SMAP" (the magic number)
        "mov edx, 0x534D4150",
        "cmp eax, edx",
        "jne 2f",
        // ecx contains the number of bytes actually read
        "mov eax, ecx",
        // Set our continuation id value
        "mov [{1:e}], ebx",
        "jmp 3f",
        "2: mov eax, -1",
        "3: ",
        in(reg) entry,
        in(reg) continuation_id,
        lateout("eax") bytes_read,
        out("ebx") _,
        out("ecx") _,
        out("edx") _
    );

    enter_32_bit_protected_mode!();

    bytes_read
}

#[inline(never)]
#[link_section = ".bios"]
pub unsafe fn bios_write_char_teletype(c: u8) {
    enter_16_bit_real_mode!();

    asm!(
        ".code16",
        "mov ecx, ebp",
        "mov ah, 0x0e",
        "mov al, {}",
        "xor bx, bx",
        "int 0x10",
        "mov ebp, ecx",
        in(reg_byte) c,
        out("eax") _,
        out("ebx") _,
        // Apparently some BIOSes can clobber ebp, so we use this as an intermediate
        out("ecx") _
    );

    enter_32_bit_protected_mode!();
}

// ; args: character
// _BIOS_Video_WriteCharTeletype:
//     [bits 32]
// 	push ebp
// 	mov ebp, esp
// 	push eax
//     push ebx
//
//     mov bl, [ebp + 8]
//
// 	x86_EnterRealMode
// 	[bits 16]
//
// 	; [bp + 8] - character
//
// 	mov ah, 0x0e
// 	mov al, bl
// 	xor bx, bx
// 	int 0x10
//
// 	x86_EnterProtectedMode
// 	[bits 32]
//
//     pop ebx
//     pop eax
// 	mov esp, ebp
// 	pop ebp
// 	ret
