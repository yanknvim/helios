#![no_main]
#![no_std]
#![feature(pointer_is_aligned_to)]

mod allocator;
mod console;
mod page;
mod trap;

use core::alloc::Layout;
use core::arch::naked_asm;

unsafe extern "C" {
    static _sheap: u8;
    static _eheap: u8;
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.boot")]
#[unsafe(naked)]
pub extern "C" fn boot() -> ! {
    naked_asm!(
        "la sp, __stack_top\n
        j {kernel_main}\n",
        kernel_main = sym kernel_main,
    );
}

#[unsafe(no_mangle)]
pub fn kernel_main() -> ! {
    let addr_trap_entry = trap::trap_entry as *const usize;
    unsafe {
        core::arch::asm!("csrw stvec, {trap_entry}", trap_entry = in(reg) addr_trap_entry);
    }

    unsafe {
        let start = core::ptr::addr_of!(_sheap) as usize;
        let end = core::ptr::addr_of!(_eheap) as usize;
        let size = end - start;

        crate::allocator::ALLOCATOR
            .lock()
            .init(start as *mut u8, size);
    }
    println!("Allocation Ready");

    extern crate alloc;

    let layout = Layout::from_size_align(4096, 4096).unwrap();
    let table = unsafe { alloc::alloc::alloc(layout) as *mut PageTable };
    page::init_page(table);

    let root_ppn = (table as u32) >> 12;
    let satp = (1 << 31) | root_ppn;

    unsafe {
        core::arch::asm!(
            "csrw satp, {satp}",
            "sfence.vma",
            satp = in(reg) satp,
        );
    }
    println!("Paging Ready");

    println!("Hello, World!");

    loop {}
}

use core::panic::PanicInfo;

use crate::page::PageTable;

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    println!("====== KERNEL PANIC! ======");
    println!("{}", info);

    sbi_rt::system_reset(sbi_rt::Shutdown, sbi_rt::NoReason);
    loop {}
}
