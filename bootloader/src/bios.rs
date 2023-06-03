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
        "stc",
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

#[inline(never)]
#[link_section = ".bios"]
/// Returns 0 if successful, nonzero otherwise
pub unsafe fn bios_drive_reset(drive_number: u8) -> i32 {
    let success: i32;

    enter_16_bit_real_mode!();

    asm!(
        ".code16",
        // "reset"
        "mov ah, 0x00",
        // Drive number
        "mov dl, {0}",
        "stc",
        "int 0x13",
        "jc 2f",
        // Success
        "mov {1:e}, 0",
        "jmp 3f",
        // Failure
        "2:",
        "mov {1:e}, 1",
        // Done
        "3:",
        in(reg_byte) drive_number,
        out(reg) success,
        out("edx") _,
    );

    enter_32_bit_protected_mode!();

    success
}

#[inline(never)]
#[link_section = ".bios"]
/// Returns 0 if successful, nonzero otherwise
pub unsafe fn bios_drive_get_params(drive_number: u8, buffer: *mut u8) -> i32 {
    let success: i32;
    let drive_type: u8;
    let ch: u8;
    let cl: u8;
    let max_head: u8;

    enter_16_bit_real_mode!();

    asm!(
        ".code16",
        // Guard against BIOS bugs: http://www.ctyme.com/intr/rb-0621.htm
        // Set ES:DI = 0x0000:0x0000
        "push es",
        "xor di, di",
        "mov es, di",
        "mov ah, 0x08",
        "mov dl, {0}",
        "stc",
        "int 0x13",
        "pop es",
        "jc 2f",
        // Success
        "xor eax, eax",
        "jmp 3f",
        // Failure
        "2: mov eax, 1",
        // Done
        "3:",
        in(reg_byte) drive_number,
        lateout("eax") success,
        lateout("bl") drive_type,
        lateout("ch") ch,
        lateout("cl") cl,
        out("dh") max_head,
        lateout("dl") _,
        out("edi") _,
    );

    enter_32_bit_protected_mode!();

    // Only do this if we have succeeded
    if success == 0 {
        // Buffer layout:
        //   u8: drive type
        //   u8: max head number
        //   u16: max cylinder number
        //   u8: max sector number

        // Store the drive type
        buffer.write(drive_type);

        // Store the max head number
        buffer.add(1).write(max_head);

        // Calculate and store the max cylinder number
        let mut max_cylinder = 0;
        max_cylinder = (ch as u16) >> 8;
        // Top 2 bits of the max cylinder number are in cl
        max_cylinder |= ((cl as u16) & 0b11000000) >> 6;

        // Let's do this the safe way
        let mut max_cylinder_bytes = max_cylinder.to_ne_bytes();
        buffer.copy_from(max_cylinder_bytes.as_ptr(), 2);

        // Get and store the max sector number
        let max_sector = cl & 0b00111111;
        buffer.add(4).write(max_sector);
    }

    success
}

// ; extern "cdecl" {
// ;   _BIOS_Drive_GetParams(drive_number: u8,
// ;                         buffer: *mut u8) -> u16;
// ; }
// ;
// _BIOS_Drive_GetParams:
//     [bits 32]
//     push ebp
//     mov ebp, esp
//     push ebx
//     push esi
//
//     x86_EnterRealMode
//     [bits 16]
//
//     ; [bp + 8] - drive number
//     ; [bp + 12] - pointer to buffer
//
//     ; buffer layout:
//     ;  u8: drive type
//     ;  u8: max head number
//     ;  u16: max cylinder number
//     ;  u8: max sector number
//
//     push es
//     push di
//     ; ES:DI = 0x0000:0x0000
//     xor di, di
//     mov es, di
//     mov ah, 0x08
//     mov dl, [bp + 8]
//     stc
//     int 0x13
//     pop di
//     pop es
//
//     jc .fail_real_mode
//
// .success_real_mode:
//     mov word [real_mode_error], 0
//     jmp .done_real_mode
//
// .fail_real_mode:
//     mov word [real_mode_error], 1
//
// .done_real_mode:
//
//     x86_EnterProtectedMode
//     [bits 32]
//
//     mov ax, [real_mode_error]
//     and ax, ax
//     jnz .fail
//
//     ; BL = drive type (AT/PS2 floppies only) (see #00242)
//     ; CH = low eight bits of maximum cylinder number
//     ; CL = maximum sector number (bits 5-0)
//     ; high two bits of maximum cylinder number (bits 7-6)
//     ; DH = maximum head number
//     ; DL = number of drives
//
//     ; Store the drive type
//     mov esi, [ebp + 12]
//     mov [esi + 0], bl
//
//     ; Store the max head number
//     mov esi, [ebp + 12]
//     mov [esi + 1], dh
//
//     ; Store the max cylinder number
//     mov bl, ch
//     mov bh, cl
//     shr bh, 6
//     mov esi, [ebp + 12]
//     mov [esi + 2], bx
//
//     ; Store the max sector number
//     and cl, 0b00111111 ; sector number is in bits 5-0
//     mov esi, [ebp + 12]
//     mov [esi + 4], cl
//
//     xor eax, eax ; success
//     jmp .done
//
// .fail:
//     mov eax, 1
//
// .done:
//     pop esi
//     pop ebx
//     mov esp, ebp
//     pop ebp
//     ret
