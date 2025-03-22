#![no_std]
#![no_main]

use common::println;
use core::{arch::asm, ptr};
use flat_device_tree::Fdt;

mod sbi;

unsafe extern "C" {
    static mut __bss: u64;
    static __bss_end: u64;
    static __stack_top: u64;
}

#[unsafe(no_mangle)]
fn kernel_main(_hartid: usize, dtb_pa: usize) {
    let bss = ptr::addr_of_mut!(__bss);
    let bss_end = ptr::addr_of!(__bss_end);
    unsafe {
        ptr::write_bytes(bss, 0, bss_end as usize - bss as usize);
    }
    init_dt(dtb_pa);

    loop {}
}

fn init_dt(dtb: usize) {
    let fdt = unsafe { Fdt::from_ptr(dtb as *const u8).unwrap() };
    println!(
        "This is a devicetree representation of a {}",
        fdt.root().unwrap().model()
    );
    println!(
        "...which is compatible with at least: {}",
        fdt.root().unwrap().compatible().first().unwrap()
    );
    println!("...and has {} CPU(s)", fdt.cpus().count());
    println!(
        "...and has at least one memory location at: {:#X}\n",
        fdt.memory()
            .unwrap()
            .regions()
            .next()
            .unwrap()
            .starting_address as usize
    );
}

#[unsafe(link_section = ".text.boot")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn _entry() {
    unsafe {
        asm!(
            "la sp, {stack_top}",
            // a0: hartid, a1: device tree blobの物理アドレス
            "mv a0, a0",
            "mv a1, a1",
            "j kernel_main",
            stack_top = sym __stack_top,
        )
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
