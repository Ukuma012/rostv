use core::{
    ptr::NonNull,
    sync::atomic::{AtomicUsize, Ordering},
};
use lazy_static::lazy_static;
use virtio_drivers::{BufferDirection, Hal, PhysAddr, PAGE_SIZE};

unsafe extern "C" {
    static __free_ram: u64;
    static __free_ram_end: u64;
}

lazy_static! {
    pub static ref DMA_PADDR: AtomicUsize = AtomicUsize::new(0);
}

pub struct HalImpl;

unsafe impl Hal for HalImpl {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let paddr = DMA_PADDR.fetch_add(PAGE_SIZE * pages, Ordering::SeqCst);
        let aligned_paddr = (paddr + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        let vaddr = NonNull::new(aligned_paddr as _).expect("DMA allocation returned NULL pointer");
        (aligned_paddr, vaddr)
    }

    unsafe fn dma_dealloc(_paddr: PhysAddr, _vaddr: NonNull<u8>, pages: usize) -> i32 {
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, _size: usize) -> NonNull<u8> {
        NonNull::new(paddr as _).unwrap()
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> PhysAddr {
        let vaddr = buffer.as_ptr() as *mut u8 as usize;
        virt_to_phys(vaddr)
    }

    unsafe fn unshare(_paddr: PhysAddr, _buffer: NonNull<[u8]>, _direction: BufferDirection) {}
}

fn virt_to_phys(vaddr: usize) -> PhysAddr {
    vaddr
}
