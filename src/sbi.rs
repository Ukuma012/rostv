use core::arch::asm;

struct Sbiret {
    error: usize,
    value: usize,
}

unsafe fn sbi_call(
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    fid: usize,
    eid: usize,
) -> Sbiret {
    let mut error;
    let mut value;
    unsafe {
        asm!(
            "ecall",
            inout("a0") arg0 => error,
            inout("a1") arg1 => value,
            in("a2") arg2,
            in("a3") arg3,
            in("a4") arg4,
            in("a5") arg5,
            in("a6") fid,
            in("a7") eid
        );
    }
    Sbiret { error, value }
}

#[unsafe(no_mangle)]
pub fn putchar(ch: u8) {
    unsafe {
        sbi_call(ch as usize, 0, 0, 0, 0, 0, 0, 1);
    }
}
