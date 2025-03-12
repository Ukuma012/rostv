#![no_std]
#![no_main]

use core::{arch::asm, ptr};

mod csr;

unsafe extern "C" {
    static mut __bss: u64;
    static __bss_end: u64;
    static __stack_top: u64;
}

#[unsafe(no_mangle)]
fn kernel_main() {
    let bss = ptr::addr_of_mut!(__bss);
    let bss_end = ptr::addr_of!(__bss_end);
    unsafe {
        ptr::write_bytes(bss, 0, bss_end as usize - bss as usize);
    }
    let current_sp: u32;
    unsafe {
        asm!("mv {}, sp", out(reg) current_sp);
    }
    write_csr!("sscratch", current_sp);
    loop {}
}

#[unsafe(link_section = ".text.boot")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn _entry() {
    unsafe {
        asm!(
            "la sp, {stack_top}",
            "j kernel_main",
            stack_top = sym __stack_top,
        )
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
