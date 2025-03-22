#![no_std]
#![no_main]

use common::println;
use core::{
    arch::asm,
    ptr::{self, NonNull},
};
use flat_device_tree::{node::FdtNode, Fdt};
use virtio::HalImpl;
use virtio_drivers::{
    device::gpu::VirtIOGpu,
    transport::{
        mmio::{MmioTransport, VirtIOHeader},
        DeviceType, Transport,
    },
};

extern crate alloc;
mod sbi;
mod virtio;

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
    println!("--- Device Tree Nodes ---");
    println!(
        "This is a devicetree representation of a {}",
        fdt.root().unwrap().model()
    );
    walk_dt(&fdt);
}

fn walk_dt(fdt: &Fdt) {
    for node in fdt.all_nodes() {
        if let Some(compatible) = node.compatible() {
            if compatible.all().any(|s| s == "virtio,mmio") {
                virtio_probe(node)
            }
        }
    }
}

fn virtio_probe(node: FdtNode) {
    if let Some(reg) = node.reg().next() {
        let paddr = reg.starting_address as usize;
        let size = reg.size.unwrap();
        let vaddr = paddr;
        let header = NonNull::new(vaddr as *mut VirtIOHeader).unwrap();
        match unsafe { MmioTransport::new(header, size) } {
            Err(_) => return,
            Ok(transport) => {
                println!(
                    "Detected virtio MMIO device with vendor id {:#X}, device type {:?}, version {:?}",
                    transport.vendor_id(),
                    transport.device_type(),
                    transport.version()
                );
                virtio_device(transport);
            }
        }
    }
}

fn virtio_device(transport: impl Transport) {
    match transport.device_type() {
        DeviceType::GPU => virtio_gpu(transport),
        t => println!("Unrecognized virtio device: {:?}", t),
    }
}

fn virtio_gpu<T: Transport>(transport: T) {
    let mut gpu = VirtIOGpu::<HalImpl, T>::new(transport).expect("failed to create gpu driver");
    let (width, height) = gpu.resolution().expect("failed to get resolution");
    let width = width as usize;
    let height = height as usize;
    log::info!("GPU resolution is {}x{}", width, height);
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
