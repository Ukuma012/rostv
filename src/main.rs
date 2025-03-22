#![no_std]
#![no_main]
#![feature(unsafe_cell_access)]

use common::println;
use core::{
    alloc::{GlobalAlloc, Layout},
    arch::asm,
    cell::UnsafeCell,
    ptr::{self, NonNull},
    sync::atomic::{AtomicUsize, Ordering},
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
    static __free_ram: u64;
    static __free_ram_end: u64;
}

pub struct BumpAllocator {
    heap_start: AtomicUsize,
    heap_end: UnsafeCell<usize>,
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();

        let start = self.heap_start.load(Ordering::Relaxed);
        let aligned_start = (start + align - 1) & !(align - 1);

        let new_start = aligned_start + size;

        if new_start > *self.heap_end.as_ref_unchecked() {
            return ptr::null_mut();
        }

        // Update the heap start atomically
        self.heap_start.store(new_start, Ordering::Relaxed);

        // Return the aligned address
        aligned_start as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // This is a bump allocator, so we don't actually free memory
        // A real allocator would implement this
    }
}

unsafe impl Sync for BumpAllocator {}
#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator {
    heap_start: AtomicUsize::new(0),
    heap_end: UnsafeCell::new(0),
};

#[unsafe(no_mangle)]
fn kernel_main(_hartid: usize, dtb_pa: usize) {
    unsafe {
        let heap_start = ptr::addr_of!(__free_ram) as usize;
        let heap_end = heap_start + 16 * 1024 * 1024;

        // グローバルアロケータを初期化
        ALLOCATOR.heap_start.store(heap_start, Ordering::Relaxed);
        *ALLOCATOR.heap_end.get() = heap_end;

        // DMA領域はヒープの後から開始
        virtio::DMA_PADDR.store(heap_end, Ordering::SeqCst);
    }

    init_dt(dtb_pa);

    loop {}
}

fn init_dt(dtb: usize) {
    let fdt = unsafe { Fdt::from_ptr(dtb as *const u8).unwrap() };
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
    let mut gpu = VirtIOGpu::<HalImpl, T>::new(transport).unwrap();
    let (width, height) = gpu.resolution().unwrap();
    let width = width as usize;
    let height = height as usize;
    let fb = gpu.setup_framebuffer().unwrap();
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 4;
            fb[idx] = x as u8;
            fb[idx + 1] = y as u8;
            fb[idx + 2] = (x + y) as u8;
        }
    }
    gpu.flush().unwrap();
    for _ in 0..10000 {
        for _ in 0..10000 {
            unsafe {
                core::arch::asm!("nop");
            }
        }
    }
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
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("Panic: {}", info);
    loop {}
}
