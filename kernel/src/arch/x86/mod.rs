use core::arch::asm;

#[inline]
pub fn port_out(port: u16, data: u8) {
    unsafe {
        asm!(
        "mov dx, {0:x}", // mov dx, port
        "mov al, {1}",   // mov al, data
        "out dx, al",    // out dx, al
        in(reg_abcd) port,
        in(reg_byte) data,
        in("al") 0u8,
        in("dx") 0u16,
        );
    }
}

#[inline]
pub fn port_in(port: u16) -> u8 {
    let data;

    unsafe {
        asm!(
        "mov dx, {0:x}", // mov dx, port
        "in al, dx",    // out dx, al
        in(reg_abcd) port,
        out("al") data,
        in("dx") 0u16,
        );
    }

    data
}
