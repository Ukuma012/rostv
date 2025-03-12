#![no_std]
#![no_main]

use core::arch::asm;

unsafe extern "C" {
    static mut __bss: u32;
    static __bss_end: u32;
    static __stack_top: u32;
}

#[unsafe(no_mangle)]
fn kernel_main() {
    loop {
        
    }
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
